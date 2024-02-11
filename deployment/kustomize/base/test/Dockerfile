# to get latest firefox and chromium
FROM docker.io/library/debian:sid

RUN apt-get update && apt-get install -y firefox libpci3 chromium chromium-driver curl mkcert sudo podman

# use your user id and group id
ARG GROUP_ID=1000
ARG USER_ID=1000

RUN groupadd -g $GROUP_ID test \
    && useradd -u $USER_ID -g $GROUP_ID -G audio,video,sudo -m test

RUN echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers

RUN mkdir /run/user/1000 && chmod 755 /run/user/1000 && chown test:test /run/user/1000

ARG CARGO_MANIFEST_DIR
RUN mkdir -p ${CARGO_MANIFEST_DIR}
COPY ./deployment /${CARGO_MANIFEST_DIR}/../deployment
COPY ./run-integration-tests.sh /${CARGO_MANIFEST_DIR}/../
# FIXME do this as late as possible
COPY ./tmp /${CARGO_MANIFEST_DIR}/../tmp
RUN chown -R test:test ${CARGO_MANIFEST_DIR}/../

WORKDIR /bin
RUN curl -OL "https://github.com/kubernetes-sigs/kustomize/releases/download/kustomize%2Fv5.3.0/kustomize_v5.3.0_linux_amd64.tar.gz"
RUN tar xzf kustomize_v5.3.0_linux_amd64.tar.gz

USER test
RUN groups

ENV WAYLAND_DISPLAY=wayland-0
ENV XDG_RUNTIME_DIR=/run/user/1000

ARG BINARY
COPY ${BINARY} /bin/test
WORKDIR ${CARGO_MANIFEST_DIR}
ENTRYPOINT ["/bin/test"]
CMD ["--nocapture"]

# podman build -t test --file deployment/kustomize/base/test/Dockerfile . && podman run --device /dev/dri --userns=keep-id --user=$(id -u):$(id -g) -v /run/user/1000/wayland-0:/run/user/1000/wayland-0 test firefox

# podman build -t test --file deployment/kustomize/base/test/Dockerfile . && podman run --device /dev/dri --userns=keep-id --user=$(id -u):$(id -g) -e WAYLAND_DISPLAY=$WAYLAND_DISPLAY -v $XDG_RUNTIME_DIR/$WAYLAND_DISPLAY:$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY test chromium --ozone-platform=wayland
# podman kube generate youthful_volhard