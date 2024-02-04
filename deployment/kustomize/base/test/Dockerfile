# to get latest firefox and chromium
FROM docker.io/library/debian:sid-slim

RUN apt-get update && apt-get install -y firefox libpci3 chromium chromium-driver

ENV WAYLAND_DISPLAY=/tmp/wayland-0
ENV XDG_RUNTIME_DIR=/run/user/1000

# use your user id and group id
ARG GROUP_ID=1000
ARG USER_ID=1000

RUN groupadd -g $GROUP_ID test \
    && useradd -u $USER_ID -g $GROUP_ID -G audio,video -m test

RUN mkdir /run/user/1000 && chmod 755 /run/user/1000 && chown test:test /run/user/1000

USER test

ENTRYPOINT ["firefox"]

# podman build -t test --file deployment/kustomize/base/test/Dockerfile . && podman run --device /dev/dri --userns=keep-id:uid=1000,gid=1000 --user=$(id -u):$(id -g) -e WAYLAND_DISPLAY=$WAYLAND_DISPLAY -v $XDG_RUNTIME_DIR/$WAYLAND_DISPLAY:$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY test firefox

# podman build -t test --file deployment/kustomize/base/test/Dockerfile . && podman run --device /dev/dri --userns=keep-id --user=$(id -u):$(id -g) -e WAYLAND_DISPLAY=$WAYLAND_DISPLAY -v $XDG_RUNTIME_DIR/$WAYLAND_DISPLAY:$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY test chromium --ozone-platform=wayland
# podman kube generate youthful_volhard