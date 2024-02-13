#!/usr/bin/env bash
set -o errexit   # abort on nonzero exitstatus
set -o nounset   # abort on unbound variable
set -o pipefail  # don't hide errors within pipes
set -x

#lightningcss --bundle --minify --sourcemap --output-file frontend/bundle.css frontend/index.css
#cargo fmt --check
#cargo deny check
#cargo hack clippy --workspace --feature-powerset --optional-deps --all-targets -- -D warnings
#cargo hack build --workspace --feature-powerset --optional-deps --all-targets
podman system service --time 0 &
./run-integration-tests.sh keycloak
./run-integration-tests.sh prepare
RUST_BACKTRACE=1 cargo hack test --workspace --feature-powerset --optional-deps --all-targets
