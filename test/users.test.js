const test = require('tape');
const request = require('request');
const exec = require('child_process').exec;
const Pool = require('pg-pool');
const path = require('path');

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

        psql -v ON_ERROR_STOP=1 -q -U postgres -f ${path.resolve(__dirname, '../src/schema.sql')} hecate
    `, (err, stdout, stderr) => {
        t.error(err, 'no errors');
        t.end();
    });

});

test('users - create user', (q) => {
    q.test('users - create user - endpoint', (r) => {
        request.get({
            headers: { 'content-type' : 'application/json' },
            url: 'http://localhost:8000/api/user/create?username=ingalls&password=test123&email=ingalls@protonmail.com'
        }, (err, res) => {
            r.error(err, 'no errors');
            r.equals(res.statusCode, 200);
            r.equals(res.body, 'true');
            r.end();
        });
    });

    q.test('users - create user - endpoint - duplicate fail', (r) => {
        request.get({
            headers: { 'content-type' : 'application/json' },
            url: 'http://localhost:8000/api/user/create?username=ingalls&password=test123&email=ingalls@protonmail.com'
        }, (err, res) => {
            r.error(err, 'no errors');
            r.equals(res.statusCode, 400);
            r.equals(res.body, 'Could not create user: duplicate key value violates unique constraint "users_username_key"');
            r.end();
        });
    });

    q.test('users - create user - database', (r) => {
        pool.query('SELECT id, username, email, meta FROM users ORDER BY id', (err, res) => {
            r.error(err);
            r.deepEquals(res.rows[0], {
                id: '1',
                username: 'ingalls',
                email: 'ingalls@protonmail.com',
                meta: {}
            });

            r.end();
        });
    });

    q.test('users - create user - endpoint - no auth', (r) => {
        request.post({
            headers: { 'content-type' : 'application/json' },
            url: 'http://localhost:8000/api/data/feature',
            body: JSON.stringify({
                type: 'Feature',
                properties: {},
                geometry: {
                    type: 'Point',
                    coordinates: [0,0]
                }
            })
        }, (err, res) => {
            r.error(err, 'no errors');
            r.equals(res.statusCode, 401);
            r.deepEquals(JSON.parse(res.body), {
                code: 401,
                reason: 'You must be logged in to access this resource',
                status: 'Not Authorized'
            });
            r.end();
        });
    });

    q.test('users - create user - endpoint - bad username', (r) => {
        request.post({
            headers: { 'content-type' : 'application/json' },
            url: 'http://fakeuser:123@localhost:8000/api/data/feature',
            body: JSON.stringify({
                type: 'Feature',
                properties: {},
                geometry: {
                    type: 'Point',
                    coordinates: [0,0]
                }
            })
        }, (err, res) => {
            r.error(err, 'no errors');
            r.equals(res.statusCode, 401);
            r.deepEquals(res.body, 'Not Authorized!');
            r.end();
        });
    });

    q.test('users - create user - endpoint - bad password', (r) => {
        request.post({
            headers: { 'content-type' : 'application/json' },
            url: 'http://ingalls:321@localhost:8000/api/data/feature',
            body: JSON.stringify({
                type: 'Feature',
                properties: {},
                geometry: {
                    type: 'Point',
                    coordinates: [0,0]
                }
            })
        }, (err, res) => {
            r.error(err, 'no errors');
            r.equals(res.statusCode, 401);
            r.deepEquals(res.body, 'Not Authorized!');
            r.end();
        });
    });

    q.test('users - create user - endpoint - correct password', (r) => {
        request.post({
            headers: { 'content-type' : 'application/json' },
            url: 'http://ingalls:test123@localhost:8000/api/data/feature',
            body: JSON.stringify({
                type: 'Feature',
                action: 'create',
                properties: {},
                geometry: {
                    type: 'Point',
                    coordinates: [0,0]
                }
            })
        }, (err, res) => {
            r.error(err, 'no errors');
            r.equals(res.statusCode, 200);
            r.end();
        });
    });
});

test('Disconnect', (t) => {
    pool.end(t.end);
});
