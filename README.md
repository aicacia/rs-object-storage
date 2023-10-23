# Aicacia Object Storage API

[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue")](LICENSE-MIT)
[![Test Status](https://github.com/aicacia/rs-object-storage/actions/workflows/test.yml/badge.svg)](https://github.com/aicacia/rs-object_storage/actions)

aicacia object_storage api

## Dev

- install [rustup](https://rustup.rs/)
- install [cargo-watch](https://crates.io/crates/cargo-watch)
- install [sqlx-cli](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli)
- rename .env file `cp .env.example .env`
- Startup main web service `cargo watch -c -w src -x run`
- create postgres `docker compose -f postgres-docker-compose.yaml up -d`
- delete postgres `docker compose -f postgres-docker-compose.yaml down` and `docker volume rm object_storage_object_storage-postgres`

## Build

- `cargo install --path .`

## Migrations

- create the database `sqlx database create`
- run migrations `sqlx migrate run`
- prepare for offline `cargo sqlx prepare`

## Extra DB Commands

- drop the database `sqlx database drop`
- revert migrations `sqlx migrate revert`

## Docker

- `docker build -t ghcr.io/aicacia/rs-object-storage:latest .`
- `docker push ghcr.io/aicacia/rs-object-storage:latest`
