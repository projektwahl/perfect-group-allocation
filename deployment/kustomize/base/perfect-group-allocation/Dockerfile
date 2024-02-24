# podman build -t perfect-group-allocation --file deployment/kustomize/base/perfect-group-allocation/Dockerfile .
FROM docker.io/library/fedora:40

RUN dnf -y update && dnf -y install curl && dnf clean all

# use your user id and group id
ARG GROUP_ID=1000
ARG USER_ID=1000

RUN groupadd -g $GROUP_ID test \
    && useradd -u $USER_ID -g $GROUP_ID -G audio,video -m test

USER test

ARG BINARY
COPY ${BINARY} /bin/server
ENTRYPOINT ["/bin/server"]