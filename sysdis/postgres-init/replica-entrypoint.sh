#!/bin/bash
set -e

if [ -z "$(ls -A "$PGDATA" 2>/dev/null)" ]; then
  echo "[replica] Empty data dir, cloning from primary..."

  until pg_isready -h postgres -p 5432 -U replicator 2>/dev/null; do
    echo "[replica] Waiting for primary..."
    sleep 2
  done

  PGPASSWORD="$REPLICATION_PASSWORD" pg_basebackup \
    -h postgres -p 5432 -U replicator \
    -D "$PGDATA" -Fp -Xs -P -R -v

  chown -R postgres:postgres "$PGDATA"
  chmod 700 "$PGDATA"
  echo "[replica] Clone complete, starting as standby."
fi

exec docker-entrypoint.sh postgres
