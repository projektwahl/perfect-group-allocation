# perfect-group-allocation

## Design goals

1. Secure
2. Efficient on mobile devices in regards to data comsumption and processing power
3. Low Latency
4. Low Resource usage on the server

## Testing

```bash
podman run --rm --name postgres-testing --env POSTGRES_PASSWORD=password --publish 5431:5432 docker.io/postgres
cargo test
```

## Dev

```bash
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

DATABASE_URL="postgres://postgres:password@localhost/pga?sslmode=disable" OTEL_METRIC_EXPORT_INTERVAL=1000 RUST_BACKTRACE=1 cargo run --bin server
RUST_BACKTRACE=1 RUSTFLAGS="-Zthreads=8 -Zcodegen-backend=cranelift --cfg tokio_unstable" cargo run --bin server

```

## Profiling

https://valgrind.org/docs/manual/cl-manual.html
Callgrind

# DO NOT USE TRUST AUTHENTICATION IN PRODUCTION! For profiling we don't want to measure sha2 hashing overhead
podman run --rm --detach --name postgres-profiling --env POSTGRES_HOST_AUTH_METHOD=trust --publish 5432:5432 docker.io/postgres

export DATABASE_URL="postgres://postgres@localhost/pga?sslmode=disable"
diesel database reset

https://nnethercote.github.io/perf-book/profiling.html

# rebuild std to get debug symbols and same settings?
cargo build --target=x86_64-unknown-linux-gnu -Z build-std --profile=release-with-debug --bin server

# WARNING: Only connect without ssl over localhost. This makes the profiling better as there is not countless ssl stuff in there.
# I think you need to run this from the workspace root for debug symbols?
DATABASE_URL="postgres://postgres@localhost/pga?sslmode=disable" valgrind --tool=callgrind --cache-sim=yes --simulate-wb=yes --simulate-hwpref=yes --branch-sim=yes --dump-instr=yes --collect-jumps=yes --collect-bus=yes --collect-systime=nsec ./target/x86_64-unknown-linux-gnu/release-with-debug/server #  --cacheuse=yes

use zed attack proxy to create some requests

export DEBUGINFOD_URLS="https://debuginfod.archlinux.org"
kcachegrind callgrind.out.110536
```
https://www.keycloak.org/getting-started/getting-started-podman
podman run -p 8080:8080 -e KEYCLOAK_ADMIN=admin -e KEYCLOAK_ADMIN_PASSWORD=admin quay.io/keycloak/keycloak:22.0.5 start-dev
podman start b217886c51eb

http://localhost:8080/realms/pga/account/

Add GitHub as identity provider for demo

Identity Providers -> Manage display order
