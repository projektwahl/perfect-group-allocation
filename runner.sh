#!/usr/bin/env bash

# FIXME path relative to cargo workspace root
BINARY=$(realpath --relative-to=.. $1)
echo $PWD
# maybe we can get the last line from this or so?
sudo podman build --build-arg CARGO_MANIFEST_DIR=$PWD --build-arg BINARY=$BINARY --file ../deployment/kustomize/base/test/Dockerfile ..
IMAGE=$(sudo podman build --quiet --build-arg CARGO_MANIFEST_DIR=$PWD --build-arg BINARY=$BINARY --file ../deployment/kustomize/base/test/Dockerfile ..)
echo running $IMAGE
sudo podman run --privileged -it --rm $IMAGE
