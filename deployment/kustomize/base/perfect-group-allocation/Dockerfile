# podman build -t perfect-group-allocation --file deployment/kustomize/base/perfect-group-allocation/Dockerfile .
FROM docker.io/library/fedora:40

# use your user id and group id
ARG GROUP_ID=1000
ARG USER_ID=1000

RUN groupadd -g $GROUP_ID pga \
    && useradd -u $USER_ID -g $GROUP_ID -G audio,video -m pga

USER pga

ARG BINARY
COPY ${BINARY} /bin/server
ENTRYPOINT ["/bin/sleep" "infinity"]