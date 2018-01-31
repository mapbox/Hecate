#! /bin/bash
set -euo pipefail

service postgresql start

cd $(dirname $0)/..
~/.cargo/bin/cargo build

echo "CREATE DATABASE hecate" | psql -U postgres

~/.cargo/bin/cargo run&
sleep 2

~/.cargo/bin/cargo test
