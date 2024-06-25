#!/usr/bin/env bash
set -x
set -eo pipefail

DB_USER=${POSTGRES_USER:=postgres}
DB_PASSWORD=${POSTGRES_PASSWORD:=password}
DB_NAME=${POSTGRES_DB:=newsletter}
DB_PORT=${POSTGRES_PORT:=54321}
DB_HOST=${POSTGRES_HOST:=localhost}

if [[ -z "${SKIP_DOCKER}" ]]
then
    docker run \
           -e POSTGRES_USER=${DB_USER} \
           -e POSTGRES_PASSWORD=${DB_PASSWORD} \
           -e POSTGRES_DB=${DB_NAME} \
           -p "${DB_PORT}":5432 \
           -d postgres postgres -N 1000
fi

export PGPASSWORD=${DB_PASSWORD}

until psql -h ${DB_HOST} -p ${DB_PORT} -U ${DB_USER} -d "postgres" -c '\q'; do
    >&2 echo "postgres is still unavailable. Sleeping"
    sleep 1
done;

DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
export DATABASE_URL
~/.cargo/bin/sqlx database create
~/.cargo/bin/sqlx migrate run
