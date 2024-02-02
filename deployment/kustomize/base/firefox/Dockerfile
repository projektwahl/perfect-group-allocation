FROM docker.io/library/debian:sid

# use your user id and group id
ARG GROUP_ID=1000
ARG USER_ID=1000

RUN apt-get update && apt-get install -y firefox chromium libpci3

RUN groupadd -g $GROUP_ID ff \
    && useradd -u $USER_ID -g $GROUP_ID -G audio,video -m ff

ENV WAYLAND_DISPLAY=/tmp/wayland-0
#ENV DBUS_SESSION_BUS_ADDRESS="unix:path=/tmp/bus"

USER ff

# podman build -t firefoxy . && podman run --user=$(id -u):$(id -g) --security-opt label=type:container_runtime_ts --userns=keep-id --device /dev/dri --volume "$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY":/tmp/wayland-0:Z --rm -it firefoxy firefox
# podman run -it --userns=keep-id --user=$(id -u):$(id -g) -e XDG_RUNTIME_DIR=/tmp -e WAYLAND_DISPLAY=$WAYLAND_DISPLAY -v $XDG_RUNTIME_DIR/$WAYLAND_DISPLAY:/tmp/$WAYLAND_DISPLAY firefoxy
