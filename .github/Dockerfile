# https://github.com/settings/tokens/new?scopes=write:packages
# echo $CR_PAT | podman login ghcr.io -u mohe2015 --password-stdin
# podman build -t ghcr.io/projektwahl/perfect-group-allocation:1 -f .github/Dockerfile .github
# podman push ghcr.io/projektwahl/perfect-group-allocation:1
# make package public

# todo switch to fedora because its more up-to-date (even the stable version)?
FROM docker.io/library/debian:sid

RUN apt-get update && apt-get -y upgrade && apt-get install -y wget gpg curl

RUN wget -qO- https://packages.microsoft.com/keys/microsoft.asc | gpg --dearmor > packages.microsoft.gpg
RUN install -D -o root -g root -m 644 packages.microsoft.gpg /etc/apt/keyrings/packages.microsoft.gpg
RUN echo "deb [arch=amd64,arm64,armhf signed-by=/etc/apt/keyrings/packages.microsoft.gpg] https://packages.microsoft.com/repos/code stable main" > /etc/apt/sources.list.d/vscode.list
RUN rm -f packages.microsoft.gpg

RUN curl -fsSL https://deb.nodesource.com/setup_21.x | bash -

RUN apt-get update && apt-get -y upgrade && apt-get install -y nodejs code apt-transport-https wget gpg jq valgrind git cmake gcc zstd curl mold coinor-libcbc-dev mkcert podman libssl-dev pkg-config zlib1g-dev

RUN curl -OL "https://github.com/kubernetes-sigs/kustomize/releases/download/kustomize%2Fv5.3.0/kustomize_v5.3.0_linux_amd64.tar.gz"
RUN tar -zxf kustomize_v5.3.0_linux_amd64.tar.gz --directory /usr/local/bin

# useradd creates a default subuid/gid range
# inner podman build commands will try to use the full subuid/gid range and if we don't pass this range from outside it breaks.
# also we need at least 65536 to make apt work inside of a podman build inside of this container
# the defaults should be large enough
RUN useradd -m podman
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
USER podman
ENV XDG_RUNTIME_DIR=/run/user/1000
