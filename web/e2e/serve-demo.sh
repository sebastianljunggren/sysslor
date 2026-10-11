#!/bin/sh
# Seeds fresh demo data and serves it with a frozen clock, for the README screenshots.
# Expects a binary built with `--features demo` in $SYSSLOR_BIN.
set -eu

bin=${SYSSLOR_BIN:-../target/release/sysslor}
dir=$(mktemp -d)
trap 'rm -rf "$dir"' EXIT

export SYSSLOR_DATABASE_URL="sqlite://$dir/demo.db"
export SYSSLOR_TIME_ZONE=Europe/Stockholm
export SYSSLOR_LOCALE=en
export SYSSLOR_ADMIN_PASSWORD=demo
# Throwaway: the server only lives for one screenshot run.
export SYSSLOR_COOKIE_KEY=screenshots-only-not-a-secret-0123456789
export SYSSLOR_BIND_ADDR="127.0.0.1:$PORT"

"$bin" seed-demo
"$bin"
