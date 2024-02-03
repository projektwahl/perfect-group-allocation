# podman build -t perfect-group-allocation --file deployment/kustomize/base/perfect-group-allocation/Dockerfile .
FROM docker.io/library/debian:stable-slim

RUN apt-get update && apt install -y curl

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

ENTRYPOINT ["/bin/server"]