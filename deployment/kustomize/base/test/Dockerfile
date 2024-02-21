# to get latest firefox and chromium
FROM docker.io/library/debian:sid

RUN apt-get update && apt-get install -y firefox libpci3 chromium chromium-driver

RUN useradd -m podman

RUN mkdir /run/user/1000 && chmod 755 /run/user/1000 && chown podman:podman /run/user/1000

USER podman

ENV WAYLAND_DISPLAY=wayland-0
ENV XDG_RUNTIME_DIR=/run/user/1000

ARG BINARY
COPY ${BINARY} /bin/test

ARG EXECUTABLE
ENV EXECUTABLE=$EXECUTABLE
ENTRYPOINT $EXECUTABLE

# podman build -t test --file deployment/kustomize/base/test/Dockerfile . && podman run --device /dev/dri --userns=keep-id --user=$(id -u):$(id -g) -v /run/user/1000/wayland-0:/run/user/1000/wayland-0 test firefox
# podman build -t test --file deployment/kustomize/base/test/Dockerfile . && podman run --device /dev/dri --userns=keep-id --user=$(id -u):$(id -g) -e WAYLAND_DISPLAY=$WAYLAND_DISPLAY -v $XDG_RUNTIME_DIR/$WAYLAND_DISPLAY:$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY test chromium --ozone-platform=wayland