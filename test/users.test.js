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
});

test('Disconnect', (t) => {
    pool.end(t.end);
});
