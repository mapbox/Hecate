#!/bin/bash

set -eo pipefail

if [[ -z ${CIRCLE_TAG} ]]; then
  echo "ok - No tag, skipping release."
  exit
fi

echo "ok - Building binary."

cargo build --release

echo "ok - Publishing release."

node ./ci/publish.js $GITHUB_APP_PREFIX

echo "ok - Release published and binary uploaded"
