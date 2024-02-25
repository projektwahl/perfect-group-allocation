#!/usr/bin/env bash
set -o errexit   # abort on nonzero exitstatus
set -o nounset   # abort on unbound variable
set -o pipefail  # don't hide errors within pipes
set -x

PROJECT=$PWD

# generate root certs as these are the only thing that is nice to persist (so keycloak gets the same root cert and your browser doesn't need to add a new root ca all the time)
mkdir -p rootca

export CAROOT=$PWD/rootca
GARBAGE=$(mktemp -d)

(cd $GARBAGE && echo -n myawesomeclientsecret > client-secret)

# we need to use rootful podman to get routable ip addresses.

# https://kubernetes.io/docs/tasks/run-application/run-single-instance-stateful-application/

if [ "${1-}" == "keycloak" ]; then
    # https://keycloak-tmp-keycloak/
    cd "$GARBAGE"

    KEYCLOAK_CONTAINERIGNORE=$(mktemp)
    echo -e '*' > "$KEYCLOAK_CONTAINERIGNORE"
    KEYCLOAK_IMAGE=$(podman build --file ./deployment/kustomize/keycloak/keycloak/Dockerfile "$PROJECT")
    KEYCLOAK_IMAGE=$(echo "$KEYCLOAK_IMAGE" | tail -n 1)

    mkcert "${PREFIX}keycloak"
    cp "$CAROOT"/rootCA.pem .
    kustomize edit add configmap root-ca --from-file=./rootCA.pem

    helm template $PREFIX . \
        --set-file keycloak-cert=./${PREFIX}keycloak.pem \
        --set-file keycloak-key=./${PREFIX}keycloak-key.pem \
        --set keycloak=sha256:"$KEYCLOAK_IMAGE" \
        > kubernetes.yaml
    podman kube down --force kubernetes.yaml || true # WARNING: this also removes volumes
    podman kube play --replace kubernetes.yaml

    podman logs --color --names --follow ${PREFIX}keycloak-keycloak & #  | sed 's/[\x01-\x1F\x7F]//g'
    echo waiting for keycloak
    # TODO refactor to directly loop on healthcheck?
    watch podman healthcheck run ${PREFIX}keycloak-keycloak > /dev/null 2>&1 &
    podman wait --condition healthy ${PREFIX}keycloak-keycloak
    echo keycloak started
    podman exec ${PREFIX}keycloak-keycloak keytool -noprompt -import -file /run/rootCA/rootCA.pem -alias rootCA -storepass password -keystore /tmp/.keycloak-truststore.jks
    podman exec ${PREFIX}keycloak-keycloak ls -la /opt/keycloak/
    podman exec ${PREFIX}keycloak-keycloak id
    podman exec ${PREFIX}keycloak-keycloak cat /etc/passwd
    podman exec ${PREFIX}keycloak-keycloak /opt/keycloak/bin/kcadm.sh config truststore --trustpass password /tmp/.keycloak-truststore.jks
    podman exec ${PREFIX}keycloak-keycloak /opt/keycloak/bin/kcadm.sh config credentials --server https://${PREFIX}keycloak --realm master --user admin --password admin
    podman exec ${PREFIX}keycloak-keycloak /opt/keycloak/bin/kcadm.sh create realms -s realm=pga -s enabled=true
    podman exec ${PREFIX}keycloak-keycloak /opt/keycloak/bin/kcadm.sh create users -r pga -s username=test -s email=test@example.com -s enabled=true
    podman exec ${PREFIX}keycloak-keycloak /opt/keycloak/bin/kcadm.sh set-password -r pga --username test --new-password test
    podman exec ${PREFIX}keycloak-keycloak /opt/keycloak/bin/kcadm.sh create clients -r pga -s clientId=pga -s secret=$(cat client-secret) -s 'redirectUris=["https://'${PREFIX}'perfect-group-allocation/openidconnect-redirect"]'
