#! /bin/bash
set -euo pipefail

service postgresql start

cd ./hecate_ui
yarn install
yarn build
cd $(dirname $0);

cd $(dirname $0)/..
~/.cargo/bin/cargo build

~/.cargo/bin/cargo test
