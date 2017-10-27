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

echo "# Simple Point Addition"
    DATA='{"geometry":{"coordinates":[1.0,1.0],"type":"Point"},"id":1,"properties":{"building":"yes"},"type":"Feature","version":1}'

    curl -s -X POST --data "$DATA" -H 'Content-Type: application/json' 'localhost:3000/api/data/feature'

    if [[ "$(curl -s -X GET 'localhost:3000/api/data/feature/1')" == "$DATA" ]]; then echo "ok - feature matches"
    else echo "not ok - feature differs"; fi
echo "# Simple Line Addition"
    DATA='{"geometry":{"coordinates":[[1.0,1.0],[0.0,0.0]],"type":"LineString"},"id":2,"properties":{"highway":"residential"},"type":"Feature","version":1}'

    curl -s -X POST --data "$DATA" -H 'Content-Type: application/json' 'localhost:3000/api/data/feature'

    if [[ "$(curl -s -X GET 'localhost:3000/api/data/feature/2')" == "$DATA" ]]; then echo "ok - feature matches"
    else echo "not ok - feature differs"; fi
echo "# Feature Removal"
    if [[ $(curl -s -X DELETE 'localhost:3000/api/data/feature/2') == "true" ]]; then echo "ok - deletion returned true"
    else echo "not ok - feature returned true"; fi

    if [[ $(curl -s -X GET 'localhost:3000/api/data/feature/2') == "Null or Invalid Geometry" ]]; then echo "ok - null geom"
    else echo "not ok - null geom"; fi

echo "# Feature Alteration"
    DATA='{"geometry":{"coordinates":[0.0,0.0],"type":"Point"},"id":1,"properties":{"number":"1234"},"type":"Feature","version":1}'

    curl -s -X PATCH --data "$DATA" -H 'Content-Type: application/json' 'localhost:3000/api/data/feature/1'

    if [[ "$(curl -s -X GET 'localhost:3000/api/data/feature/1')" == "$DATA" ]]; then echo "ok - feature matches"
    else echo "not ok - feature differs"; fi

echo "# XML Map"
    echo $(curl -s -X GET 'localhost:3000/api/0.6/map?bbox=-1,-1,1,1')
    echo ""

# KILL SERVER
pkill hecate || true
