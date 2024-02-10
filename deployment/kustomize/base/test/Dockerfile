# to get latest firefox and chromium
FROM docker.io/library/debian:sid-slim

ARG BINARY

RUN apt-get update && apt-get install -y firefox libpci3 chromium chromium-driver

# use your user id and group id
ARG GROUP_ID=1000
ARG USER_ID=1000

RUN groupadd -g $GROUP_ID test \
    && useradd -u $USER_ID -g $GROUP_ID -G audio,video -m test

RUN mkdir /run/user/1000 && chmod 755 /run/user/1000 && chown test:test /run/user/1000

USER test

ENV WAYLAND_DISPLAY=wayland-0
ENV XDG_RUNTIME_DIR=/run/user/1000

COPY ${BINARY} /bin/test
ENTRYPOINT ["/bin/test"]
CMD ["--nocapture"]

# podman build -t test --file deployment/kustomize/base/test/Dockerfile . && podman run --device /dev/dri --userns=keep-id --user=$(id -u):$(id -g) -v /run/user/1000/wayland-0:/run/user/1000/wayland-0 test firefox

# podman build -t test --file deployment/kustomize/base/test/Dockerfile . && podman run --device /dev/dri --userns=keep-id --user=$(id -u):$(id -g) -e WAYLAND_DISPLAY=$WAYLAND_DISPLAY -v $XDG_RUNTIME_DIR/$WAYLAND_DISPLAY:$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY test chromium --ozone-platform=wayland
# podman kube generate youthful_volhard