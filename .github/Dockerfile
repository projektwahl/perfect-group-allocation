# https://github.com/settings/tokens/new?scopes=write:packages
# echo $CR_PAT | podman login ghcr.io -u mohe2015 --password-stdin
# podman build -t ghcr.io/projektwahl/perfect-group-allocation:1 -f .github/Dockerfile .github
# podman push ghcr.io/projektwahl/perfect-group-allocation:1
# make package public

# todo switch to fedora because its more up-to-date (even the stable version)?
FROM registry.fedoraproject.org/fedora:rawhide

RUN rpm --import https://packages.microsoft.com/keys/microsoft.asc
RUN sh -c 'echo -e "[code]\nname=Visual Studio Code\nbaseurl=https://packages.microsoft.com/yumrepos/vscode\nenabled=1\ngpgcheck=1\ngpgkey=https://packages.microsoft.com/keys/microsoft.asc" > /etc/yum.repos.d/vscode.repo'

RUN dnf -y update
RUN dnf -y install firefox chromium wget code jq valgrind git cmake gcc zstd curl mold coin-or-Cbc-devel nss-tools podman openssl-devel pkg-config zlib-devel && dnf clean all

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

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain none -y
ENV PATH="/home/podman/.cargo/bin:${PATH}"
RUN rustup toolchain install stable --allow-downgrade --profile minimal --component rustfmt,clippy
RUN cargo install --locked cargo-hack
RUN cargo install --locked cargo-deny
RUN cargo install --locked --features=cli lightningcss@1.0.0-alpha.52
RUN cargo install --locked --version 0.10.2 iai-callgrind-runner

USER root
RUN mkdir /run/user/1000 && chmod 755 /run/user/1000 && chown podman:podman /run/user/1000
RUN dnf -y install nano
RUN dnf -y reinstall shadow-utils
RUN mkdir -p /home/podman/.local/share/containers && chmod -R 755 /home/podman/.local && chown -R podman:podman /home/podman/.local
USER podman
ENV XDG_RUNTIME_DIR=/run/user/1000
ENV WAYLAND_DISPLAY=wayland-0
