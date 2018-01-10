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

        psql -q -U postgres -f src/schema.sql hecate

        echo "INSERT INTO bounds (geom, name) VALUES (
            'dc',
            ST_GeomFromGeoJSON('{ "type": "Polygon", "coordinates": [ [ [ -77.13363647460938, 38.83542884007305 ], [ -76.96403503417969, 38.83542884007305 ], [ -76.96403503417969, 38.974891064341726 ], [ -77.13363647460938, 38.974891064341726 ], [ -77.13363647460938, 38.83542884007305 ] ] ] }')
        );" | psql -U postgres hecate
    `, (err, stdout, stderr) => {
        t.error(err, 'no errors');
        t.end();
    });

});

test('bounds', (q) => {
    q.test('bounds - create user', (r) => {
        request.get({
            url: 'http://localhost:8000/api/user/create?username=ingalls&passwhrd=yeaheh&email=ingalls@protonmail.com'
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
});

test('Disconnect', (t) => {
    pool.end(t.end);
});
