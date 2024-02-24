# perfect-group-allocation

## Development

The only supported way is to develop within a container. This reduces host differences.
```bash
eval $(ssh-agent)
ssh-add ~/.ssh/id_ed25519
podman run --rm -it --device /dev/dri --privileged -v ~/.gitconfig:/home/podman/.gitconfig -v $SSH_AUTH_SOCK:$SSH_AUTH_SOCK -e SSH_AUTH_SOCK=$SSH_AUTH_SOCK -v $XDG_RUNTIME_DIR/$WAYLAND_DISPLAY:/run/user/1000/wayland-0 --shm-size=1g --uidmap 1000:0:1 --uidmap 0:1:1000 --uidmap 1001:1001:64535 --uidmap 524288:65537:65536 -v pga-podman-cache:/home/podman/.local/share/containers:Z -v pga-cargo:/home/podman/.cargo -v pga-target:$PWD/target -v $PWD:$PWD --workdir=$PWD ghcr.io/projektwahl/perfect-group-allocation:1 bash
code --ozone-platform=wayland . # --verbose

# inside vscode terminal
./run-integration-tests.sh keycloak
PREFIX=dev ./run-integration-tests.sh backend-db-and-test $PWD/LICENSE /usr/bin/firefox
PREFIX=dev ./run-integration-tests.sh backend
https://devperfect-group-allocation.dns.podman
```

## Design goals

1. Secure
2. Efficient on mobile devices in regards to data comsumption and processing power
3. Low Latency
4. Low Resource usage on the server

## Development Notes

```
cargo local-registry --sync Cargo.lock registry
cargo lightningcss --bundle --minify --sourcemap --output-file frontend/bundle.css frontend/index.css
```

`http_body::Body` should always have a `+ 'static` annotation to avoid errors occuring at the wrong place.

## CI

Using Forgejo Actions
```
systemctl --user enable --now podman

# you need to enable actions on the repository and then add it to the repository itself
podman run --userns=keep-id --env DOCKER_HOST="unix://$XDG_RUNTIME_DIR/podman/podman.sock" -v $XDG_RUNTIME_DIR/podman/podman.sock:$XDG_RUNTIME_DIR/podman/podman.sock --name forgejo --rm code.forgejo.org/forgejo/runner:3.3.0 bash -c "forgejo-runner register --no-interactive --token XXX --name runner --instance https://codeberg.org && forgejo-runner daemon"

podman exec forgejo forgejo-runner cache-server

# broken
podman run --userns=keep-id --env DOCKER_HOST="unix://$XDG_RUNTIME_DIR/podman/podman.sock" -v $XDG_RUNTIME_DIR/podman/podman.sock:$XDG_RUNTIME_DIR/podman/podman.sock -v .:/data --rm code.forgejo.org/forgejo/runner:3.3.0 forgejo-runner exec

```

## Updating dependencies

```
cargo install --locked cargo-edit
cargo upgrade --verbose --incompatible allow --pinned allow
```

## Keycloak

https://www.keycloak.org/docs/latest/server_admin/index.html#admin-cli

podman exec -it perfect-group-allocation_keycloak_1 bash
cd /tmp
export PATH=$PATH:/opt/keycloak/bin
#kc.sh export --dir test
kcadm.sh config credentials --server http://localhost:8080 --realm master --user admin --password admin
#kcadm.sh delete realms/pga
kcadm.sh create realms -s realm=pga -s enabled=true
kcadm.sh create users -r pga -s username=test -s email=test@example.com -s enabled=true
kcadm.sh set-password -r pga --username test --new-password test
CID=$(kcadm.sh create clients -r pga -s clientId=pga -s 'redirectUris=["https://h3.selfmade4u.de/*"]' -i)
CID=$(kcadm.sh get clients -r pga --fields id -q clientId=pga --format csv --noquotes)
CLIENT_SECRET=$(kcadm.sh get clients/$CID/client-secret -r pga --fields value --format csv --noquotes)
echo $CLIENT_SECRET

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
cargo test
```

## Dev

```bash
cargo +stable install cargo-hack --locked

