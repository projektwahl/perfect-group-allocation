#!/usr/bin/env bash
set -o errexit   # abort on nonzero exitstatus
set -o nounset   # abort on unbound variable
set -o pipefail  # don't hide errors within pipes

PREFIX=tmp
# we should be able to use one keycloak for multiple tests
KEYCLOAK_PREFIX=tmp

function cleanup {
    echo cleanup up pods
}

#trap cleanup EXIT INT

#mkdir -p tmp
#cd tmp

CAROOT=$PWD
CAROOT=$CAROOT mkcert -CAROOT
CAROOT=$CAROOT mkcert -install # to allow local testing

# we need to use rootful podman to get routable ip addresses.

# https://kubernetes.io/docs/tasks/run-application/run-single-instance-stateful-application/

# in comparison to helm, kustomize has proper semantic merging

# dig tmp-perfect-group-allocation @10.89.1.1
# ping tmp-perfect-group-allocation

rm -f kustomization.yaml kubernetes.yaml && kustomize create
kustomize edit add configmap root-ca --from-file=./rootCA.pem

if [ "${1-}" == "keycloak" ]; then
    sudo podman build -t keycloak --file ../deployment/kustomize/keycloak/Dockerfile ..

    kustomize edit set nameprefix $KEYCLOAK_PREFIX
    kustomize edit add resource ../deployment/kustomize/keycloak

    CAROOT=$CAROOT mkcert tmp-keycloak

    kustomize build --output kubernetes.yaml
    sudo podman kube down --force kubernetes.yaml || exit 0 # WARNING: this also removes volumes
    sudo podman kube play kuberetes.yaml

    echo waiting for keycloak
    sudo podman wait --condition healthy tmp-keycloak-tmp-keycloak
    echo keycloak started
    sudo podman exec tmp-keycloak-tmp-keycloak keytool -noprompt -import -file /run/rootCA.pem -alias rootCA -storepass password -keystore /tmp/.keycloak-truststore.jks
    sudo podman exec tmp-keycloak-tmp-keycloak /opt/keycloak/bin/kcadm.sh config truststore --trustpass password /tmp/.keycloak-truststore.jks
    sudo podman exec tmp-keycloak-tmp-keycloak /opt/keycloak/bin/kcadm.sh config credentials --server https://tmp-keycloak --realm master --user admin --password admin
    sudo podman exec tmp-keycloak-tmp-keycloak /opt/keycloak/bin/kcadm.sh create realms -s realm=pga -s enabled=true
    sudo podman exec tmp-keycloak-tmp-keycloak /opt/keycloak/bin/kcadm.sh create users -r pga -s username=test -s email=test@example.com -s enabled=true
    sudo podman exec tmp-keycloak-tmp-keycloak /opt/keycloak/bin/kcadm.sh set-password -r pga --username test --new-password test
    sudo podman exec tmp-keycloak-tmp-keycloak /opt/keycloak/bin/kcadm.sh create clients -r pga -s clientId=pga -s secret=$(cat base/client-secret) -s 'redirectUris=["https://tmp-perfect-group-allocation/openidconnect-redirect"]'
else
    cargo build --bin server
    SERVER_BINARY=$(cargo build --bin server --message-format json | jq --raw-output 'select(.reason == "compiler-artifact" and .target.name == "server") | .executable')
    SERVER_BINARY=$(realpath --relative-to=.. $SERVER_BINARY)
    echo "Compiled server binary: $SERVER_BINARY"

    cargo build --test webdriver
    INTEGRATION_TEST_BINARY=$(cargo build --test webdriver --message-format json | jq --raw-output 'select(.reason == "compiler-artifact" and .target.name == "webdriver") | .executable')
    INTEGRATION_TEST_BINARY=$(realpath --relative-to=.. $INTEGRATION_TEST_BINARY)
    echo "Compiled integration test binary: $INTEGRATION_TEST_BINARY"

    # git describe --always --long --dirty 
    sudo podman build --build-arg BINARY=$SERVER_BINARY --file ../deployment/kustomize/base/perfect-group-allocation/Dockerfile ..
    sudo podman build --build-arg BINARY=$INTEGRATION_TEST_BINARY --file ../deployment/kustomize/base/test/Dockerfile ..

    kustomize edit set nameprefix $PREFIX
    kustomize edit add resource ../deployment/kustomize/base

    CAROOT=$CAROOT mkcert tmp-perfect-group-allocation
    kustomize edit add secret application-config --from-file=tls.cert=tmp-perfect-group-allocation.pem --from-file=tls.key=tmp-perfect-group-allocation-key.pem

    kustomize build --output kubernetes.yaml
    sudo podman kube down --force kubernetes.yaml || exit 0 # WARNING: this also removes volumes
    sudo podman kube play kubernetes.yaml
    sudo podman logs --color --names --follow tmp-test-test tmp-perfect-group-allocation-tmp-perfect-group-allocation & # tmp-keycloak-tmp-keycloak tmp-postgres-postgres 
    exit $(sudo podman wait tmp-test-test)

fi