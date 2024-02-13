#!/usr/bin/env bash
set -o errexit   # abort on nonzero exitstatus
set -o nounset   # abort on unbound variable
set -o pipefail  # don't hide errors within pipes
set -x

BINARY=$(realpath --relative-to=.. $1)
IMAGE=$(podman build --quiet --build-arg CARGO_MANIFEST_DIR=$PWD --build-arg BINARY=$BINARY --file ../deployment/kustomize/base/test/Dockerfile ..)
podman run --network podman-default-kube-network -v /run/user/1000/podman/podman.sock:/run/user/1000/podman/podman.sock --device /dev/dri -v /run/user/1000/wayland-0:/run/user/1000/wayland-0 --privileged --userns=keep-id --user=$(id -u):$(id -g) --rm $IMAGE