mkcert -install
cp $(mkcert -CAROOT)/rootCA.pem .
mkcert h3.selfmade4u.de

cargo install diesel_cli --no-default-features --features postgres
export DATABASE_URL="postgres://postgres@localhost/pga?sslmode=disable"
cd perfect-group-allocation-database/
diesel database reset

# for chrome and h3 you need to listen on a port < 1024 AND you need a certificate with a public root
HETZNER_API_KEY=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx lego --email Moritz.Hedtke@t-online.de --dns hetzner --domains h3.selfmade4u.de run
export PGA_DATABASE_URL="postgres://postgres@localhost/pga?sslmode=disable"
sudo sysctl net.ipv4.ip_unprivileged_port_start=0
cargo build --bin server && sudo setcap 'cap_net_bind_service=+ep' target/debug/server && ./target/debug/server
SSLKEYLOGFILE=/tmp/sslkeylogfile.txt firefox

sudo nano /etc/sysctl.conf
vm.max_map_count=262144
sudo sysctl -p

#pipx install https://github.com/containers/podman-compose/archive/devel.tar.gz # profile support not yet in 1.0.6
# install docker-compose as that is a much better implementation. podman compose will then automatically use it.
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


https://nnethercote.github.io/perf-book/profiling.html

cargo build --features profiling --target=x86_64-unknown-linux-gnu -Z build-std --profile=release-with-debug --bin server

# I think macros don't work well with this especially the select! macro

# WARNING: Only connect without ssl over localhost. This makes the profiling better as there is not countless ssl stuff in there.
# I think you need to run this from the workspace root for debug symbols?
DATABASE_URL="postgres://postgres@localhost/pga?sslmode=disable" valgrind --trace-symtab=yes --tool=callgrind --cache-sim=yes --simulate-wb=yes --simulate-hwpref=yes --branch-sim=yes --dump-instr=yes --collect-jumps=yes --collect-bus=yes --collect-systime=nsec ./target/x86_64-unknown-linux-gnu/debug/server

use zed attack proxy to create some requests

export DEBUGINFOD_URLS="https://debuginfod.archlinux.org"
kcachegrind callgrind.out.110536

# basic example of podman in podman

podman inspect quay.io/podman/stable
podman run -it --rm --privileged quay.io/podman/stable
podman run -it --rm quay.io/podman/stable # this creates a warning

so I can reproduce with our test image and sudo which is interesting

# https://www.redhat.com/sysadmin/podman-inside-container
podman run --security-opt label=disable --user podman --device /dev/fuse -it ghcr.io/projektwahl/perfect-group-allocation:1
podman run -it --rm docker.io/library/fedora:40

# maybe the podman image works better? YEAH IT DOES
podman run --security-opt label=disable --user podman --device /dev/fuse quay.io/podman/stable podman run alpine echo hello

# IMPORTANT: podman in podman needs more than 2*65k uids because the build needs 65k and the container itself 65k
sudo usermod --add-subuids 1000000-2000000 --add-subgids 1000000-2000000 $USER

# follow this exactly and think about how subgids work
podman run -it --privileged --userns=keep-id -v $PWD:$PWD --workdir=$PWD ghcr.io/projektwahl/perfect-group-allocation:1 bash
./github/run.sh

# https://github.com/containers/podman/issues/4056
# maybe the subuid file is empty is fine as long as the inner command can create mappings?
sudo podman run -v $PWD:$PWD --workdir=$PWD --userns=keep-id -it ghcr.io/projektwahl/perfect-group-allocation:1
podman run -it --rm docker.io/library/fedora:40

# this works
podman run --rm --privileged -u podman:podman quay.io/podman/stable podman run --rm -it quay.io/podman/stable bash


winpr-makecert -rdp -n rdp-security -path rdp-security
weston --backend=rdp-backend.so --rdp4-key rdp-security/rdp-security.key
/run/user/1000/wayland-1
xfreerdp localhost:3389

# if it doesnt have external connectivity it doesn't break down on network changes? (because my wifi is buggy and it's not needed)
podman network create --internal pga
