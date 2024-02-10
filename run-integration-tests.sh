#!/usr/bin/env bash
set -o errexit   # abort on nonzero exitstatus
set -o nounset   # abort on unbound variable
set -o pipefail  # don't hide errors within pipes

function cleanup {
    echo cleanup up pods
}

#trap cleanup EXIT INT

# use a tls secret if possible?
# helm template --set-file cert=test.cert .
# helm template . | sudo podman kube play --replace -
# helm template ./perfect-group-allocation | sudo podman kube play --replace -
# https://kubernetes.io/docs/tasks/run-application/run-single-instance-stateful-application/

rm -R tmp/kustomize
mkdir -p tmp/certs
mkdir -p tmp/kustomize
cp -r deployment/kustomize/* tmp/kustomize

# don't rotate ca cert so we can keep keycloak running
cd tmp/certs
export CAROOT=$PWD
CAROOT=$CAROOT mkcert -CAROOT
CAROOT=$CAROOT mkcert -install # to allow local testing
cd ../..

cd tmp/kustomize

cargo build --bin server
SERVER_BINARY=$(cargo build --bin server --message-format json | jq --raw-output 'select(.reason == "compiler-artifact" and .target.name == "server") | .executable')
echo "Compiled server binary: $SERVER_BINARY"

cargo build --test webdriver
INTEGRATION_TEST_BINARY=$(cargo build --test webdriver --message-format json | jq --raw-output 'select(.reason == "compiler-artifact" and .target.name == "webdriver") | .executable')
echo "Compiled integration test binary: $INTEGRATION_TEST_BINARY"

# we need to use rootful podman to get routable ip addresses.

# https://kubernetes.io/docs/tasks/run-application/run-single-instance-stateful-application/

# sudo podman inspect perfect-group-allocation-infra | jq ".[].NetworkSettings.Networks.[].IPAddress"
# dig tmp-perfect-group-allocation @10.89.1.1
# ping tmp-perfect-group-allocation

# maybe use some basic helm instead? I think all this patching is not nice
if [ "${1-}" == "keycloak" ]; then
    sudo podman build -t keycloak --file keycloak/keycloak/Dockerfile .
    sudo podman build -t perfect-group-allocation --file base/perfect-group-allocation/Dockerfile .
    sudo podman build -t test --file base/test/Dockerfile .

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

    (cd keycloak && CAROOT=$CAROOT mkcert tmp-keycloak)
    (cd keycloak && kustomize edit set nameprefix tmp-)
    (cd keycloak && kustomize edit add patch --patch '{"apiVersion": "v1","kind": "Pod","metadata":{"name":"keycloak"},"spec":{"volumes":[{"name":"root-ca","hostPath":{"path":"'"$CAROOT"'/rootCA.pem"}}]}}')
    (cd keycloak && kustomize build --output kubernetes.yaml)
    (cd keycloak && sudo podman kube down --force kubernetes.yaml || exit 0) # WARNING: this also removes volumes
    (cd keycloak && sudo podman kube play kubernetes.yaml)
    echo waiting for keycloak
    sudo podman wait --condition healthy tmp-keycloak-tmp-keycloak
    echo keycloak started
    # can we use import feature instead as this is super slow?
    sudo podman exec tmp-keycloak-tmp-keycloak keytool -noprompt -import -file /run/rootCA.pem -alias rootCA -storepass password -keystore /tmp/.keycloak-truststore.jks
    sudo podman exec tmp-keycloak-tmp-keycloak /opt/keycloak/bin/kcadm.sh config truststore --trustpass password /tmp/.keycloak-truststore.jks
    sudo podman exec tmp-keycloak-tmp-keycloak /opt/keycloak/bin/kcadm.sh config credentials --server https://tmp-keycloak --realm master --user admin --password admin
    sudo podman exec tmp-keycloak-tmp-keycloak /opt/keycloak/bin/kcadm.sh create realms -s realm=pga -s enabled=true
    sudo podman exec tmp-keycloak-tmp-keycloak /opt/keycloak/bin/kcadm.sh create users -r pga -s username=test -s email=test@example.com -s enabled=true
    sudo podman exec tmp-keycloak-tmp-keycloak /opt/keycloak/bin/kcadm.sh set-password -r pga --username test --new-password test
    sudo podman exec tmp-keycloak-tmp-keycloak /opt/keycloak/bin/kcadm.sh create clients -r pga -s clientId=pga -s secret=$(cat base/client-secret) -s 'redirectUris=["https://tmp-perfect-group-allocation/openidconnect-redirect"]'
fi

(cd base && CAROOT=$CAROOT mkcert tmp-perfect-group-allocation)
(cd base && kustomize edit set nameprefix tmp-)
(cd base && kustomize edit add patch --patch '{"apiVersion": "v1","kind": "Pod","metadata":{"name":"test"},"spec":{"volumes":[{"name":"test-binary","hostPath":{"path":"'"$INTEGRATION_TEST_BINARY"'"}}]}}')
(cd base && kustomize edit add patch --patch '{"apiVersion": "v1","kind": "Pod","metadata":{"name":"perfect-group-allocation"},"spec":{"volumes":[{"name":"root-ca","hostPath":{"path":"'"$CAROOT"'/rootCA.pem"}}]}}')
(cd base && kustomize edit add patch --patch '{"apiVersion": "v1","kind": "Pod","metadata":{"name":"perfect-group-allocation"},"spec":{"volumes":[{"name":"server-binary","hostPath":{"path":"'"$SERVER_BINARY"'"}}]}}')
(cd base && kustomize build --output kubernetes.yaml)
(cd base && sudo podman kube down --force kubernetes.yaml || exit 0) # WARNING: this also removes volumes
(cd base && sudo podman kube play kubernetes.yaml)
sudo podman logs --color --names --follow tmp-test-test tmp-perfect-group-allocation-tmp-perfect-group-allocation & # tmp-keycloak-tmp-keycloak tmp-postgres-postgres 
exit $(sudo podman wait tmp-test-test)
