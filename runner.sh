#!/usr/bin/env bash
set -o errexit   # abort on nonzero exitstatus
set -o nounset   # abort on unbound variable
set -o pipefail  # don't hide errors within pipes
set -x

PREFIX=test- ./run-integration-tests.sh keycloak
PREFIX=test- ./run-integration-tests.sh backend-db-and-test $1 /bin/test
PREFIX=test- ./run-integration-tests.sh backend