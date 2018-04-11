#! /bin/bash
set -euo pipefail

cd $(dirname $0)/..

if [[ -n $(echo $GIT_COMMIT_DESC | grep -Po 'v[0-9]+\.[0-9]+\.[0-9]+' ) ]]; then
    RELEASE= $(echo $GIT_COMMIT_DESC | grep -Po 'v[0-9]+\.[0-9]+\.[0-9]+') 

    ~/.cargo/bin/cargo build --release

    zip release.zip target/release/hecate

    curl \
        -H "Authorization: token $GITHUB_TOKEN" \
        -H "Accept: application/vnd.github.manifold-preview" \
        -H "Content-Type: application/zip" \
        --data-binary @release.zip \
        "https://uploads.github.com/repos/ingalls/Hecate/releases/${RELEASE}/assets?name=${RELEASE}-linux.zip"
else
    service postgresql start

    ~/.cargo/bin/cargo build

    ~/.cargo/bin/cargo test
fi


