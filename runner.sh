#!/usr/bin/env bash
set -o errexit   # abort on nonzero exitstatus
set -o nounset   # abort on unbound variable
set -o pipefail  # don't hide errors within pipes
set -x

#./run-integration-tests.sh keycloak
PREFIX=e ./run-integration-tests.sh $1