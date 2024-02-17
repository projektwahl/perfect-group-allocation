#!/usr/bin/env bash
set -o errexit   # abort on nonzero exitstatus
set -o nounset   # abort on unbound variable
set -o pipefail  # don't hide errors within pipes
set -x

# we should be able to use one keycloak for multiple tests.
KEYCLOAK_PREFIX=keycloak-tmp-

echo "in $PWD"
ls -la
mkdir -p tmp
cd tmp

CAROOT=$PWD

# we need to use rootful podman to get routable ip addresses.

# https://kubernetes.io/docs/tasks/run-application/run-single-instance-stateful-application/

# in comparison to helm, kustomize has proper semantic merging

# dig tmp-perfect-group-allocation @10.89.1.1
# ping tmp-perfect-group-allocation

if [ "${1-}" == "keycloak" ]; then
    id
    echo -n myawesomeclientsecret > client-secret

    CAROOT=$CAROOT mkcert -CAROOT
    #CAROOT=$CAROOT mkcert -install # to allow local testing

    rm -f kustomization.yaml kubernetes.yaml && kustomize create
    kustomize edit add configmap root-ca --from-file=./rootCA.pem

    id
    cat /etc/subuid
    cat /etc/subgid
    cat /proc/self/uid_map
    podman build --file ./deployment/kustomize/keycloak/keycloak/Dockerfile ..
    KEYCLOAK_IMAGE=$(podman build --file ./deployment/kustomize/keycloak/keycloak/Dockerfile ..)
    kustomize edit set image keycloak=sha256:$(echo "$KEYCLOAK_IMAGE" | tail -n 1)

    kustomize edit set nameprefix $KEYCLOAK_PREFIX
    kustomize edit add resource ../deployment/kustomize/keycloak

    CAROOT=$CAROOT mkcert "${KEYCLOAK_PREFIX}keycloak"
    kustomize edit add secret keycloak-tls-cert \
        --type=kubernetes.io/tls \
        --from-file=tls.crt=./${KEYCLOAK_PREFIX}keycloak.pem \
        --from-file=tls.key=./${KEYCLOAK_PREFIX}keycloak-key.pem

    kustomize build --output kubernetes.yaml
    podman kube down --force kubernetes.yaml || true # WARNING: this also removes volumes
    podman kube play --replace kubernetes.yaml

    podman logs --follow ${KEYCLOAK_PREFIX}keycloak-keycloak &
    echo waiting for keycloak
    sleep 30
    podman healthcheck run ${KEYCLOAK_PREFIX}keycloak-keycloak 
    #watch podman healthcheck run ${KEYCLOAK_PREFIX}keycloak-keycloak &>/dev/null & # potentially still fails because no terminal?
    podman wait --condition healthy ${KEYCLOAK_PREFIX}keycloak-keycloak
    echo keycloak started
    podman exec ${KEYCLOAK_PREFIX}keycloak-keycloak keytool -noprompt -import -file /run/rootCA/rootCA.pem -alias rootCA -storepass password -keystore /tmp/.keycloak-truststore.jks
    podman exec ${KEYCLOAK_PREFIX}keycloak-keycloak /opt/keycloak/bin/kcadm.sh config truststore --trustpass password /tmp/.keycloak-truststore.jks
    podman exec ${KEYCLOAK_PREFIX}keycloak-keycloak /opt/keycloak/bin/kcadm.sh config credentials --server https://${KEYCLOAK_PREFIX}keycloak --realm master --user admin --password admin
    podman exec ${KEYCLOAK_PREFIX}keycloak-keycloak /opt/keycloak/bin/kcadm.sh create realms -s realm=pga -s enabled=true
    podman exec ${KEYCLOAK_PREFIX}keycloak-keycloak /opt/keycloak/bin/kcadm.sh create users -r pga -s username=test -s email=test@example.com -s enabled=true
    podman exec ${KEYCLOAK_PREFIX}keycloak-keycloak /opt/keycloak/bin/kcadm.sh set-password -r pga --username test --new-password test
    # TODO FIXME the redirect url is different
    # https://github.com/keycloak/keycloak/discussions/9278
    echo DO NOT RUN THIS IN PRODUCTION!!!
    podman exec ${KEYCLOAK_PREFIX}keycloak-keycloak /opt/keycloak/bin/kcadm.sh create clients -r pga -s clientId=pga -s secret=$(cat client-secret) -s 'redirectUris=["*"]'
