# https://github.com/settings/tokens/new?scopes=write:packages
# echo $CR_PAT | podman login ghcr.io -u mohe2015 --password-stdin
# podman build -t ghcr.io/projektwahl/perfect-group-allocation:1 -f .github/Dockerfile .github
# podman push ghcr.io/projektwahl/perfect-group-allocation:1
# make package public

# podman 5
FROM docker.io/library/fedora:40

RUN rpm --import https://packages.microsoft.com/keys/microsoft.asc
RUN sh -c 'echo -e "[code]\nname=Visual Studio Code\nbaseurl=https://packages.microsoft.com/yumrepos/vscode\nenabled=1\ngpgcheck=1\ngpgkey=https://packages.microsoft.com/keys/microsoft.asc" > /etc/yum.repos.d/vscode.repo'

RUN dnf -y update && dnf -y install rust cargo clippy rust-analyzer rust-debugger-common rust-doc rust-gdb rust-src rustfmt firefox chromium wget code jq valgrind git cmake gcc zstd curl mold coin-or-Cbc-devel nss-tools podman openssl-devel pkg-config zlib-devel libpq-devel nano && dnf -y reinstall shadow-utils && dnf clean all

RUN curl -JLO "https://dl.filippo.io/mkcert/latest?for=linux/amd64"
RUN chmod +x mkcert-v*-linux-amd64
RUN cp mkcert-v*-linux-amd64 /usr/local/bin/mkcert

RUN curl -OL "https://github.com/kubernetes-sigs/kustomize/releases/download/kustomize%2Fv5.3.0/kustomize_v5.3.0_linux_amd64.tar.gz"
RUN tar -zxf kustomize_v5.3.0_linux_amd64.tar.gz --directory /usr/local/bin

# useradd creates a default subuid/gid range
# inner podman build commands will try to use the full subuid/gid range and if we don't pass this range from outside it breaks.
# also we need at least 65536 to make apt work inside of a podman build inside of this container
# the defaults should be large enough
RUN useradd -s /bin/bash -m podman
USER podman

RUN cargo install cargo-hack
RUN cargo install cargo-deny
RUN cargo install --features=cli lightningcss@1.0.0-alpha.52
RUN cargo install --version 0.10.2 iai-callgrind-runner
RUN cargo install diesel_cli --no-default-features --features postgres
RUN cargo install cargo-watch

USER root
RUN mkdir /run/user/1000 && chmod 755 /run/user/1000 && chown podman:podman /run/user/1000
RUN mkdir -p /home/podman/.local/share/containers && chmod -R 755 /home/podman/.local && chown -R podman:podman /home/podman/.local
USER podman
ENV XDG_RUNTIME_DIR=/run/user/1000
ENV WAYLAND_DISPLAY=wayland-0
ENV EDITOR=nano
