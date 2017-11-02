#! /bin/bash

set -euo pipefail

echo "
    SELECT pg_terminate_backend(pg_stat_activity.pid)
        FROM pg_stat_activity
        WHERE
            pg_stat_activity.datname = 'hecate'
            AND pid <> pg_backend_pid();
" | psql -U postgres -q >/dev/null

echo "
    DROP DATABASE hecate;
    CREATE DATABASE hecate;
" | psql -U postgres -q

psql -q -U postgres -f src/schema.sql hecate

pkill hecate || true

cargo build -q 2>/dev/null || (echo "not ok - Failed to build" && exit 1)

cargo run -q &

sleep 1

echo -e "\n# Simple Point Addition"
    DATA='{"geometry":{"coordinates":[1.0,1.0],"type":"Point"},"id":1,"properties":{"building":"yes"},"type":"Feature","version":1}'

    curl -s -X POST --data "$DATA" -H 'Content-Type: application/json' 'localhost:3000/api/data/feature'

    if [[ "$(curl -s -X GET 'localhost:3000/api/data/feature/1')" == "$DATA" ]]; then echo "ok - feature matches"
    else echo "not ok - feature differs"; fi

echo -e "\n# Simple Line Addition"
    DATA='{"geometry":{"coordinates":[[1.0,1.0],[0.0,0.0]],"type":"LineString"},"id":2,"properties":{"highway":"residential"},"type":"Feature","version":1}'

    curl -s -X POST --data "$DATA" -H 'Content-Type: application/json' 'localhost:3000/api/data/feature'

    if [[ "$(curl -s -X GET 'localhost:3000/api/data/feature/2')" == "$DATA" ]]; then echo "ok - feature matches"
    else echo "not ok - feature differs"; fi

echo -e "\n# Feature Removal"
    if [[ $(curl -s -X DELETE 'localhost:3000/api/data/feature/2') == "true" ]]; then echo "ok - deletion returned true"
    else echo "not ok - feature returned true"; fi

    if [[ $(curl -s -X GET 'localhost:3000/api/data/feature/2') == "Null or Invalid Geometry" ]]; then echo "ok - null geom"
    else echo "not ok - null geom"; fi

echo -e "\n# Simple Line Addition 2"
    DATA='{"geometry":{"coordinates":[[1.0,1.0],[0.0,0.0]],"type":"LineString"},"id":3,"properties":{"access":"resricted","highway":"residential"},"type":"Feature","version":1}'

    curl -s -X POST --data "$DATA" -H 'Content-Type: application/json' 'localhost:3000/api/data/feature'

    if [[ "$(curl -s -X GET 'localhost:3000/api/data/feature/3')" == "$DATA" ]]; then echo "ok - feature matches"
    else echo "not ok - feature differs"; fi

echo -e "\n# Feature Alteration"
    DATA='{"geometry":{"coordinates":[0.0,0.0],"type":"Point"},"id":1,"properties":{"number":"1234"},"type":"Feature","version":1}'

    curl -s -X PATCH --data "$DATA" -H 'Content-Type: application/json' 'localhost:3000/api/data/feature/1'

    if [[ "$(curl -s -X GET 'localhost:3000/api/data/feature/1')" == "$DATA" ]]; then echo "ok - feature matches"
    else echo "not ok - feature differs"; fi

echo -e "\n# Create Multiple"
    DATA='{"features":[{"geometry":{"coordinates":[0.0,0.0],"type":"Point"},"id":1,"properties":{"number":"1234"},"type":"Feature"}],"type":"FeatureCollection"}'

    curl -s -X POST --data "$DATA" -H 'Content-Type: application/json' 'localhost:3000/api/data/features'
    echo ""

echo -e "\n# XML Map"
    echo $(curl -s -X GET 'localhost:3000/api/0.6/map?bbox=-1,-1,1,1')
    echo ""

echo -e "\n# XML Changeset Create"
    DATA='<osm><changeset><tag k="created_by" v="JOSM 1.61"/><tag k="comment" v="Just adding some streetnames"/></changeset></osm>'

    curl -s -X PUT --data "$DATA" 'localhost:3000/api/0.6/changeset/create'
    echo ""
    echo "SELECT id, props FROM deltas" | psql -U postgres hecate

echo -e "\n# Features Post"
    DATA='
        {
            "type": "FeatureCollection",
            "features": [
                { "action": "create", "type": "Feature", "properties": { "addr:number": "543" }, "geometry": {"type": "Point", "coordinates": [2.1, 2.1] } },
                { "id": 1, "action": "modify", "version": 2, "type": "Feature", "properties": { "addr:number": "543" }, "geometry": {"type": "Point", "coordinates": [2.2, 1.1] } }
            ]
        }
    '

    curl -i -s -X POST --data "$DATA" -H 'Content-Type: application/json' 'localhost:3000/api/data/features'

# KILL SERVER
pkill hecate || true
