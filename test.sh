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

# --- Simple Point Addition ---
curl -X POST\
    --data '{ "type": "Feature", "properties": { "building": "yes" }, "geometry": { "type": "Point", "coordinates": [ 1, 1 ] } }'\
    -H 'Content-Type: application/json'\
    'localhost:3000/api/data/feature'

echo "
    SELECT id, version, ST_AsGeoJSON(geom), props, hashes FROM geo;
" | psql -U postgres hecate

# --- Simple Line Addition ---
curl -X POST\
    --data '{ "type": "Feature", "properties": { "highway": "residential" }, "geometry": { "type": "LineString", "coordinates": [ [ 1, 1 ], [ 0, 0 ] ] } }'\
    -H 'Content-Type: application/json'\
    'localhost:3000/api/data/feature'

echo "
    SELECT id, version, ST_AsGeoJSON(geom), props, hashes FROM geo;
" | psql -U postgres hecate

pkill hecate || true
