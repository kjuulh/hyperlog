#!/usr/bin/env zsh

echo "starting services"
docker compose -f templates/docker-compose.yaml up -d --remove-orphans

tear_down() {
  docker compose -f templates/docker-compose.yaml down -v || true
}

trap  tear_down EXIT

RUST_LOG=trace,tokio=info,tower=info,mio=info,sqlx=info cargo run -F include_server -- serve
