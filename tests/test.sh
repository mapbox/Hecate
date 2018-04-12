#! /bin/bash
set -euo pipefail

cd $(dirname $0)/..

if [[ -n $(echo $COMMITMSG | grep -Po 'v[0-9]+\.[0-9]+\.[0-9]+' ) ]]; then
    echo "OK - Building Release"
    RELEASE= $(echo $COMMITMSG | grep -Po 'v[0-9]+\.[0-9]+\.[0-9]+') 

    echo "OK - Deletect: $RELEASE"
    ~/.cargo/bin/cargo build --release

    zip release.zip target/release/hecate

    curl \
        -H "Authorization: token $GITHUB_TOKEN" \
        -H "Accept: application/vnd.github.manifold-preview" \
        -H "Content-Type: application/zip" \
        --data-binary @release.zip \
        "https://uploads.github.com/repos/ingalls/Hecate/releases/${RELEASE}/assets?name=${RELEASE}-linux.zip"
else
    echo "OK - Running Tests"
    service postgresql start

    ~/.cargo/bin/cargo build

    ~/.cargo/bin/cargo test
fi
