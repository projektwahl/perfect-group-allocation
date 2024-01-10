# perfect-group-allocation

## Design goals

1. Secure
2. Efficient on mobile devices in regards to data comsumption and processing power
3. Low Latency
4. Low Resource usage on the server


## Interesting (unstable) features

https://doc.rust-lang.org/nightly/unstable-book/
https://doc.rust-lang.org/nightly/cargo/reference/unstable.html
https://doc.rust-lang.org/rustdoc/unstable-features.html
https://rust-lang.github.io/rfcs/3424-cargo-script.html
parallel rust frontend

rustup component add rustc-codegen-cranelift-preview --toolchain nightly

warning blocks in rustdoc
<div class="warning">A big warning!</div>

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
podman compose logs opentelemetry-collector

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

# errors
# http://localhost:16686/search?end=1704133081894000&limit=20&lookback=1h&maxDuration&minDuration&service=perfect-group-allocation&start=1704129481894000&tags=%7B%22error%22%3A%22true%22%7D

https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.TaskMonitor.html

# otel-v1-apm-span-*

psql postgres://postgres:password@localhost/pga?sslmode=disable
DATABASE_URL="postgres://postgres:password@localhost/pga?sslmode=disable" cargo run --release --bin server

DATABASE_URL="postgres://postgres:password@localhost/pga?sslmode=disable" OTEL_METRIC_EXPORT_INTERVAL=1000 RUST_BACKTRACE=1 cargo run --bin server
RUST_BACKTRACE=1 RUSTFLAGS="-Zthreads=8 -Zcodegen-backend=cranelift --cfg tokio_unstable" cargo run --bin server

tokio-console
```

## Async profiling

https://github.com/tokio-rs/console

cargo install --locked tokio-console

tokio-console

TODO https://docs.rs/tracing-timing/latest/tracing_timing/ per request

maybe https://github.com/tokio-rs/async-backtrace

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

28% http
router clone seems to do heap allocs

# https://github.com/launchbadge/sqlx/blob/929af41745a9434ae83417dcf2571685cecca6f0/sqlx-postgres/src/options/mod.rs#L15
# WARNING: Only connect without ssl over localhost. This makes the profiling better as there is not countless ssl stuff in there.
# I think you need to run this from the workspace root for debug symbols?
DATABASE_URL="postgres://postgres@localhost/pga?sslmode=disable" valgrind --tool=callgrind --cache-sim=yes --simulate-wb=yes --simulate-hwpref=yes --branch-sim=yes --dump-instr=yes --collect-jumps=yes --collect-bus=yes --collect-systime=nsec ./target/x86_64-unknown-linux-gnu/release-with-debug/server #  --cacheuse=yes

use zed attack proxy to create some requests

export DEBUGINFOD_URLS="https://debuginfod.archlinux.org"
# https://bugs.kde.org/show_bug.cgi?id=472973
kcachegrind callgrind.out.110536

55,300 requests
28% client hello
50% handlebars

debuginfod-find debuginfo /lib/libc.so.6
debuginfod-find source /lib/libc.so.6 /usr/src/debug/glibc/glibc/sysdeps/x86_64/multiarch/memmove-vec-unaligned-erms.S

TODO FIXME audit all database queries for race conditions
or use SERIALIZABLE I think

```
https://www.keycloak.org/getting-started/getting-started-podman
podman run -p 8080:8080 -e KEYCLOAK_ADMIN=admin -e KEYCLOAK_ADMIN_PASSWORD=admin quay.io/keycloak/keycloak:22.0.5 start-dev
podman start b217886c51eb

http://localhost:8080/realms/pga/account/

Add GitHub as identity provider for demo

Identity Providers -> Manage display order

https://lightningcss.dev/docs.html

# maybe create a local k3s in docker setup?
podman run --rm docker.io/envoyproxy/envoy:v1.27-latest --version # only use using k3s and cilium


https://github.com/djc/askama
https://github.com/rosetta-rs/template-benchmarks-rs


https://www.arewewebyet.org/topics/templating/
https://github.com/rosetta-rs/template-benchmarks-rs

