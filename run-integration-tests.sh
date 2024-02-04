#!/usr/bin/env bash
set -o errexit   # abort on nonzero exitstatus
set -o nounset   # abort on unbound variable
set -o pipefail  # don't hide errors within pipes

function cleanup {
    echo cleanup up pods
}

#trap cleanup EXIT INT

podman build -t keycloak --file deployment/kustomize/base/keycloak/Dockerfile .
podman build -t perfect-group-allocation --file deployment/kustomize/base/perfect-group-allocation/Dockerfile .
podman build -t test --file deployment/kustomize/base/test/Dockerfile .

SERVER_BINARY=$(cargo build --bin server --message-format json | jq --raw-output 'select(.reason == "compiler-artifact" and .target.name == "server") | .executable')
echo "Compiled server binary: $SERVER_BINARY"

INTEGRATION_TEST_BINARY=$(cargo build --test webdriver --message-format json | jq --raw-output 'select(.reason == "compiler-artifact" and .target.name == "webdriver") | .executable')
echo "Compiled integration test binary: $INTEGRATION_TEST_BINARY"

CAROOT=$(mktemp -d)
echo "temporary CA directory: $CAROOT"

mkdir -p tmp
cp -r deployment/kustomize/base/* tmp/
(
    cd tmp &&
    CAROOT=$CAROOT mkcert keycloak &&
    CAROOT=$CAROOT mkcert perfect-group-allocation &&
    kustomize edit set nameprefix tmp- &&
    kustomize edit add patch --patch '{"apiVersion": "v1","kind": "Pod","metadata":{"name":"keycloak"},"spec":{"volumes":[{"name":"root-ca","hostPath":{"path":"'"$CAROOT"'/rootCA.pem"}}]}}' &&
    kustomize edit add patch --patch '{"apiVersion": "v1","kind": "Pod","metadata":{"name":"test"},"spec":{"volumes":[{"name":"root-ca","hostPath":{"path":"'"$CAROOT"'/rootCA.pem"}}]}}' &&
    kustomize edit add patch --patch '{"apiVersion": "v1","kind": "Pod","metadata":{"name":"test"},"spec":{"volumes":[{"name":"test-binary","hostPath":{"path":"'"$INTEGRATION_TEST_BINARY"'"}}]}}' &&
    kustomize edit add patch --patch '{"apiVersion": "v1","kind": "Pod","metadata":{"name":"perfect-group-allocation"},"spec":{"volumes":[{"name":"server-binary","hostPath":{"path":"'"$SERVER_BINARY"'"}}]}}' &&
    kustomize build --output kubernetes.yaml &&
    (podman kube down --force kubernetes.yaml || exit 0) && # WARNING: this also removes volumes
    podman kube play kubernetes.yaml &&
    podman wait --condition healthy tmp-keycloak-keycloak &&
    podman exec tmp-keycloak-keycloak keytool -noprompt -import -file /run/rootCA.pem -alias rootCA -storepass password -keystore /tmp/.keycloak-truststore.jks &&
    podman exec tmp-keycloak-keycloak /opt/keycloak/bin/kcadm.sh config truststore --trustpass password /tmp/.keycloak-truststore.jks &&
    podman exec tmp-keycloak-keycloak /opt/keycloak/bin/kcadm.sh config credentials --server https://keycloak:8443 --realm master --user admin --password admin &&
    #export PATH=$PATH:/opt/keycloak/bin
    #kcadm.sh config credentials --server http://localhost:8080 --realm master --user admin --password admin
    #kcadm.sh create realms -s realm=pga -s enabled=true
    #kcadm.sh create users -r pga -s username=test -s email=test@example.com -s enabled=true
    #kcadm.sh set-password -r pga --username test --new-password test
    #CID=$(kcadm.sh create clients -r pga -s clientId=pga -s 'redirectUris=["https://h3.selfmade4u.de/*"]' -i)
    #CID=$(kcadm.sh get clients -r pga --fields id -q clientId=pga --format csv --noquotes)
    #CLIENT_SECRET=$(kcadm.sh get clients/$CID/client-secret -r pga --fields value --format csv --noquotes)
    #echo $CLIENT_SECRET
    podman logs --color --names --follow tmp-test-test #tmp-keycloak-keycloak tmp-postgres-postgres tmp-perfect-group-allocation-perfect-group-allocation

)
