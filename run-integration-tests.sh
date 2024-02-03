#!/usr/bin/env bash
set -o errexit   # abort on nonzero exitstatus
set -o nounset   # abort on unbound variable
set -o pipefail  # don't hide errors within pipes

function cleanup {
    echo cleanup up pods
    podman pod stop tmp-keycloak tmp-postgres tmp-webdriver-pod tmp-perfect-group-allocation
    podman pod rm tmp-keycloak tmp-postgres tmp-webdriver-pod tmp-perfect-group-allocation
    podman volume rm tmp-postgres-claim 
}

#trap cleanup EXIT INT

cargo build --bin server

INTEGRATION_TEST_BINARY=$(cargo build --test webdriver --message-format json | jq --raw-output 'select(.reason == "compiler-artifact" and .target.name == "webdriver") | .executable')
echo "Compiled integration test binary: $INTEGRATION_TEST_BINARY"

cp -r deployment/kustomize/base/* tmp/
(
    cd tmp &&
    mkcert keycloak &&
    mkcert perfect-group-allocation &&
    kustomize edit set nameprefix tmp- &&
    (kustomize build | podman kube down --force - || exit 0) && # WARNING: this also removes volumes
    kustomize build | podman kube play - &&
    podman logs --color --names --follow tmp-keycloak-keycloak tmp-postgres-postgres tmp-webdriver-pod-webdriver

)