(all excluded libraries don't do runtime templates)
https://github.com/cobalt-org/liquid-rust (not xss safe https://github.com/cobalt-org/liquid-rust/issues/68)
https://github.com/Keats/tera (not secure by default, wouldn't use)
https://github.com/maciejhirsz/ramhorns (has some benchmarks, seems to escape html, {{{ seems to not escape)

https://github.com/sunng87/handlebars-rust (slow?, really popular, seems to escape html, {{{ seems to not escape)


https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html#synchronizer-token-pattern


# I think http3 needs the low ports anyways
sudo caddy run --watch # does it send the Early-Data header?
https://caddyserver.com/docs/json/
https://caddy.community/t/how-to-use-dns-provider-modules-in-caddy-2/8148

https://github.com/abiosoft/caddy-json-schema

xcaddy build --with github.com/abiosoft/caddy-json-schema
~/Documents/xcaddy/caddy json-schema --vscode # only needed for the json schema


curl --header "Accept-Encoding: deflate" -O https://h3.selfmade4u.de:8443/download
curl --header "Accept-Encoding: gzip" -O https://h3.selfmade4u.de:8443/download
curl --header "Accept-Encoding: br" -O https://h3.selfmade4u.de:8443/download
curl --header "Accept-Encoding: zstd" -O https://h3.selfmade4u.de:8443/download

```

All streams need explicit error handling as the browser otherwise doesn't show anything

```
# TODO we should get a valid tls certificate from our domain as otherwise the browsers don't like you
sudo apt install lego
HETZNER_API_KEY=xxx lego --cert.timeout 180 --email Moritz.Hedtke@t-online.de --dns hetzner --domains h3.selfmade4u.de --accept-tos run
```

if using wsl in vscode, add port forward in vscode

https://h3.selfmade4u.de/

https://docs.rs/hyper/1.0.0-rc.4/hyper/index.html
https://hyper.rs/guides/1/
https://github.com/hyperium/hyper/blob/master/examples/README.md
https://blog.cloudflare.com/speeding-up-https-and-http-3-negotiation-with-dns/

https://docs.quic.tech/quiche/h3/index.html


https://interop.seemann.io/
ngtcp2 and picoquic best impls
https://interop.seemann.io/?client=ngtcp2,picoquic

network.http.http3.alt-svc-mapping-for-testing
h3.selfmade4u.de;h3=":443"

gh repo clone cloudflare/quiche
cd quiche
cargo run --bin quiche-client -- --dump-json https://h3.selfmade4u.de/
cargo run --bin quiche-client -- --dump-json https://127.0.0.1/
cargo run --bin quiche-client -- --dump-json https://[::1]/

# dns only 127.0.0.1 as chrome won't resolve link local addresses
cargo run --bin quiche-client -- --dump-json https://192.168.2.126/
cargo run --bin quiche-client -- --dump-json https://[fe80::acab:ec2e:86c3:1517]/

https://github.com/aws/s2n-quic/blob/main/quic/s2n-quic-qns/src/server/h3.rs

cargo build --release --bin server && sudo setcap CAP_NET_BIND_SERVICE+eip ./target/release/server && ./target/release/server

cargo build --release --bin server-http3 && sudo setcap CAP_NET_BIND_SERVICE+eip ./target/release/server-http3 && ./target/release/server-http3

cargo build --release --bin server-warp && sudo setcap CAP_NET_BIND_SERVICE+eip ./target/release/server-warp && ./target/release/server-warp


cargo build --release --bin server-http3-s2n && sudo setcap CAP_NET_BIND_SERVICE+eip ./target/release/server-http3-s2n && ./target/release/server-http3-s2n


cargo build --bin server && sudo setcap CAP_NET_BIND_SERVICE+eip ./target/debug/server && ./target/debug/server
cargo build --bin server-http3 && sudo setcap CAP_NET_BIND_SERVICE+eip ./target/debug/server-http3 && ./target/debug/server-http3



chromium-browser --enable-quic --origin-to-force-quic-on=localhost:443
```bash
mkdir -p .lego/certificates

openssl req -x509 -nodes -newkey ec -pkeyopt ec_paramgen_curve:secp384r1 -keyout .lego/certificates/h3.selfmade4u.de.key -out .lego/certificates/h3.selfmade4u.de.crt -days 30  -subj "/CN=example.com"

openssl req -x509 -newkey rsa:4096 -sha256 -days 3650 -nodes -keyout .lego/certificates/h3.selfmade4u.de.key -out .lego/certificates/h3.selfmade4u.de.crt -subj "/CN=example.com" -addext "subjectAltName=DNS:example.com,DNS:*.example.com,IP:10.0.0.1"


openssl rsa -inform pem -in example.com.key.pem -outform der -out example.com.key.der
openssl x509 -outform der -in example.com.cert.pem -out example.com.cert.der

cargo run --bin server -- --cert example.com.crt --key example.com.key.der
```

```bash
podman run --rm --detach --name postgres --volume pga-postgres:/var/lib/postgresql/data --env POSTGRES_PASSWORD=password docker.io/postgres
# --expose, --publish

podman run --rm --detach --name mariadb --volume pga-mariadb:/var/lib/mysql --env MARIADB_ROOT_PASSWORD=password docker.io/mariadb
```


https://vitess.io/docs/15.0/get-started/local-docker/