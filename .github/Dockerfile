# https://github.com/settings/tokens/new?scopes=write:packages
# echo $CR_PAT | podman login ghcr.io -u mohe2015 --password-stdin
# podman build -t ghcr.io/projektwahl/perfect-group-allocation:1 -f .github/Dockerfile .github
# podman push ghcr.io/projektwahl/perfect-group-allocation:1
# make package public
FROM docker.io/library/debian:sid

RUN apt-get update && apt-get -y upgrade && apt-get install -y jq valgrind git cmake gcc zstd curl mold coinor-libcbc-dev mkcert podman libssl-dev pkg-config zlib1g-dev
RUN curl -fsSL https://deb.nodesource.com/setup_21.x | bash -
RUN apt-get install -y nodejs
RUN curl -OL "https://github.com/kubernetes-sigs/kustomize/releases/download/kustomize%2Fv5.3.0/kustomize_v5.3.0_linux_amd64.tar.gz"
RUN tar -zxf kustomize_v5.3.0_linux_amd64.tar.gz --directory /usr/local/bin

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
RUN echo podman:165536:65536 > /etc/subuid
RUN echo podman:165536:65536 > /etc/subgid
RUN mkdir /run/user/1000 && chmod 755 /run/user/1000 && chown podman:podman /run/user/1000
USER podman
ENV XDG_RUNTIME_DIR=/run/user/1000
