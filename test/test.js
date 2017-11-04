const test = require('tape');
const request = require('request');
const exec = require('child_process').exec;

test.skip('Create Empty Database', (t) => {
    exec(`
        pkill hecate || true

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

        cargo build
    `, (err, stdout, stderr) => {
        t.error(err);
        t.end();
    });
});

test.skip('Start Server', (t) => {
    exec('cargo run');
    exec('sleep 2', (err, stdout, stderr) => {
        t.error(err);
        t.end();
    });
});

test('feature#create', (t) => {
    t.test('featur#create - no geometry/props', (q) => {
        request.post({
            headers: { 'content-type' : 'application/json' },
            url: 'http://localhost:3000/api/data/feature',
            body: JSON.stringify({
                type: 'Feature'
            })
        }, (err, res) => {
            t.error(err);

            t.equals(res.statusCode, 415);
            t.equals(res.body, 'Body must be valid GeoJSON Feature');
            q.end();
        });
    });

    t.test('featur#create - no geometry', (q) => {
        request.post({
            headers: { 'content-type' : 'application/json' },
            url: 'http://localhost:3000/api/data/feature',
            body: JSON.stringify({
                type: 'Feature',
                properties: {
                    number: '1234'
                }
            })
        }, (err, res) => {
            t.error(err);

            t.equals(res.statusCode, 415);
            t.equals(res.body, 'Body must be valid GeoJSON Feature');
            q.end();
        });
    });

    t.test('featur#create - no props', (q) => {
        request.post({
            headers: { 'content-type' : 'application/json' },
            url: 'http://localhost:3000/api/data/feature',
            body: JSON.stringify({
                type: 'Feature',
                geometry: {
                    type: 'Point',
                    coordinates: [ 0, 0 ]
                }
            })
        }, (err, res) => {
            t.error(err);

            t.equals(res.statusCode, 415);
            t.equals(res.body, 'Body must be valid GeoJSON Feature');
            q.end();
        });
    });

    t.test('featur#create - point', (q) => {
        request.post({
            headers: { 'content-type' : 'application/json' },
            url: 'http://localhost:3000/api/data/feature',
            body: JSON.stringify({
                type: 'Feature',
                properties: {
                    number: '123'
                },
                geometry: {
                    type: 'Point',
                    coordinates: [ 0, 0 ]
                }
            })
        }, (err, res) => {
            t.error(err);
            t.equals(res.statusCode, 415);
            t.equals(res.body, 'Body must be valid GeoJSON Feature');
            q.end();
        });
    });

    t.end();
});

test('Stop Server', (t) => {
    exec(`
        pkill hecate || true
    `, (err, stdout, stderr) => {
        t.error(err);
        t.end();
    });
});

/*
echo -e "\n# Simple Line Addition"
    DATA='{"geometry":{"coordinates":[[1.0,1.0],[0.0,0.0]],"type":"LineString"},"id":2,"properties":{"highway":"residential"},"type":"Feature","version":1}'

    curl -s -X POST --data "$DATA" -H 'Content-Type: application/json' 'localhost:3000/api/data/feature'

    if [[ "$(curl -s -X GET 'localhost:3000/api/data/feature/2')" == "$DATA" ]]; then echo "ok - feature matches"
    else echo "not ok - feature differs"; fi

echo -e "\n# Feature Removal"
    DATA='{"version":1}'

    if [[ $(curl -s -X DELETE --data "$DATA" 'localhost:3000/api/data/feature/2') == "true" ]]; then echo "ok - deletion returned true"
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
*/
