#!/usr/bin/env bash
# Builds and runs the backend with demo data on :8080 and the Vite dev server on :5173.
# Ctrl+C stops both. The demo database is recreated on every run.
set -euo pipefail
cd "$(dirname "$0")"

db=target/demo.db
export SYSSLOR_DATABASE_URL="sqlite://$db"
export SYSSLOR_TIME_ZONE=Europe/Stockholm
export SYSSLOR_ADMIN_PASSWORD=demo
export SYSSLOR_COOKIE_KEY="$(head -c 48 /dev/urandom | base64 -w0)"

cargo build --features demo
[[ -d web/node_modules ]] || npm --prefix web ci

# seed-demo refuses to write into a database that already has data.
rm -f "$db" "$db-wal" "$db-shm"
target/debug/sysslor seed-demo

pids=()
cleanup() {
  trap - INT TERM EXIT
  kill "${pids[@]}" 2>/dev/null || true
  wait 2>/dev/null || true
}
trap cleanup INT TERM EXIT

target/debug/sysslor & pids+=($!)
npm --prefix web run dev & pids+=($!)

echo "UI: http://localhost:5173 (admin password \"demo\")"
# Exit as soon as either one stops, taking the other down with it.
wait -n
