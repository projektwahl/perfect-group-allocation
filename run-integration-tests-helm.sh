#!/usr/bin/env bash
set -o errexit   # abort on nonzero exitstatus
set -o nounset   # abort on unbound variable
set -o pipefail  # don't hide errors within pipes

mkdir -p tmp-helm/
cd tmp-helm/
export CAROOT=$PWD
CAROOT=$CAROOT mkcert -install # to allow local testing

cargo build --bin server
SERVER_BINARY=$(cargo build --bin server --message-format json | jq --raw-output 'select(.reason == "compiler-artifact" and .target.name == "server") | .executable')
echo "Compiled server binary: $SERVER_BINARY"

cargo build --test webdriver
INTEGRATION_TEST_BINARY=$(cargo build --test webdriver --message-format json | jq --raw-output 'select(.reason == "compiler-artifact" and .target.name == "webdriver") | .executable')
echo "Compiled integration test binary: $INTEGRATION_TEST_BINARY"

# we need to use rootful podman to get routable ip addresses.

helm template ../deployment/perfect-group-allocation/charts/perfect-group-allocation-postgres | sudo podman kube play --replace -
CAROOT=$CAROOT mkcert tmp-keycloak
helm template ../deployment/perfect-group-allocation/charts/perfect-group-allocation-keycloak --set-file cert=tmp-keycloak.pem --set-file key=tmp-keycloak-key.pem | sudo podman kube play --replace -

# https://kubernetes.io/docs/tasks/run-application/run-single-instance-stateful-application/

# ping release-name-pga-keycloak-pod
# sudo podman inspect perfect-group-allocation-infra | jq ".[].NetworkSettings.Networks.[].IPAddress"
# dig tmp-perfect-group-allocation @10.89.1.1
# ping tmp-perfect-group-allocation


#sudo podman logs --color --names --follow tmp-test-test tmp-perfect-group-allocation-tmp-perfect-group-allocation & # tmp-keycloak-tmp-keycloak tmp-postgres-postgres 
#exit $(sudo podman wait tmp-test-test)
