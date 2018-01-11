const test = require('tape');
const request = require('request');
const exec = require('child_process').exec;
const Pool = require('pg-pool');

const pool = new Pool({
    database: 'hecate',
    user: 'postgres',
    port: 5432
});

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

        psql -v ON_ERROR_STOP=1 -q -U postgres -f src/schema.sql hecate
    `, (err, stdout, stderr) => {
        t.error(err, 'no errors');
        t.end();
    });

});

test('bounds', (q) => {
    q.test('bounds - insert bounds data', (r) => {
        pool.query(`
            INSERT INTO bounds(geom, name) VALUES (
                ST_SetSRID(ST_GeomFromGeoJSON('{ "type": "Polygon", "coordinates": [ [ [ -77.13363647460938, 38.83542884007305 ], [ -76.96403503417969, 38.83542884007305 ], [ -76.96403503417969, 38.974891064341726 ], [ -77.13363647460938, 38.974891064341726 ], [ -77.13363647460938, 38.83542884007305 ] ] ] }'), 4326),
                'dc'
            );
        `, (err, res) => {
            r.error(err, 'no errors');
            r.end();
        });
    });

    q.test('bounds - create user', (r) => {
        request.get({
            url: 'http://localhost:8000/api/user/create?username=ingalls&password=yeaheh&email=ingalls@protonmail.com'
        }, (err, res) => {
            r.error(err, 'no errors');
            r.equals(res.statusCode, 200);
            r.end();
        });
    });

    q.test('bounds - point inside of bounds', (r) => {
        request.post({
            headers: { 'content-type' : 'application/json' },
            url: 'http://ingalls:yeaheh@localhost:8000/api/data/feature',
            body: JSON.stringify({
                type: 'Feature',
                action: 'create',
                properties: {
                    indc: true
                },
                geometry: {
                    type: 'Point',
                    coordinates: [-77.01210021972656,38.925763232374514]
                }
            })
        }, (err, res) => {
            r.error(err, 'no errors');
            r.equals(res.statusCode, 200);
            r.equals(res.body, 'true');
            r.end();
        });
    });

    q.test('bounds - point outside of bounds', (r) => {
        request.post({
            headers: { 'content-type' : 'application/json' },
            url: 'http://ingalls:yeaheh@localhost:8000/api/data/feature',
            body: JSON.stringify({
                type: 'Feature',
                action: 'create',
                properties: {
                    indc: false
                },
                geometry: {
                    type: 'Point',
                    coordinates: [-77.01210021972656,38.925763232374514]
                }
            })
        }, (err, res) => {
            r.error(err, 'no errors');
            r.equals(res.statusCode, 200);
            r.equals(res.body, 'true');
            r.end();
        });
    });

    q.test('bounds - list all', (r) => {
        request.get({
            url: 'http://localhost:8000/api/data/bounds',
        }, (err, res) => {
            r.error(err, 'no errors');
            r.equals(res.statusCode, 200);
            r.deepEquals(JSON.parse(res.body), [ 'dc' ]);
            r.end();
        });
    });

    q.test('bounds - get dc', (r) => {
        request.get({
            url: 'http://localhost:8000/api/data/bounds/dc',
        }, (err, res) => {
            r.error(err, 'no errors');
            r.equals(res.statusCode, 200);
            r.deepEquals(JSON.parse(res.body), {
                
            });
            r.end();
        });
    });
});

test('Disconnect', (t) => {
    pool.end(t.end);
});
