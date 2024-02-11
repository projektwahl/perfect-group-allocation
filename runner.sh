#!/usr/bin/env bash

# FIXME path relative to cargo workspace root
BINARY=$(realpath --relative-to=.. $1)
echo $PWD
# maybe we can get the last line from this or so?
podman build --build-arg CARGO_MANIFEST_DIR=$PWD --build-arg BINARY=$BINARY --file ../deployment/kustomize/base/test/Dockerfile ..
IMAGE=$(podman build --quiet --build-arg CARGO_MANIFEST_DIR=$PWD --build-arg BINARY=$BINARY --file ../deployment/kustomize/base/test/Dockerfile ..)
echo running $IMAGE
podman run --device /dev/dri --userns=keep-id --user=$(id -u):$(id -g) -v /run/user/1000/wayland-0:/run/user/1000/wayland-0 $IMAGE
