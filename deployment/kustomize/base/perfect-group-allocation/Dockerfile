# podman build -t perfect-group-allocation --file deployment/kustomize/base/perfect-group-allocation/Dockerfile .
FROM docker.io/library/debian:stable-slim

RUN apt-get update && apt install -y curl

# use your user id and group id
ARG GROUP_ID=1000
ARG USER_ID=1000

RUN groupadd -g $GROUP_ID test \
    && useradd -u $USER_ID -g $GROUP_ID -G audio,video -m test

USER test

ENTRYPOINT ["/bin/server"]