elif [ "${1-}" == "backend-db-and-test" ]; then
    cd "$GARBAGE"

    kustomize create
    cp "$PROJECT"/deployment/kustomize/base/postgres.yaml .
    kustomize edit add resource ./postgres.yaml
    cp "$PROJECT"/deployment/kustomize/base/test.yaml .
    kustomize edit add resource ./test.yaml
    kustomize edit set nameprefix "$PREFIX"

    INTEGRATION_TEST_BINARY=$(realpath --relative-to="$PROJECT" "$2")
    INTEGRATION_TEST_CONTAINERIGNORE=$(mktemp)
    echo -e '*\n!rootca/rootCA.pem\n!'"$INTEGRATION_TEST_BINARY" > "$INTEGRATION_TEST_CONTAINERIGNORE"
    INTEGRATION_TEST_IMAGE=$(podman build --ignorefile "$INTEGRATION_TEST_CONTAINERIGNORE" --build-arg BINARY="$INTEGRATION_TEST_BINARY" --build-arg EXECUTABLE="$3" --file ./deployment/kustomize/base/test/Dockerfile "$PROJECT")
    INTEGRATION_TEST_IMAGE=$(echo "$INTEGRATION_TEST_IMAGE" | tail -n 1)
    kustomize edit set image test=sha256:"$INTEGRATION_TEST_IMAGE"

    kustomize edit add secret application-config \
        --from-literal=url=https://"${PREFIX}"perfect-group-allocation

    kustomize build --output kubernetes.yaml
    podman kube down --force kubernetes.yaml || true # WARNING: this also removes volumes
    podman kube play kubernetes.yaml # ahh kube uses another network
    podman logs --color --names --follow "${PREFIX}"test-test & #${PREFIX}keycloak-keycloak & # ${PREFIX}postgres-postgres
    (exit $(podman wait "${PREFIX}"test-test))
    podman kube down --force kubernetes.yaml || true # WARNING: this also removes volumes
elif [ "${1-}" == "backend" ]; then
    cargo build --bin server
    cd "$GARBAGE"

    kustomize create

    SERVER_CONTAINERIGNORE=$(mktemp)
    echo -e '*\n!target/debug/server\n!deployment/kustomize/base/perfect-group-allocation/Dockerfile' > "$SERVER_CONTAINERIGNORE"
    # tag with: git describe --always --long --dirty 
    SERVER_IMAGE=$(podman build --ignorefile "$SERVER_CONTAINERIGNORE" --build-arg BINARY=./target/debug/server --file ./deployment/kustomize/base/perfect-group-allocation/Dockerfile "$PROJECT")
    kustomize edit set image perfect-group-allocation=sha256:$(echo "$SERVER_IMAGE" | tail -n 1)

    cp "$PROJECT"/deployment/kustomize/base/perfect-group-allocation.yaml .
    kustomize edit add resource ./perfect-group-allocation.yaml
    kustomize edit set nameprefix "$PREFIX"

    if [ ! -f "$CAROOT"/"${PREFIX}"perfect-group-allocation-key.pem ]; then
        (cd $CAROOT && mkcert "${PREFIX}perfect-group-allocation")
    fi

    cp "$CAROOT"/rootCA.pem .
    cp "$CAROOT"/"${PREFIX}"perfect-group-allocation.pem .
    cp "$CAROOT"/"${PREFIX}"perfect-group-allocation-key.pem .
    kustomize edit add configmap root-ca --from-file=./rootCA.pem

    kustomize edit add secret application-config \
        --from-file=tls.crt=./"${PREFIX}"perfect-group-allocation.pem \
        --from-file=tls.key=./"${PREFIX}"perfect-group-allocation-key.pem \
        --from-file=openidconnect.client_secret=./client-secret \
        --from-literal=openidconnect.client_id=pga \
        --from-literal=openidconnect.issuer_url=https://${PREFIX}keycloak/realms/pga \
        --from-literal="database_url=postgres://postgres:bestpassword@postgres/pga" \
        --from-literal=url=https://"${PREFIX}"perfect-group-allocation

    kustomize build --output kubernetes.yaml
    # podman inspect devperfect-group-allocation-perfect-group-allocation
    # restarting the container changes ip, the podman dns has a ttl of 60 seconds so we need to have a persistent ip
    podman kube play --ip=10.89.0.8 --replace kubernetes.yaml # ahh kube uses another network
    echo https://${PREFIX}perfect-group-allocation
    podman logs --color --names --follow "${PREFIX}"perfect-group-allocation-perfect-group-allocation & #${PREFIX}keycloak-keycloak & # ${PREFIX}postgres-postgres
else
    echo "unknown command"
fi
