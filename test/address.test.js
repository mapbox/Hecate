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

test('address', (q) => {
    //every test should have the delta affect all 999 features
    let affected = [];
    for (let i = 1; i < 1000; i++) affected.push(String(i));

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
        pool.query('SELECT id, features::JSON, affected, props, uid, finalized FROM deltas ORDER BY id', (err, res) => {
            r.error(err, 'no errors');
            r.equals(res.rows.length, 1);
            res = res.rows[0];

            r.equals(res.id, '1');
            r.deepEquals(res.affected, affected);
            r.deepEquals(res.props, {});

            for (let row of res.features.features) {
                r.ok(row.id > 0, 'feature assigned id');
            }
            r.equals(res.uid, '1');
            r.equals(res.finalized, true);

            r.end();
        });
    });

    q.test('features - basic modify - endpoint', (r) => {
        let id = 0;
        request.post({
            headers: { 'content-type' : 'application/json' },
            url: 'http://localhost:3000/api/data/features',
            body: JSON.stringify({
                type: 'FeatureCollection',
                features: require('./fixtures/us_dc_pts.json').features.map((feat) => {
                    feat.id = ++id;
                    feat.action = 'modify';
                    feat.version = 1;
                    feat.properties = {
                        orange: 'is the new',
                        black: true
                    }
                    return feat;
                })
            })
        }, (err, res) => {
            r.error(err, 'no errors');
            r.equals(res.statusCode, 200);
            r.equals(res.body, 'true');
            r.end();
        });
    });

    q.test('address - basic modify - geo database', (r) => {
        pool.query('SELECT id, version, ST_AsGeoJSON(geom)::JSON AS geometry, props AS properties, deltas FROM geo ORDER BY id', (err, res) => {
            r.error(err, 'no errors');
            r.equals(res.rows.length, 999);

            for (let row of res.rows) {
                r.ok(row.id > 0, 'feature assigned id');
                r.ok(row.version == 2, 'feature is version 1');
                r.ok(row.properties.black === true, 'feature retained street');
                r.ok(row.properties.orange.length > 0, 'feature retained number');
                r.ok(row.geometry.type === 'Point', 'feature is a point');
            }
            r.end();
        });
    });

    q.test('address - basic modify - deltas database', (r) => {
        pool.query('SELECT id, features, affected, props, uid, finalized FROM deltas ORDER BY id', (err, res) => {
            r.error(err, 'no errors');
            r.equals(res.rows.length, 2);
            res = res.rows[1];

            r.equals(res.id, '2');
            r.deepEquals(res.affected, affected);
            r.deepEquals(res.props, {});
            r.equals(res.uid, '1');
            r.equals(res.finalized, true);

            r.end();
        });
    });

    q.test('features - basic delete - endpoint', (r) => {
        let id = 0;
        request.post({
            headers: { 'content-type' : 'application/json' },
            url: 'http://localhost:3000/api/data/features',
            body: JSON.stringify({
                type: 'FeatureCollection',
                features: require('./fixtures/us_dc_pts.json').features.map((feat) => {
                    feat.id = ++id;
                    feat.action = 'delete';
                    feat.version = 2;
                    return feat;
                })
            })
        }, (err, res) => {
            r.error(err, 'no errors');
            r.equals(res.statusCode, 200);
            r.equals(res.body, 'true');
            r.end();
        });
    });

    q.test('address - basic modify - geo database', (r) => {
        pool.query('SELECT id, version, ST_AsGeoJSON(geom)::JSON AS geometry, props AS properties, deltas FROM geo ORDER BY id', (err, res) => {
            r.error(err, 'no errors');
            r.equals(res.rows.length, 0);
            r.end();
        });
    });

    q.test('address - basic modify - deltas database', (r) => {
        pool.query('SELECT id, features, affected, props, uid, finalized FROM deltas ORDER BY id', (err, res) => {
            r.error(err, 'no errors');
            r.equals(res.rows.length, 3);
            res = res.rows[2];

            r.equals(res.id, '3');
            r.deepEquals(res.affected, affected);
            r.deepEquals(res.props, {});
            r.equals(res.uid, '1');
            r.equals(res.finalized, true);

            r.end();
        });
    });
    q.end();
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
