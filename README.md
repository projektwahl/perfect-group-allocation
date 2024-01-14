# perfect-group-allocation

## Design goals

1. Secure
2. Efficient on mobile devices in regards to data comsumption and processing power
3. Low Latency
4. Low Resource usage on the server

## Updating dependencies

```
cargo install cargo-edit
cargo upgrade --verbose --incompatible allow --pinned allow
```

## Keycloak

http://localhost:8080/admin/master/console/
admin
admin

https://www.keycloak.org/docs/23.0.4/server_admin/#configuring-realms

Create Realm "pga"
Import file from deployment/pga.json

http://localhost:8080/admin/master/console/#/pga/realm-settings/localization

Internationalization -> Deutsch

https://www.keycloak.org/docs/23.0.4/server_admin/#assembly-managing-users_server_administration_guide

Create test user, add password

https://www.keycloak.org/docs/23.0.4/server_admin/#con-user-impersonation_server_administration_guide

Impersonate user for testing

https://www.keycloak.org/docs/23.0.4/server_admin/#_identity_broker

https://www.keycloak.org/docs/23.0.4/server_admin/#_client_suggested_idp

https://www.keycloak.org/docs/23.0.4/securing_apps/#_java_adapter_logout

https://www.keycloak.org/docs/23.0.4/server_admin/#sso-protocols

https://www.keycloak.org/docs/23.0.4/server_admin/#_oidc-logout

https://www.keycloak.org/docs/23.0.4/server_admin/#assembly-managing-clients_server_administration_guide

Create an OpenID client

Clients -> Create Client -> ...

Client Authentication On

Only enable Standard Flow

Valid redirect urls:
https://h3.selfmade4u.de

https://www.keycloak.org/docs/23.0.4/server_admin/#configuring-auditing-to-track-events

https://www.keycloak.org/docs/23.0.4/server_admin/#auditing-admin-events

CRITIAL SECURITY NOTES:

https://www.keycloak.org/docs/23.0.4/server_admin/#host

https://www.keycloak.org/docs/23.0.4/server_admin/#admin-cli

http://localhost:8080/realms/pga/account/

/realms/{realm-name}/.well-known/openid-configuration

Add GitHub as identity provider for demo

Identity Providers -> Manage display order


## Testing

```bash
podman run --rm --name postgres-testing --env POSTGRES_PASSWORD=password --publish 5431:5432 docker.io/postgres
cargo test
```

## Dev

```bash
cargo +stable install cargo-hack --locked

mkcert -install
mkcert h3.selfmade4u.de

# for chrome and h3 you need to listen on a port < 1024 AND you need a certificate with a public root
HETZNER_API_KEY=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx lego --email Moritz.Hedtke@t-online.de --dns hetzner --domains h3.selfmade4u.de run
export DATABASE_URL="postgres://postgres@localhost/pga?sslmode=disable"
cargo build --bin server && sudo setcap 'cap_net_bind_service=+ep' target/debug/server && ./target/debug/server
SSLKEYLOGFILE=/tmp/sslkeylogfile.txt firefox

sudo nano /etc/sysctl.conf
vm.max_map_count=262144
sudo sysctl -p

pipx install https://github.com/containers/podman-compose/archive/devel.tar.gz # profile support not yet in 1.0.6
clear && podman compose down && podman compose up
# clear && podman compose --profile opensearch up

# jaeger http://localhost:16686
# opensearch http://localhost:5601
# prometheus http://localhost:9090
# grafana http://localhost:3001
# add prometheus source to grafana: http://prometheus:9090, SET INTERVAL TO THE SAME AS OTEL_METRIC_EXPORT_INTERVAL in seconds

# https://github.com/google/re2/wiki/Syntax
# {__name__=~"tokio_runtime_metrics_.*",__name__!~"tokio_runtime_metrics_.*_nanoseconds"}
# {__name__=~"tokio_runtime_metrics_.*_nanoseconds"}

# {__name__=~"tokio_task_metrics_.*_nanoseconds"}
# {__name__=~"tokio_task_metrics_.*",__name__!~"tokio_task_metrics_.*_nanoseconds"}

# Grafana: Export for sharing externally

# otel-v1-apm-span-*

psql postgres://postgres:password@localhost/pga?sslmode=disable
DATABASE_URL="postgres://postgres:password@localhost/pga?sslmode=disable" cargo run --release --bin server

```

## Profiling

https://valgrind.org/docs/manual/cl-manual.html
Callgrind

# DO NOT USE TRUST AUTHENTICATION IN PRODUCTION! For profiling we don't want to measure sha2 hashing overhead
podman run --rm --detach --name postgres-profiling --env POSTGRES_HOST_AUTH_METHOD=trust --publish 5432:5432 docker.io/postgres

export DATABASE_URL="postgres://postgres@localhost/pga?sslmode=disable"
diesel database reset

https://nnethercote.github.io/perf-book/profiling.html

cargo build --features profiling --target=x86_64-unknown-linux-gnu -Z build-std --profile=release-with-debug --bin server

# I think macros don't work well with this especially the select! macro

# WARNING: Only connect without ssl over localhost. This makes the profiling better as there is not countless ssl stuff in there.
# I think you need to run this from the workspace root for debug symbols?
DATABASE_URL="postgres://postgres@localhost/pga?sslmode=disable" valgrind --trace-symtab=yes --tool=callgrind --cache-sim=yes --simulate-wb=yes --simulate-hwpref=yes --branch-sim=yes --dump-instr=yes --collect-jumps=yes --collect-bus=yes --collect-systime=nsec ./target/x86_64-unknown-linux-gnu/debug/server

use zed attack proxy to create some requests

export DEBUGINFOD_URLS="https://debuginfod.archlinux.org"
kcachegrind callgrind.out.110536
