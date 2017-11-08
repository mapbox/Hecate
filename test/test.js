const test = require('tape');
const request = require('request');
const exec = require('child_process').exec;

test('Reset Database', (t) => {
    exec(`
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
    `, (err, stdout, stderr) => {
        t.error(err);
        t.end();
    });

});

test.skip('Compile & Run', (t) => {
    exec(`
        pkill hecate || true

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
    t.test('feature#create - no geometry/props', (q) => {
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

    t.test('feature#create - no geometry', (q) => {
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

    t.test('feature#create - no props', (q) => {
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

    t.test('feature#create - Point', (q) => {
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
            t.equals(res.statusCode, 200);
            t.equals(res.body, 'true');
            q.end();
        });
    });

    t.test('feature#create - MultiPoint', (q) => {
        request.post({
            headers: { 'content-type' : 'application/json' },
            url: 'http://localhost:3000/api/data/feature',
            body: JSON.stringify({
                type: 'Feature',
                properties: {
                    number: '123'
                },
                geometry: {
                    type: 'MultiPoint',
                    coordinates: [[ 0, 0 ], [ 1,1 ]]
                }
            })
        }, (err, res) => {
            t.error(err);
            t.equals(res.statusCode, 200);
            t.equals(res.body, 'true');
            q.end();
        });
    });

    t.test('feature#create - LineString', (q) => {
        request.post({
            headers: { 'content-type' : 'application/json' },
            url: 'http://localhost:3000/api/data/feature',
            body: JSON.stringify({
                type: 'Feature',
                properties: {
                    building: true
                },
                geometry: {
                    type: 'LineString',
                    coordinates: [[ 0, 0 ], [ 1,1 ]]
                }
            })
        }, (err, res) => {
            t.error(err);
            t.equals(res.statusCode, 200);
            t.equals(res.body, 'true');
            q.end();
        });
    });

    t.test('feature#create - MultiLineString', (q) => {
        request.post({
            headers: { 'content-type' : 'application/json' },
            url: 'http://localhost:3000/api/data/feature',
            body: JSON.stringify({
                type: 'Feature',
                properties: {
                    building: true
                },
                geometry: {
                    type: 'MultiLineString',
                    coordinates: [[[ 0, 0 ], [ 1,1 ]], [[ 1,1 ], [ 2, 2 ]]]
                }
            })
        }, (err, res) => {
            t.error(err);
            t.equals(res.statusCode, 200);
            t.equals(res.body, 'true');
            q.end();
        });
    });

    t.end();
});

test('feature#patch', (t) => {
    t.test('feature#patch - Point', (q) => {
        request.patch({
            headers: { 'content-type' : 'application/json' },
            url: 'http://localhost:3000/api/data/feature',
            body: JSON.stringify({
                id: 1,
                version: 2,
                type: 'Feature',
                properties: {
                    number: '321'
                },
                geometry: {
                    type: 'Point',
                    coordinates: [ 1, 1 ]
                }
            })
        }, (err, res) => {
            t.error(err);
            t.equals(res.statusCode, 200);
            t.equals(res.body, 'true');
            q.end();
        });
    });

    t.test('feature#patch - MultiPoint', (q) => {
        request.patch({
            headers: { 'content-type' : 'application/json' },
            url: 'http://localhost:3000/api/data/feature',
            body: JSON.stringify({
                id: 2,
                version: 2,
                type: 'Feature',
                properties: {
                    number: '321'
                },
                geometry: {
                    type: 'MultiPoint',
                    coordinates: [[ 1, 1 ], [ 0, 0 ]]
                }
            })
        }, (err, res) => {
            t.error(err);
            t.equals(res.statusCode, 200);
            t.equals(res.body, 'true');
            q.end();
        });
    });

    t.test('feature#patch - LineString', (q) => {
        request.patch({
            headers: { 'content-type' : 'application/json' },
            url: 'http://localhost:3000/api/data/feature',
            body: JSON.stringify({
                id: 3,
                version: 2,
                type: 'Feature',
                properties: {
                    building: false
                },
                geometry: {
                    type: 'LineString',
                    coordinates: [[ 1, 1 ], [ 0, 0 ]]
                }
            })
        }, (err, res) => {
            t.error(err);
            t.equals(res.statusCode, 200);
            t.equals(res.body, 'true');
            q.end();
        });
    });

    t.test('feature#patch - MultiLineString', (q) => {
        request.patch({
            headers: { 'content-type' : 'application/json' },
            url: 'http://localhost:3000/api/data/feature',
            body: JSON.stringify({
                id: 4,
                version: 2,
                type: 'Feature',
                properties: {
                    building: false
                },
                geometry: {
                    type: 'MultiLineString',
                    coordinates: [[[ 1, 1 ], [ 0, 0 ]], [[ 2, 2 ], [ 1, 1 ]]]
                }
            })
        }, (err, res) => {
            t.error(err);
            t.equals(res.statusCode, 200);
            t.equals(res.body, 'true');
            q.end();
        });
    });

    t.end();
});

test.skip('feature#delete', (t) => {
    t.test('feature#delete - version mismatch', (q) => {
        request.delete({
            headers: { 'content-type' : 'application/json' },
            url: 'http://localhost:3000/api/data/feature',
            body: JSON.stringify({
                id: 1,
                type: 'Feature',
                version: 1,
                properties: null,
                geometry: null
            })
        }, (err, res) => {
            t.error(err);

            t.equals(res.statusCode, 417);
            t.equals(res.body, 'Delete Version Mismatch');
            q.end();
        });
    });

    t.test('feature#delete - Point', (q) => {
        request.delete({
            headers: { 'content-type' : 'application/json' },
            url: 'http://localhost:3000/api/data/feature',
            body: JSON.stringify({
                id: 1,
                type: 'Feature',
                version: 3,
                properties: null,
                geometry: null
            })
        }, (err, res) => {
            t.error(err);

            t.equals(res.statusCode, 200);
            t.equals(res.body, 'true');
            q.end();
        });
    });

    t.test('feature#delete - MultiPoint', (q) => {
        request.delete({
            headers: { 'content-type' : 'application/json' },
            url: 'http://localhost:3000/api/data/feature',
            body: JSON.stringify({
                id: 2,
                type: 'Feature',
                version: 3,
                properties: null,
                geometry: null
            })
        }, (err, res) => {
            t.error(err);

            t.equals(res.statusCode, 200);
            t.equals(res.body, 'true');
            q.end();
        });
    });

    t.test('feature#delete - LineString', (q) => {
        request.delete({
            headers: { 'content-type' : 'application/json' },
            url: 'http://localhost:3000/api/data/feature',
            body: JSON.stringify({
                id: 3,
                type: 'Feature',
                version: 3,
                properties: null,
                geometry: null
            })
        }, (err, res) => {
            t.error(err);

            t.equals(res.statusCode, 200);
            t.equals(res.body, 'true');
            q.end();
        });
    });

    t.test('feature#delete - MultiLineString', (q) => {
        request.delete({
            headers: { 'content-type' : 'application/json' },
            url: 'http://localhost:3000/api/data/feature',
            body: JSON.stringify({
                id: 4,
                type: 'Feature',
                version: 3,
                properties: null,
                geometry: null
            })
        }, (err, res) => {
            t.error(err);

            t.equals(res.statusCode, 200);
            t.equals(res.body, 'true');
            q.end();
        });
    });
});

t.test('features', (t) => {
    t.test('features#basic', (q) => {
        request.post({
            headers: { 'content-type' : 'application/json' },
            url: 'http://localhost:3000/api/data/features',
            body: JSON.stringify({
                type: 'FeatureCollection',
                features: []
            })
        }, (err, res) => {
            t.error(err);

            t.equals(res.statusCode, 200);
            t.equals(res.body, 'true');
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

/**
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
