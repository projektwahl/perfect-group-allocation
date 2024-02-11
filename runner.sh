#!/usr/bin/env bash

# TODO FIXME build the container for pga and test and put the binaries inside of the test container here? or somehow pass the images through

# FIXME path relative to cargo workspace root
BINARY=$(realpath --relative-to=.. $1)
echo $PWD
# maybe we can get the last line from this or so?
podman build --build-arg CARGO_MANIFEST_DIR=$PWD --build-arg BINARY=$BINARY --file ../deployment/kustomize/base/test/Dockerfile ..
IMAGE=$(podman build --quiet --build-arg CARGO_MANIFEST_DIR=$PWD --build-arg BINARY=$BINARY --file ../deployment/kustomize/base/test/Dockerfile ..)
echo aaa running $IMAGE
podman run --privileged --userns=keep-id --user=$(id -u):$(id -g) -v /home/moritz/.local/share/containers:/home/moritz/.local/share/containers --rm $IMAGE
