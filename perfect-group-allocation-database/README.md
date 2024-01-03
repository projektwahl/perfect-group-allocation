
```
cargo install diesel_cli --no-default-features --features postgres

podman run --rm --detach --name postgres --volume pga-postgres:/var/lib/postgresql/data --env POSTGRES_PASSWORD=password --publish 5432:5432 docker.io/postgres

export DATABASE_URL="postgres://postgres:password@localhost?sslmode=disable"

diesel setup

```