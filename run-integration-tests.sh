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

mkdir -p tmp
cd tmp

CAROOT=$PWD
CAROOT=$CAROOT mkcert -CAROOT
CAROOT=$CAROOT mkcert -install # to allow local testing

# we need to use rootful podman to get routable ip addresses.

# https://kubernetes.io/docs/tasks/run-application/run-single-instance-stateful-application/

# sudo podman inspect perfect-group-allocation-infra | jq ".[].NetworkSettings.Networks.[].IPAddress"
# dig tmp-perfect-group-allocation @10.89.1.1
# ping tmp-perfect-group-allocation

if [ "${1-}" == "keycloak" ]; then
    sudo podman build -t keycloak --file ../deployment/kustomize/keycloak/Dockerfile ..

    rm -f kustomization.yaml kubernetes.yaml && kustomize create --nameprefix $PREFIX --resources ../deployment/kustomize/keycloak

    # https://kubectl.docs.kubernetes.io/references/kustomize/
    # https://kubectl.docs.kubernetes.io/references/kustomize/kustomization/
    # we should definitely use a tool that understands about the semantics of merging different config parts or is at least not as bad as helm
    # https://kubectl.docs.kubernetes.io/guides/config_management/components/ looks really interesting because we also want to enable and disable some components (like keycloak)
    # valueFrom probably also really interesting
    # https://kubernetes.io/docs/tasks/inject-data-application/environment-variable-expose-pod-information/
    # https://kubernetes.io/docs/tasks/inject-data-application/define-interdependent-environment-variables/
    # https://kubernetes.io/docs/reference/kubernetes-api/workload-resources/pod-v1/#environment-variables
    # all these *From look really interesting
    # potentially best doc https://github.com/kubernetes-sigs/kustomize
    # because website seems outdated?
    # https://github.com/kubernetes-sigs/kustomize/blob/master/cmd/config/docs/commands/merge3.md
    # https://github.com/kubernetes-sigs/kustomize/tree/master/site/content/en/docs/Reference/API/
    # we have a fixed but changing domain name so probably not useful. we could use one keycloak for all instances though? I think that would make sense. and then use a separate postgres for keycloak
    # sudo podman container checkpoint --tcp-established tmp-keycloak-tmp-keycloak
    # sudo podman container restore --tcp-established --keep tmp-keycloak-tmp-keycloak && sudo podman wait --condition healthy tmp-keycloak-tmp-keycloak
    # sudo podman stop tmp-keycloak-tmp-keycloak
    # sudo podman start tmp-keycloak-tmp-keycloak && sudo podman wait --condition healthy tmp-keycloak-tmp-keycloak

    CAROOT=$CAROOT mkcert tmp-keycloak

    kustomize edit add patch --patch '{"apiVersion": "v1","kind": "Pod","metadata":{"name":"keycloak"},"spec":{"volumes":[{"name":"root-ca","hostPath":{"path":"'"$CAROOT"'/rootCA.pem"}}]}}' # it would be nice if we would only need to specify this once
    
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
    echo "Compiled server binary: $SERVER_BINARY"

    cargo build --test webdriver
    INTEGRATION_TEST_BINARY=$(cargo build --test webdriver --message-format json | jq --raw-output 'select(.reason == "compiler-artifact" and .target.name == "webdriver") | .executable')
    echo "Compiled integration test binary: $INTEGRATION_TEST_BINARY"

    sudo podman build -t perfect-group-allocation --build-arg BINARY=$SERVER_BINARY --file ../deployment/kustomize/base/perfect-group-allocation/Dockerfile ..
    sudo podman build -t test --build-arg BINARY=$INTEGRATION_TEST_BINARY --file ../deployment/kustomize/base/test/Dockerfile ..

    rm -f kustomization.yaml kubernetes.yaml && kustomize create --nameprefix $PREFIX --resources ../deployment/kustomize/base



    CAROOT=$CAROOT mkcert tmp-perfect-group-allocation
    kustomize edit add patch --patch '{"apiVersion": "v1","kind": "Pod","metadata":{"name":"test"},"spec":{"volumes":[{"name":"test-binary","hostPath":{"path":"'"$INTEGRATION_TEST_BINARY"'"}}]}}' # maybe we should build container instead
    kustomize edit add patch --patch '{"apiVersion": "v1","kind": "Pod","metadata":{"name":"perfect-group-allocation"},"spec":{"volumes":[{"name":"root-ca","hostPath":{"path":"'"$CAROOT"'/rootCA.pem"}}]}}'
    kustomize edit add patch --patch '{"apiVersion": "v1","kind": "Pod","metadata":{"name":"perfect-group-allocation"},"spec":{"volumes":[{"name":"server-binary","hostPath":{"path":"'"$SERVER_BINARY"'"}}]}}'
    kustomize build --output kubernetes.yaml
    sudo podman kube down --force kubernetes.yaml || exit 0 # WARNING: this also removes volumes
    sudo podman kube play kubernetes.yaml
    sudo podman logs --color --names --follow tmp-test-test tmp-perfect-group-allocation-tmp-perfect-group-allocation & # tmp-keycloak-tmp-keycloak tmp-postgres-postgres 
    exit $(sudo podman wait tmp-test-test)

fi