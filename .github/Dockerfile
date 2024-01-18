# https://github.com/settings/tokens/new?scopes=write:packages
# echo $CR_PAT | podman login ghcr.io -u mohe2015 --password-stdin
# podman build -t ghcr.io/projektwahl/perfect-group-allocation:latest .
# podman push ghcr.io/projektwahl/perfect-group-allocation:latest
# make package public
FROM debian

RUN apt-get update && apt-get install -y valgrind git cmake gcc zstd curl mold coinor-libcbc-dev
RUN curl -fsSL https://deb.nodesource.com/setup_21.x | bash -
RUN apt-get install -y nodejs
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain none -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup toolchain install nightly --allow-downgrade --profile minimal --component rustfmt,clippy
RUN cargo install --locked --version 0.10.0 iai-callgrind-runner
RUN cargo install --locked cargo-hack
RUN cargo install --locked cargo-deny
RUN cargo install lightningcss@1.0.0-alpha.52
RUN apt-get install -y mkcert
