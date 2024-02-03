# to get latest firefox and chromium
FROM docker.io/library/debian:sid-slim

# use your user id and group id
ARG GROUP_ID=1000
ARG USER_ID=1000

RUN apt-get update && apt-get install -y firefox chromium libpci3

RUN groupadd -g $GROUP_ID ff \
    && useradd -u $USER_ID -g $GROUP_ID -G audio,video -m ff

ENV WAYLAND_DISPLAY=/tmp/wayland-0
ENV XDG_RUNTIME_DIR=/tmp

USER ff

# cargo build --test webdriver --message-format json | jq --raw-output 'select(.reason == "compiler-artifact" and .target.name == "webdriver") | .executable'
COPY target/debug/deps/webdriver-* /bin/

CMD ["/bin/webdriver"]

# podman build -t webdriver --file deployment/kustomize/base/webdriver/Dockerfile . && podman run --device /dev/dri --userns=keep-id:uid=1000,gid=1000 --user=$(id -u):$(id -g) -e WAYLAND_DISPLAY=$WAYLAND_DISPLAY -v $XDG_RUNTIME_DIR/$WAYLAND_DISPLAY:/tmp/$WAYLAND_DISPLAY firefoxy firefox

# podman build -t webdriver --file deployment/kustomize/base/webdriver/Dockerfile . && podman run --device /dev/dri --userns=keep-id --user=$(id -u):$(id -g) -e WAYLAND_DISPLAY=$WAYLAND_DISPLAY -v $XDG_RUNTIME_DIR/$WAYLAND_DISPLAY:/tmp/$WAYLAND_DISPLAY firefoxy chromium --ozone-platform=wayland
# podman kube generate youthful_volhard