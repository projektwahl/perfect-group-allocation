# podman build --file deployment/kustomize/base/application/Dockerfile .
FROM docker.io/library/debian:stable-slim

ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

COPY pga.toml /etc/pga.toml
COPY target/debug/server /bin/server

EXPOSE 3000
WORKDIR /etc
CMD ["/bin/server"]