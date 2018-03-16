#! /bin/bash
set -euo pipefail

service postgresql start

cd $(dirname $0)/..
~/.cargo/bin/cargo build

~/.cargo/bin/cargo run&
sleep 2

~/.cargo/bin/cargo test
