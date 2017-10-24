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

echo "# --- Simple Point Addition ---"
    curl -s -X POST\
        --data '{ "type": "Feature", "properties": { "building": "yes" }, "geometry": { "type": "Point", "coordinates": [ 1, 1 ] } }'\
        -H 'Content-Type: application/json'\
        'localhost:3000/api/data/feature'

    echo $(curl -s -X GET 'localhost:3000/api/data/feature/1')
    echo ""
    echo "SELECT id, version, ST_AsGeoJSON(geom), props, hashes FROM geo WHERE id = 1;" | psql -U postgres hecate

echo "# --- Simple Line Addition ---"
    curl -s -X POST\
        --data '{ "type": "Feature", "properties": { "highway": "residential" }, "geometry": { "type": "LineString", "coordinates": [ [ 1, 1 ], [ 0, 0 ] ] } }'\
        -H 'Content-Type: application/json'\
        'localhost:3000/api/data/feature'

    echo $(curl -s -X GET 'localhost:3000/api/data/feature/2')
    echo ""
    echo "SELECT id, version, ST_AsGeoJSON(geom), props, hashes FROM geo WHERE id = 2;" | psql -U postgres hecate

echo "# --- XML Map ---"
    echo $(curl -s -X GET 'localhost:3000/api/0.6/map?bbox=-1,-1,1,1')
    echo ""

# KILL SERVER
pkill hecate || true
