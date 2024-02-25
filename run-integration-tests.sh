#!/usr/bin/env bash
set -o errexit   # abort on nonzero exitstatus
set -o nounset   # abort on unbound variable
set -o pipefail  # don't hide errors within pipes
set -x

PROJECT=$PWD

# we should be able to use one keycloak for multiple tests.
KEYCLOAK_PREFIX=keycloak-tmp-

# generate root certs as these are the only thing that is nice to persist (so keycloak gets the same root cert and your browser doesn't need to add a new root ca all the time)
mkdir -p rootca

export CAROOT=$PWD/rootca
GARBAGE=$(mktemp -d)

(cd $GARBAGE && echo -n myawesomeclientsecret > client-secret)

# we need to use rootful podman to get routable ip addresses.

# https://kubernetes.io/docs/tasks/run-application/run-single-instance-stateful-application/

# in comparison to helm, kustomize has proper semantic merging

# dig tmp-perfect-group-allocation @10.89.1.1
# ping tmp-perfect-group-allocation

if [ "${1-}" == "keycloak" ]; then
    cd "$GARBAGE"

    kustomize create

    KEYCLOAK_CONTAINERIGNORE=$(mktemp)
    echo -e '*' > "$KEYCLOAK_CONTAINERIGNORE"
    KEYCLOAK_IMAGE=$(podman build --file ./deployment/kustomize/keycloak/keycloak/Dockerfile "$PROJECT")
    KEYCLOAK_IMAGE=$(echo "$KEYCLOAK_IMAGE" | tail -n 1)
    kustomize edit set image keycloak=sha256:"$KEYCLOAK_IMAGE"

    kustomize edit set nameprefix $KEYCLOAK_PREFIX
    kustomize edit add resource ../../"$PROJECT"/deployment/kustomize/keycloak

    mkcert "${KEYCLOAK_PREFIX}keycloak"
    cp "$CAROOT"/rootCA.pem .
    kustomize edit add configmap root-ca --from-file=./rootCA.pem

    kustomize edit add secret keycloak-tls-cert \
        --type=kubernetes.io/tls \
        --from-file=tls.crt=./${KEYCLOAK_PREFIX}keycloak.pem \
        --from-file=tls.key=./${KEYCLOAK_PREFIX}keycloak-key.pem

    kustomize build --output kubernetes.yaml
    podman kube down --force kubernetes.yaml || true # WARNING: this also removes volumes
    podman kube play --replace kubernetes.yaml

    podman logs --color --names --follow ${KEYCLOAK_PREFIX}keycloak-keycloak & #  | sed 's/[\x01-\x1F\x7F]//g'
    echo waiting for keycloak
    # TODO refactor to directly loop on healthcheck?
    watch podman healthcheck run ${KEYCLOAK_PREFIX}keycloak-keycloak > /dev/null 2>&1 &
    podman wait --condition healthy ${KEYCLOAK_PREFIX}keycloak-keycloak
    echo keycloak started
    podman exec ${KEYCLOAK_PREFIX}keycloak-keycloak keytool -noprompt -import -file /run/rootCA/rootCA.pem -alias rootCA -storepass password -keystore /tmp/.keycloak-truststore.jks
    podman exec ${KEYCLOAK_PREFIX}keycloak-keycloak ls -la /opt/keycloak/
    podman exec ${KEYCLOAK_PREFIX}keycloak-keycloak id
    podman exec ${KEYCLOAK_PREFIX}keycloak-keycloak cat /etc/passwd
    podman exec ${KEYCLOAK_PREFIX}keycloak-keycloak /opt/keycloak/bin/kcadm.sh config truststore --trustpass password /tmp/.keycloak-truststore.jks
    podman exec ${KEYCLOAK_PREFIX}keycloak-keycloak /opt/keycloak/bin/kcadm.sh config credentials --server https://${KEYCLOAK_PREFIX}keycloak --realm master --user admin --password admin
    podman exec ${KEYCLOAK_PREFIX}keycloak-keycloak /opt/keycloak/bin/kcadm.sh create realms -s realm=pga -s enabled=true
    podman exec ${KEYCLOAK_PREFIX}keycloak-keycloak /opt/keycloak/bin/kcadm.sh create users -r pga -s username=test -s email=test@example.com -s enabled=true
    podman exec ${KEYCLOAK_PREFIX}keycloak-keycloak /opt/keycloak/bin/kcadm.sh set-password -r pga --username test --new-password test
    # TODO FIXME the redirect url is different
    # https://github.com/keycloak/keycloak/discussions/9278
    echo DO NOT RUN THIS IN PRODUCTION!!!
    podman exec ${KEYCLOAK_PREFIX}keycloak-keycloak /opt/keycloak/bin/kcadm.sh create clients -r pga -s clientId=pga -s secret=$(cat client-secret) -s 'redirectUris=["*"]'
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
    echo -e '*\n!'"$INTEGRATION_TEST_BINARY" > "$INTEGRATION_TEST_CONTAINERIGNORE"
    INTEGRATION_TEST_IMAGE=$(podman build --ignorefile "$INTEGRATION_TEST_CONTAINERIGNORE" --build-arg BINARY="$INTEGRATION_TEST_BINARY" --build-arg EXECUTABLE="$3" --file ./deployment/kustomize/base/test/Dockerfile "$PROJECT")
    INTEGRATION_TEST_IMAGE=$(echo "$INTEGRATION_TEST_IMAGE" | tail -n 1)
    kustomize edit set image test=sha256:"$INTEGRATION_TEST_IMAGE"

    kustomize edit add secret application-config \
        --from-literal=url=https://"${PREFIX}"perfect-group-allocation.dns.podman

    kustomize build --output kubernetes.yaml
    podman kube down --force kubernetes.yaml || true # WARNING: this also removes volumes
    podman kube play kubernetes.yaml # ahh kube uses another network
    #echo https://${PREFIX}perfect-group-allocation.dns.podman
    podman logs --color --names --follow "${PREFIX}"test-test & #${KEYCLOAK_PREFIX}keycloak-keycloak & # ${PREFIX}postgres-postgres
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
        --from-literal=openidconnect.issuer_url=https://${KEYCLOAK_PREFIX}keycloak/realms/pga \
        --from-literal="database_url=postgres://postgres:bestpassword@postgres/pga" \
        --from-literal=url=https://"${PREFIX}"perfect-group-allocation.dns.podman

    kustomize build --output kubernetes.yaml
    podman kube play --replace kubernetes.yaml # ahh kube uses another network
    #echo https://${PREFIX}perfect-group-allocation.dns.podman
    podman logs --color --names --follow "${PREFIX}"perfect-group-allocation-perfect-group-allocation & #${KEYCLOAK_PREFIX}keycloak-keycloak & # ${PREFIX}postgres-postgres
else
    echo "unknown command"
fi
