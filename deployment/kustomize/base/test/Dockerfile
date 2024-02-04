# to get latest firefox and chromium
FROM docker.io/library/debian:sid-slim

RUN apt-get update && apt-get install -y firefox chromium chromium-driver libpci3 p11-kit p11-kit-modules libnss3-tools mkcert

ENV WAYLAND_DISPLAY=/tmp/wayland-0
ENV XDG_RUNTIME_DIR=/tmp

# use your user id and group id
ARG GROUP_ID=1000
ARG USER_ID=1000

RUN groupadd -g $GROUP_ID test \
    && useradd -u $USER_ID -g $GROUP_ID -G audio,video -m test

RUN dpkg-divert --local --rename --divert /usr/lib.x86_64-linux-gnu.libnssckbi.so.diverted --add /usr/lib/x86_64-linux-gnu/libnssckbi.so
RUN ln -sr /usr/lib/x86_64-linux-gnu/pkcs11/p11-kit-trust.so /usr/lib/x86_64-linux-gnu/libnssckbi.so

USER test

RUN mkdir -p ~/.pki/nssdb
RUN certutil -d sql:$HOME/.pki/nssdb -N

ENTRYPOINT ["/bin/test"]
CMD ["--nocapture"]

# podman build -t test --file deployment/kustomize/base/test/Dockerfile . && podman run --device /dev/dri --userns=keep-id:uid=1000,gid=1000 --user=$(id -u):$(id -g) -e WAYLAND_DISPLAY=$WAYLAND_DISPLAY -v $XDG_RUNTIME_DIR/$WAYLAND_DISPLAY:/tmp/$WAYLAND_DISPLAY firefoxy firefox

# podman build -t test --file deployment/kustomize/base/test/Dockerfile . && podman run --device /dev/dri --userns=keep-id --user=$(id -u):$(id -g) -e WAYLAND_DISPLAY=$WAYLAND_DISPLAY -v $XDG_RUNTIME_DIR/$WAYLAND_DISPLAY:/tmp/$WAYLAND_DISPLAY firefoxy chromium --ozone-platform=wayland
# podman kube generate youthful_volhard