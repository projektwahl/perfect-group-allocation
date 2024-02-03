#!/usr/bin/env bash
set -o errexit   # abort on nonzero exitstatus
set -o nounset   # abort on unbound variable
set -o pipefail  # don't hide errors within pipes

INTEGRATION_TEST_BINARY=$(cargo build --test webdriver --message-format json | jq --raw-output 'select(.reason == "compiler-artifact" and .target.name == "webdriver") | .executable')
echo "Compiled integration test binary: $INTEGRATION_TEST_BINARY"

cp -r deployment/kustomize/base/ tmp
(
    cd tmp &&
    mkcert keycloak &&
    mkcert perfect-group-allocation &&
    kustomize edit set nameprefix tmp &&
    kustomize build | podman kube play -
)
