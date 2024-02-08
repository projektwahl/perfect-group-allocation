#!/usr/bin/env bash
set -o errexit   # abort on nonzero exitstatus
set -o nounset   # abort on unbound variable
set -o pipefail  # don't hide errors within pipes

function cleanup {
    echo cleanup up pods
}

#trap cleanup EXIT INT

rm -R tmp/kustomize
mkdir -p tmp/certs
mkdir -p tmp/kustomize
cp -r deployment/kustomize/* tmp/kustomize

# don't rotate ca cert so we can keep keycloak running
cd tmp/certs
export CAROOT=$PWD
CAROOT=$CAROOT mkcert -CAROOT
cd ../..

cd tmp/kustomize

cargo build --bin server
SERVER_BINARY=$(cargo build --bin server --message-format json | jq --raw-output 'select(.reason == "compiler-artifact" and .target.name == "server") | .executable')
echo "Compiled server binary: $SERVER_BINARY"

cargo build --test webdriver
INTEGRATION_TEST_BINARY=$(cargo build --test webdriver --message-format json | jq --raw-output 'select(.reason == "compiler-artifact" and .target.name == "webdriver") | .executable')
echo "Compiled integration test binary: $INTEGRATION_TEST_BINARY"

# sudo podman inspect perfect-group-allocation-infra | jq ".[].NetworkSettings.Networks.[].IPAddress"
# dig tmp-perfect-group-allocation @10.89.1.1
# ping tmp-perfect-group-allocation

if [ "${1-}" == "keycloak" ]; then
    sudo podman build -t keycloak --file keycloak/keycloak/Dockerfile .
    sudo podman build -t perfect-group-allocation --file base/perfect-group-allocation/Dockerfile .
    sudo podman build -t test --file base/test/Dockerfile .

    (cd keycloak && CAROOT=$CAROOT mkcert keycloak)
    (cd keycloak && kustomize edit set nameprefix tmp-)
    (cd keycloak && kustomize edit add patch --patch '{"apiVersion": "v1","kind": "Pod","metadata":{"name":"keycloak"},"spec":{"volumes":[{"name":"root-ca","hostPath":{"path":"'"$CAROOT"'/rootCA.pem"}}]}}')
    (cd keycloak && kustomize build --output kubernetes.yaml)
    (cd keycloak && sudo podman kube down --force kubernetes.yaml || exit 0) # WARNING: this also removes volumes
    (cd keycloak && sudo podman kube play kubernetes.yaml)
    echo waiting for keycloak
    sudo podman wait --condition healthy tmp-keycloak-keycloak
    echo keycloak started
    sudo podman exec tmp-keycloak-keycloak keytool -noprompt -import -file /run/rootCA.pem -alias rootCA -storepass password -keystore /tmp/.keycloak-truststore.jks
    sudo podman exec tmp-keycloak-keycloak /opt/keycloak/bin/kcadm.sh config truststore --trustpass password /tmp/.keycloak-truststore.jks
    sudo podman exec tmp-keycloak-keycloak /opt/keycloak/bin/kcadm.sh config credentials --server https://keycloak --realm master --user admin --password admin
    sudo podman exec tmp-keycloak-keycloak /opt/keycloak/bin/kcadm.sh create realms -s realm=pga -s enabled=true
    sudo podman exec tmp-keycloak-keycloak /opt/keycloak/bin/kcadm.sh create users -r pga -s username=test -s email=test@example.com -s enabled=true
    sudo podman exec tmp-keycloak-keycloak /opt/keycloak/bin/kcadm.sh set-password -r pga --username test --new-password test
    sudo podman exec tmp-keycloak-keycloak /opt/keycloak/bin/kcadm.sh create clients -r pga -s clientId=pga -s secret=$(cat base/client-secret) -s 'redirectUris=["https://perfect-group-allocation/openidconnect-redirect"]'
fi

(cd base && CAROOT=$CAROOT mkcert perfect-group-allocation)
(cd base && kustomize edit set nameprefix tmp-)
(cd base && kustomize edit add patch --patch '{"apiVersion": "v1","kind": "Pod","metadata":{"name":"test"},"spec":{"volumes":[{"name":"test-binary","hostPath":{"path":"'"$INTEGRATION_TEST_BINARY"'"}}]}}')
(cd base && kustomize edit add patch --patch '{"apiVersion": "v1","kind": "Pod","metadata":{"name":"perfect-group-allocation"},"spec":{"volumes":[{"name":"root-ca","hostPath":{"path":"'"$CAROOT"'/rootCA.pem"}}]}}')
(cd base && kustomize edit add patch --patch '{"apiVersion": "v1","kind": "Pod","metadata":{"name":"perfect-group-allocation"},"spec":{"volumes":[{"name":"server-binary","hostPath":{"path":"'"$SERVER_BINARY"'"}}]}}')
(cd base && kustomize build --output kubernetes.yaml)
(cd base && sudo podman kube down --force kubernetes.yaml || exit 0) # WARNING: this also removes volumes
(cd base && sudo podman kube play kubernetes.yaml)
sudo podman logs --color --names --follow tmp-test-test tmp-perfect-group-allocation-perfect-group-allocation & # tmp-keycloak-keycloak tmp-postgres-postgres 
exit $(sudo podman wait tmp-test-test)
