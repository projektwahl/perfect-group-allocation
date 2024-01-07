-- Your SQL goes here

-- transaction isolation
-- https://www.postgresql.org/docs/current/transaction-iso.html
-- https://www.postgresql.org/docs/current/transaction-iso.html#XACT-READ-COMMITTED
-- is the default which we also use

-- append only tables have high security guarantees

CREATE TABLE IF NOT EXISTS project_history (
  history_id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY NOT NULL,
  id INTEGER NOT NULL,
  title VARCHAR(255) NOT NULL,
  info VARCHAR(4096) NOT NULL
  --place VARCHAR(256) NOT NULL,
  --costs FLOAT NOT NULL,
  --min_age INTEGER NOT NULL,
  --max_age INTEGER NOT NULL,
  --min_participants INTEGER NOT NULL,
  --max_participants INTEGER NOT NULL,
  --random_assignments BOOLEAN NOT NULL DEFAULT FALSE,
  --deleted BOOLEAN NOT NULL DEFAULT FALSE,
  --last_updated_by INTEGER
);

CREATE INDEX IF NOT EXISTS project_history_index ON project_history (id, history_id);