elif [ "${1-}" == "prepare" ]; then
    # TODO FIXME we need to get these into the container
    rm -f kustomization.yaml kubernetes.yaml && kustomize create
    kustomize edit add configmap root-ca --from-file=./rootCA.pem

    cargo build --bin server
    SERVER_BINARY=$(cargo build --bin server --message-format json | jq --raw-output 'select(.reason == "compiler-artifact" and .target.name == "server") | .executable')
    SERVER_BINARY=$(realpath --relative-to=.. $SERVER_BINARY)
    echo "Compiled server binary: $SERVER_BINARY"

    #cargo build --test webdriver
    #INTEGRATION_TEST_BINARY=$(cargo build --test webdriver --message-format json | jq --raw-output 'select(.reason == "compiler-artifact" and .target.name == "webdriver") | .executable')
    #INTEGRATION_TEST_BINARY=$(realpath --relative-to=.. $INTEGRATION_TEST_BINARY)
    #echo "Compiled integration test binary: $INTEGRATION_TEST_BINARY"

    # git describe --always --long --dirty 
    SERVER_IMAGE=$(podman build --build-arg BINARY=$SERVER_BINARY --file ./deployment/kustomize/base/perfect-group-allocation/Dockerfile ..)
    kustomize edit set image perfect-group-allocation=sha256:$(echo "$SERVER_IMAGE" | tail -n 1)
    #TEST_IMAGE=$(podman build --quiet --build-arg BINARY=$INTEGRATION_TEST_BINARY --file ./deployment/kustomize/base/test/Dockerfile ..)
    #kustomize edit set image test=sha256:$TEST_IMAGE

    rm -f pga.tar
    #podman image save -o pga.tar sha256:$SERVER_IMAGE
else
    KTMP=$(mktemp -d)
    cp ./{kustomization.yaml,client-secret,rootCA.pem} $KTMP/
    cd $KTMP

    kustomize edit add resource ../../$CAROOT/../deployment/kustomize/base/
    kustomize edit set nameprefix $PREFIX

    CAROOT=$CAROOT mkcert "${PREFIX}perfect-group-allocation" # maybe use a wildcard certificate instead? to speed this up
    kustomize edit add secret application-config \
        --from-file=tls.crt=./${PREFIX}perfect-group-allocation.pem \
        --from-file=tls.key=./${PREFIX}perfect-group-allocation-key.pem \
        --from-file=openidconnect.client_secret=./client-secret \
        --from-literal=openidconnect.client_id=pga \
        --from-literal=openidconnect.issuer_url=https://${KEYCLOAK_PREFIX}keycloak/realms/pga \
        --from-literal="database_url=postgres://postgres@postgres/pga?sslmode=disable" \
        --from-literal=url=https://${PREFIX}perfect-group-allocation.dns.podman \

    kustomize build --output kubernetes.yaml
    id
    groups
    cat /sys/fs/cgroup/cgroup.controllers
    podman --remote kube down --force kubernetes.yaml || true # WARNING: this also removes volumes
    podman --remote kube play kubernetes.yaml # ahh kube uses another network
    #echo https://${PREFIX}perfect-group-allocation.dns.podman
    #podman logs --color --names --follow ${PREFIX}test-test ${PREFIX}perfect-group-allocation-perfect-group-allocation ${KEYCLOAK_PREFIX}keycloak-keycloak & # ${PREFIX}postgres-postgres
    #(exit $(podman wait ${PREFIX}test-test))
    #podman kube down --force kubernetes.yaml || true # WARNING: this also removes volumes
fi