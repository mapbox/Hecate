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

test('xml#changeset#create', (t) => {
    t.test('xml#changeset#create - basic', (q) => {
        q.test('xml#changeset#create - basic - endpoint', (r) => {
            request.put({
                headers: { 'content-type' : 'application/json' },
                url: 'http://localhost:3000/api/0.6/changeset/create',
                body: `
                    <osm><changeset>
                        <tag k="created_by" v="Hecate Server"/>
                        <tag k="comment" v="Buncho Random Text"/>
                    </changeset></osm>
                `
            }, (err, res) => {
                r.error(err, 'no errors');
                r.equals(res.statusCode, 200);
                r.end();
            });
        });

        q.test('xml#changeset#create - basic - database', (r) => {
            pool.query('SELECT id, features, affected, props, uid FROM deltas WHERE id = 1', (err, res) => {
                r.error(err, 'no errors');
                r.deepEquals(res.rows[0], {
                    id: '1',
                    features: {
                        type: 'FeatureCollection',
                        features: []
                    },
                    affected: null,
                    props: {
                        comment: 'Buncho Random Text',
                        created_by: 'Hecate Server'
                    },
                    uid: '1'
                });
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
