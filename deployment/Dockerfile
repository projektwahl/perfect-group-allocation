FROM docker.io/library/fedora:40
WORKDIR /usr/src/myapp

RUN dnf -y update && dnf -y install rust cmake coinor-libcbc-dev && dnf clean all
COPY . .
RUN cargo install --path perfect-group-allocation-backend

FROM docker.io/library/fedora:40
RUN dnf -y update && dnf -y install curl coinor-libcbc && dnf clean all
COPY --from=builder /usr/local/cargo/bin/server /usr/local/bin/server
CMD ["myapp"]