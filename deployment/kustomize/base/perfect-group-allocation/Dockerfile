# podman build -t application --file deployment/kustomize/base/application/Dockerfile .
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

COPY target/debug/server /bin/server

EXPOSE 443
CMD ["/bin/server"]