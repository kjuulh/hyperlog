#!/usr/bin/env zsh

echo "starting services"
docker compose -f templates/docker-compose.yaml up -d --remove-orphans

sleep 5

tear_down() {
  echo "cleaning up services in the background"
  (docker compose -f templates/docker-compose.yaml down -v &) > /dev/null 2>&1
}

trap tear_down SIGINT

RUST_LOG=info,hyperlog=trace cargo watch -x 'run -F include_server -- serve'
