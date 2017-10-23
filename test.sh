#! /bin/bash

set -xeuo pipefail

echo "
    DROP DATABASE hecate;
    CREATE DATABASE hecate;
" | psql -U postgres

psql -U postgres -f src/schema.sql hecate

pkill hecate || true
cargo run &

sleep 5

curl -X POST\
    --data '{ "type": "Feature", "properties": { "building": "yes" }, "geometry": { "type": "Point", "coordinates": [ 1, 1 ] } }'\
    -H 'Content-Type: application/json'\
    'localhost:3000/api/data/feature'

echo "
    SELECT * FROM geo;
" | psql -U postgres hecate

pkill hecate
