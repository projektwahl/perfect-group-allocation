#!/usr/bin/env bash

# FIXME path relative to cargo workspace root
BINARY=$(realpath --relative-to=.. $1)
echo $PWD
# maybe we can get the last line from this or so?
podman build --build-arg CARGO_MANIFEST_DIR=$PWD --build-arg BINARY=$BINARY --file ../deployment/kustomize/base/test/Dockerfile ..
IMAGE=$(podman build --quiet --build-arg CARGO_MANIFEST_DIR=$PWD --build-arg BINARY=$BINARY --file ../deployment/kustomize/base/test/Dockerfile ..)
echo aaa running $IMAGE
podman run --rm $IMAGE
