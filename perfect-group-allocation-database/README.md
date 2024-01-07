
```
cargo install diesel_cli --no-default-features --features postgres

# podman volume rm pga-postgres
podman run --rm --detach --name postgres --volume pga-postgres:/var/lib/postgresql/data --env POSTGRES_PASSWORD=password --publish 5432:5432 docker.io/postgres

export DATABASE_URL="postgres://postgres:password@localhost/pga?sslmode=disable"

diesel database reset
psql postgres://postgres:password@localhost/pga
```

```sql
-- https://stackoverflow.com/questions/25536422/optimize-group-by-query-to-retrieve-latest-row-per-user
INSERT INTO projects_history (id) SELECT generate_series(1, 1000000) / 10;
ANALYZE VERBOSE;
EXPLAIN ANALYZE SELECT id, MAX(history_id) FROM projects_history GROUP BY id;
```