#! /bin/bash
set -euo pipefail

service postgresql start

cd $(dirname $0)/..
~/.cargo/bin/cargo build

killall -9 hecate || true

function startsrv() {
    ~/.cargo/bin/cargo run&

    sleep 2

    # On OSX when cargo first runs it will be connected to `localhost:8000`
    # but not to 127.0.0.1. curl localhost:8000  will work but curl 127.0.0.1 will fail.
    # Node request converts localhost:8000 to 127.0.0.1 behind the scenes, resulting in failure
    # running a second startsrc fixes this.
    if ! curl '127.0.0.1:8000'; then
        startsrv
    else
        testsrv
    fi
}

function testsrv() {
    for TEST in tests/; do
        node tests/$TEST
    done

}

startsrv
