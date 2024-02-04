#!/usr/bin/env bash
set -o errexit   # abort on nonzero exitstatus
set -o nounset   # abort on unbound variable
set -o pipefail  # don't hide errors within pipes

function cleanup {
    echo cleanup up pods
}

#trap cleanup EXIT INT

rm -R tmp
mkdir -p tmp
cp -r deployment/kustomize/* tmp/
cd tmp

podman build -t keycloak --file keycloak/keycloak/Dockerfile .
podman build -t perfect-group-allocation --file base/perfect-group-allocation/Dockerfile .
podman build -t test --file base/test/Dockerfile .

SERVER_BINARY=$(cargo build --bin server --message-format json | jq --raw-output 'select(.reason == "compiler-artifact" and .target.name == "server") | .executable')
echo "Compiled server binary: $SERVER_BINARY"

INTEGRATION_TEST_BINARY=$(cargo build --test webdriver --message-format json | jq --raw-output 'select(.reason == "compiler-artifact" and .target.name == "webdriver") | .executable')
echo "Compiled integration test binary: $INTEGRATION_TEST_BINARY"

CAROOT=$(mktemp -d)
echo "temporary CA directory: $CAROOT"

podman network create --ignore pga

(cd keycloak && CAROOT=$CAROOT mkcert keycloak)
(cd keycloak && kustomize edit set nameprefix tmp-)
(cd keycloak && kustomize edit add patch --patch '{"apiVersion": "v1","kind": "Pod","metadata":{"name":"keycloak"},"spec":{"volumes":[{"name":"root-ca","hostPath":{"path":"'"$CAROOT"'/rootCA.pem"}}]}}')
(cd keycloak && kustomize build --output kubernetes.yaml)
(cd keycloak && podman kube down --force kubernetes.yaml || exit 0) # WARNING: this also removes volumes
(cd keycloak && podman kube play --network pga kubernetes.yaml)
echo waiting for keycloak
podman wait --condition healthy tmp-keycloak-keycloak
echo keycloak started
podman exec tmp-keycloak-keycloak keytool -noprompt -import -file /run/rootCA.pem -alias rootCA -storepass password -keystore /tmp/.keycloak-truststore.jks
podman exec tmp-keycloak-keycloak /opt/keycloak/bin/kcadm.sh config truststore --trustpass password /tmp/.keycloak-truststore.jks
podman exec tmp-keycloak-keycloak /opt/keycloak/bin/kcadm.sh config credentials --server https://keycloak --realm master --user admin --password admin
podman exec tmp-keycloak-keycloak /opt/keycloak/bin/kcadm.sh create realms -s realm=pga -s enabled=true
podman exec tmp-keycloak-keycloak /opt/keycloak/bin/kcadm.sh create users -r pga -s username=test -s email=test@example.com -s enabled=true
podman exec tmp-keycloak-keycloak /opt/keycloak/bin/kcadm.sh set-password -r pga --username test --new-password test
podman exec tmp-keycloak-keycloak /opt/keycloak/bin/kcadm.sh create clients -r pga -s clientId=pga -s secret=$(cat base/client-secret) -s 'redirectUris=["https://h3.selfmade4u.de/*"]'

(cd base && CAROOT=$CAROOT mkcert perfect-group-allocation)
(cd base && kustomize edit set nameprefix tmp-)
(cd base && kustomize edit add patch --patch '{"apiVersion": "v1","kind": "Pod","metadata":{"name":"test"},"spec":{"volumes":[{"name":"root-ca","hostPath":{"path":"'"$CAROOT"'/rootCA.pem"}}]}}')
(cd base && kustomize edit add patch --patch '{"apiVersion": "v1","kind": "Pod","metadata":{"name":"test"},"spec":{"volumes":[{"name":"test-binary","hostPath":{"path":"'"$INTEGRATION_TEST_BINARY"'"}}]}}')
(cd base && kustomize edit add patch --patch '{"apiVersion": "v1","kind": "Pod","metadata":{"name":"perfect-group-allocation"},"spec":{"volumes":[{"name":"server-binary","hostPath":{"path":"'"$SERVER_BINARY"'"}}]}}')
(cd base && kustomize build --output kubernetes.yaml)
(cd base && podman kube down --force kubernetes.yaml || exit 0) # WARNING: this also removes volumes
(cd base && podman kube play --network pga kubernetes.yaml)
podman logs --color --names --follow tmp-keycloak-keycloak tmp-postgres-postgres tmp-perfect-group-allocation-perfect-group-allocation tmp-test-test
