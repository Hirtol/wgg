#!/usr/bin/env just --justfile
set windows-shell := ["powershell"]

release:
  cargo build --release    

clippy:
  cargo clippy

generate-entity:
  sea-orm-cli generate entity -o .\wgg_db_entity\src\entity

migrate:
  sqlx migrate run

create-migration name:
  sqlx migrate add {{name}}