#!/usr/bin/env bash
set -o errexit   # abort on nonzero exitstatus
set -o nounset   # abort on unbound variable
set -o pipefail  # don't hide errors within pipes
set -x

# we should be able to use one keycloak for multiple tests.
KEYCLOAK_PREFIX=keycloak-tmp-

# exit the backgrounded jobs
trap "exit" INT TERM ERR
trap "kill 0" EXIT

mkdir -p tmp
cd tmp

CAROOT=$PWD
CAROOT=$CAROOT mkcert -CAROOT
CAROOT=$CAROOT mkcert -install # to allow local testing

# we need to use rootful podman to get routable ip addresses.

# https://kubernetes.io/docs/tasks/run-application/run-single-instance-stateful-application/

# in comparison to helm, kustomize has proper semantic merging

# dig tmp-perfect-group-allocation @10.89.1.1
# ping tmp-perfect-group-allocation

echo -n myawesomeclientsecret > client-secret

if [ "${1-}" == "keycloak" ]; then
    rm -f kustomization.yaml kubernetes.yaml && kustomize create
    kustomize edit add configmap root-ca --from-file=./rootCA.pem

    KEYCLOAK_IMAGE=$(sudo podman build --quiet --file ./deployment/kustomize/keycloak/keycloak/Dockerfile ..)
    kustomize edit set image keycloak=sha256:$KEYCLOAK_IMAGE

    kustomize edit set nameprefix $KEYCLOAK_PREFIX
    kustomize edit add resource ../deployment/kustomize/keycloak

    CAROOT=$CAROOT mkcert "${KEYCLOAK_PREFIX}keycloak"
    kustomize edit add secret keycloak-tls-cert \
        --type=kubernetes.io/tls \
        --from-file=tls.crt=./${KEYCLOAK_PREFIX}keycloak.pem \
        --from-file=tls.key=./${KEYCLOAK_PREFIX}keycloak-key.pem

    kustomize build --output kubernetes.yaml
    sudo podman kube down --force kubernetes.yaml || true # WARNING: this also removes volumes
    sudo podman kube play --replace kubernetes.yaml

    reset
    sudo podman logs --follow ${KEYCLOAK_PREFIX}keycloak-keycloak &
    echo waiting for keycloak
    sudo podman wait --condition healthy ${KEYCLOAK_PREFIX}keycloak-keycloak
    echo keycloak started
    sudo podman exec ${KEYCLOAK_PREFIX}keycloak-keycloak keytool -noprompt -import -file /run/rootCA/rootCA.pem -alias rootCA -storepass password -keystore /tmp/.keycloak-truststore.jks
    sudo podman exec ${KEYCLOAK_PREFIX}keycloak-keycloak /opt/keycloak/bin/kcadm.sh config truststore --trustpass password /tmp/.keycloak-truststore.jks
    sudo podman exec ${KEYCLOAK_PREFIX}keycloak-keycloak /opt/keycloak/bin/kcadm.sh config credentials --server https://${KEYCLOAK_PREFIX}keycloak --realm master --user admin --password admin
    sudo podman exec ${KEYCLOAK_PREFIX}keycloak-keycloak /opt/keycloak/bin/kcadm.sh create realms -s realm=pga -s enabled=true
    sudo podman exec ${KEYCLOAK_PREFIX}keycloak-keycloak /opt/keycloak/bin/kcadm.sh create users -r pga -s username=test -s email=test@example.com -s enabled=true
    sudo podman exec ${KEYCLOAK_PREFIX}keycloak-keycloak /opt/keycloak/bin/kcadm.sh set-password -r pga --username test --new-password test
    # TODO FIXME the redirect url is different
    # https://github.com/keycloak/keycloak/discussions/9278
    echo DO NOT RUN THIS IN PRODUCTION!!!
    sudo podman exec ${KEYCLOAK_PREFIX}keycloak-keycloak /opt/keycloak/bin/kcadm.sh create clients -r pga -s clientId=pga -s secret=$(cat client-secret) -s 'redirectUris=["*"]'
elif [ "${1-}" == "prepare" ]; then
    rm -f kustomization.yaml kubernetes.yaml && kustomize create
    kustomize edit add configmap root-ca --from-file=./rootCA.pem

    cargo build --bin server
    SERVER_BINARY=$(cargo build --bin server --message-format json | jq --raw-output 'select(.reason == "compiler-artifact" and .target.name == "server") | .executable')
    SERVER_BINARY=$(realpath --relative-to=.. $SERVER_BINARY)
    echo "Compiled server binary: $SERVER_BINARY"

    cargo build --test webdriver
    INTEGRATION_TEST_BINARY=$(cargo build --test webdriver --message-format json | jq --raw-output 'select(.reason == "compiler-artifact" and .target.name == "webdriver") | .executable')
    INTEGRATION_TEST_BINARY=$(realpath --relative-to=.. $INTEGRATION_TEST_BINARY)
    echo "Compiled integration test binary: $INTEGRATION_TEST_BINARY"

    # git describe --always --long --dirty 
    SERVER_IMAGE=$(sudo podman build --quiet --build-arg BINARY=$SERVER_BINARY --file ./deployment/kustomize/base/perfect-group-allocation/Dockerfile ..)
    kustomize edit set image perfect-group-allocation=sha256:$SERVER_IMAGE
    TEST_IMAGE=$(sudo podman build --quiet --build-arg BINARY=$INTEGRATION_TEST_BINARY --file ./deployment/kustomize/base/test/Dockerfile ..)
    kustomize edit set image test=sha256:$TEST_IMAGE
else
    # TODO FIXME copy to tmp folder
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
    sudo podman kube down --force kubernetes.yaml || true # WARNING: this also removes volumes
    sudo podman kube play --replace kubernetes.yaml
    #echo https://${PREFIX}perfect-group-allocation.dns.podman
    #sudo podman logs --color --names --follow ${PREFIX}test-test ${PREFIX}perfect-group-allocation-perfect-group-allocation ${KEYCLOAK_PREFIX}keycloak-keycloak & # ${PREFIX}postgres-postgres
    #(exit $(sudo podman wait ${PREFIX}test-test))
    #sudo podman kube down --force kubernetes.yaml || true # WARNING: this also removes volumes
fi