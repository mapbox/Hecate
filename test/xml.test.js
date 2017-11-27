const test = require('tape');
const request = require('request');
const exec = require('child_process').exec;
const Pool = require('pg-pool');
const path = require('path');
const fs = require('fs');

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
                r.equals(res.body, '1');
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

test('xml#changeset#upload', (t) => {
    t.test('xml#changeset#upload - node', (q) => {
        q.test('xml#changeset#upload - node - endpoint', (r) => {
            request.post({
                headers: { 'content-type' : 'application/json' },
                url: 'http://localhost:3000/api/0.6/changeset/1/upload',
                body: `
                    <osmChange version="0.6" generator="Hecate Server">
                        <create>
                            <node id='1' version='1' changeset='1' lat='-0.66180939203' lon='3.59219690827'>
                                <tag k='amenity' v='shop' />
                                <tag k='building' v='yes' />
                            </node>
                        </create>
                    </osmChange>
                `
            }, (err, res) => {
                r.error(err, 'no errors');
                r.equals(res.statusCode, 200);


                let fixture = String(fs.readFileSync(path.resolve(__dirname, 'fixtures/xml#changeset#upload#node')));
                r.equals(res.body, fixture);
                if (res.body != fixture && process.env.UPDATE) {
                    t.fail('Updated Fixture');
                    fs.writeFileSync(path.resolve(__dirname, 'fixtures/xml#changeset#upload#node'), res.body);
                }

                r.end();
            });
        });

        q.test('xml#changeset#upload - node - database', (r) => {
            pool.query('SELECT id, version, ST_AsGeoJSON(geom) AS geom, props, deltas FROM geo WHERE id = 1;', (err, res) => {
                r.error(err, 'no errors');
                r.deepEquals(res.rows[0], {
                    id: '1',
                    version: '1',
                    geom: '{"type":"Point","coordinates":[3.59219694137573,-0.661809384822845]}',
                    props: {
                        amenity: 'shop',
                        building: 'yes'
                    },
                    deltas: ['1']
                });
                r.end();
            });
        });

        q.end();
    });

    t.test('xml#changeset#upload - node', (q) => {
        q.test('xml#changeset#upload - way - endpoint', (r) => {
            request.post({
                headers: { 'content-type' : 'application/json' },
                url: 'http://localhost:3000/api/0.6/changeset/1/upload',
                body: `
                    <osmChange version="0.6" generator="Hecate Server">
                        <create>
                            <node id='-1' changeset='1' lat='1.1' lon='1.1'/>
                            <node id='-2' changeset='1' lat='2.2' lon='2.2'/>
                            <node id='-3' changeset='1' lat='3.3' lon='3.3'/>
                            <way id='-1' version='1' changeset='1'>
                                <tag k='amenity' v='shop' />
                                <nd ref='-1'/>
                                <nd ref='-2'/>
                                <nd ref='-3'/>
                            </way>
                            <way id='-2' version='1' changeset='1'>
                                <tag k='building' v='yes' />
                                <nd ref='-1'/>
                                <nd ref='-2'/>
                                <nd ref='-3'/>
                                <nd ref='-1'/>
                            </way>
                        </create>
                    </osmChange>
                `
            }, (err, res) => {
                r.error(err, 'no errors');
                r.equals(res.statusCode, 200);

                let fixture = String(fs.readFileSync(path.resolve(__dirname, 'fixtures/xml#changeset#upload#way')));
                r.equals(res.body, fixture);
                if (res.body != fixture && process.env.UPDATE) {
                    t.fail('Updated Fixture');
                    fs.writeFileSync(path.resolve(__dirname, 'fixtures/xml#changeset#upload#way'), res.body);
                }

                r.end();
            });
        });

        q.test('xml#changeset#upload - way - database', (r) => {
            pool.query('SELECT id, version, ST_AsGeoJSON(geom) AS geom, props, deltas FROM geo WHERE id = 2;', (err, res) => {
                r.error(err, 'no errors');
                r.deepEquals(res.rows[0], {
                    id: '1',
                    version: '1',
                    geom: '{"type":"Point","coordinates":[3.59219694137573,-0.661809384822845]}',
                    props: {
                        amenity: 'shop',
                        building: 'yes'
                    },
                    deltas: ['1']
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
