#! /bin/bash

set -euo pipefail

echo "
    DROP DATABASE hecate;
    CREATE DATABASE hecate;
" | psql -U postgres -q

psql -q -U postgres -f src/schema.sql hecate

pkill hecate || true

cargo build -q 2>/dev/null || (echo "not ok - Failed to build" && exit 1)

cargo run -q &

sleep 1

curl -X POST\
    --data '{ "type": "Feature", "properties": { "building": "yes" }, "geometry": { "type": "Point", "coordinates": [ 1, 1 ] } }'\
    -H 'Content-Type: application/json'\
    'localhost:3000/api/data/feature'

echo "
    SELECT * FROM geo;
" | psql -U postgres hecate

pkill hecate || true
