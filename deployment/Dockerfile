FROM docker.io/rustlang/rust:bookworm-slim AS builder
WORKDIR /usr/src/myapp

RUN apt-get update && apt-get install -y cmake coinor-libcbc-dev
COPY . .
RUN cargo install --path perfect-group-allocation-backend

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y coinor-libcbc && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/server /usr/local/bin/server
CMD ["myapp"]