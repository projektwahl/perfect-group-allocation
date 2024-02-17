#!/usr/bin/env bash
set -o errexit   # abort on nonzero exitstatus
set -o nounset   # abort on unbound variable
set -o pipefail  # don't hide errors within pipes
set -x

BINARY=$(realpath --relative-to=. $1)

CONTAINERIGNORE=$(mktemp)
echo -e '*\n!'"$BINARY" > $CONTAINERIGNORE
cat $CONTAINERIGNORE

IMAGE=$(podman build --ignorefile $CONTAINERIGNORE --build-arg BINARY=$BINARY --file ./deployment/kustomize/base/test/Dockerfile .)
podman run --network podman-default-kube-network --device /dev/dri -v /run/user/1000/wayland-0:/run/user/1000/wayland-0 --privileged --userns=keep-id --user=$(id -u):$(id -g) --rm $(echo "$IMAGE" | tail -n 1)
