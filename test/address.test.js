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
    `, (err, stdout, stderr) => {
        t.error(err, 'no errors');
        t.end();
    });

});

if (!process.env.DEBUG) {
    test('Compile & Run', (t) => {
        exec(`
            pkill hecate || true

            cargo build
        `, (err, stdout, stderr) => {
            t.error(err, 'no errors');
            t.end();
        });
    });

    test('Start Server', (t) => {
        exec('cargo run');
        exec('sleep 2', (err, stdout, stderr) => {
            t.error(err, 'no errors');
            t.end();
        });
    });
}

test('address', (t) => {
    t.test('address - import', (q) => {
        q.test('features - basic create - endpoint', (r) => {
            request.post({
                headers: { 'content-type' : 'application/json' },
                url: 'http://localhost:3000/api/data/features',
                body: JSON.stringify(require('./fixtures/us_dc_pts.json'))
            }, (err, res) => {
                r.error(err, 'no errors');
                r.equals(res.statusCode, 200);
                r.equals(res.body, 'true');
                r.end();
            });
        });

        q.test('address - basic create - geo database', (r) => {
            pool.query('SELECT id, version, ST_AsGeoJSON(geom)::JSON AS geometry, props AS properties, deltas FROM geo ORDER BY id', (err, res) => {
                r.error(err, 'no errors');
                r.equals(res.rows.length, 999);

                for (let row of res.rows) {
                    r.ok(row.id > 0, 'feature assigned id');
                    r.ok(row.version == 1, 'feature is version 1');
                    r.ok(row.properties.street.length > 0, 'feature retained street');
                    r.ok(row.properties.number.length > 0, 'feature retained number');
                    r.ok(row.properties.source.length > 0, 'feature retained source');
                    r.ok(row.geometry.type === 'Point', 'feature is a point');
                }
                r.end();
            });
        });

        q.test('address - basic create - deltas database', (r) => {
            pool.query('SELECT id, features, affected, props, uid FROM deltas ORDER BY id', (err, res) => {
                r.error(err, 'no errors');
                r.equals(res.rows.length, 1);
                res = res.rows[0];

                t.equals(res.id, '1');
                t.deepEquals(res.affected, null);
                t.deepEquals(res.props, {});
                t.equals(res.uid, '1');

                r.end();
            });
        });
        q.end();
    });
    t.end();
});

test('Disconnect', (t) => {
    pool.end(t.end);
});

if (!process.env.DEBUG) {
    test('Stop Server', (t) => {
        exec(`
            pkill hecate || true
        `, (err, stdout, stderr) => {
            t.error(err, 'no errors');
            t.end();
        });
    });
}
