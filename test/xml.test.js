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
                r.equals(res.body, '1');
                r.end();
            });
        });

        q.test('xml#changeset#create - basic - database', (r) => {
            pool.query('SELECT id, features, affected, props, uid FROM deltas WHERE id = 1', (err, res) => {
                r.error(err, 'no errors');
                r.deepEquals(res.rows[0], {
                    id: '1',
                    features: null,
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
    t.test('xml#changeset#upload - create - node', (q) => {
        q.test('xml#changeset#upload - create - node - endpoint', (r) => {
            request.post({
                headers: { 'content-type' : 'application/json' },
                url: 'http://localhost:3000/api/0.6/changeset/1/upload',
                body: `
                    <osmChange version="0.6" generator="Hecate Server">
                        <create>
                            <node id='-1' version='1' changeset='1' lat='-0.66180939203' lon='3.59219690827'>
                                <tag k='amenity' v='shop' />
                                <tag k='building' v='yes' />
                            </node>
                        </create>
                    </osmChange>
                `
            }, (err, res) => {
                r.error(err, 'no errors');
                r.equals(res.statusCode, 200);

                r.equals(XML(res.body), XML`
                    <diffResult generator="Hecate Server" version="0.6">
                        <node old_id="-1" new_id="1" new_version="1"/>
                    </diffResult>
                `);
                r.end();
            });
        });

        q.test('xml#changeset#upload - create - node - database', (r) => {
            pool.query('SELECT id, version, ST_AsGeoJSON(geom)::JSON AS geom, props, deltas FROM geo WHERE id = 1;', (err, res) => {
                r.error(err, 'no errors');
                r.deepEquals(res.rows[0], {
                    id: '1',
                    version: '1',
                    geom: {
                        type: "Point",
                        coordinates: [3.59219694137573,-0.661809384822845]
                    },
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

    t.test('xml#changeset#upload - modify - node', (q) => {
        q.test('xml#changeset#upload - modify - node - endpoint', (r) => {
            request.post({
                headers: { 'content-type' : 'application/json' },
                url: 'http://localhost:3000/api/0.6/changeset/2/upload',
                body: `
                    <osmChange version="0.6" generator="Hecate Server">
                        <modify>
                            <node id='1' version='1' changeset='1' lat='1.1' lon='1.1'>
                                <tag k='building' v='house' />
                            </node>
                        </modify>
                    </osmChange>
                `
            }, (err, res) => {
                r.error(err, 'no errors');
                r.equals(res.statusCode, 200);

                r.equals(XML(res.body), XML`
                    <diffResult generator="Hecate Server" version="0.6">
                        <node old_id="1" new_id="1" new_version="2"/>
                    </diffResult>
                `);

                r.end();
            });
        });

        q.test('xml#changeset#upload - modify - node - database', (r) => {
            pool.query('SELECT id, version, ST_AsGeoJSON(geom)::JSON AS geom, props, deltas FROM geo WHERE id = 1;', (err, res) => {
                r.error(err, 'no errors');
                r.deepEquals(res.rows[0], {
                    id: '1',
                    version: '2',
                    geom: {
                        type: "Point",
                        coordinates: [1.10000002384186,1.10000002384186]
                    },
                    props: {
                        building: 'house'
                    },
                    deltas: ['1', '2']
                });
                r.end();
            });
        });

        q.end();
    });

    t.test('xml#changeset#upload - delete - node', (q) => {
        q.test('xml#changeset#upload - delete - node - endpoint', (r) => {
            request.post({
                headers: { 'content-type' : 'application/json' },
                url: 'http://localhost:3000/api/0.6/changeset/3/upload',
                body: `
                    <osmChange version="0.6" generator="Hecate Server">
                        <delete>
                            <node id='1' version='2'/>
                        </delete>
                    </osmChange>
                `
            }, (err, res) => {
                r.error(err, 'no errors');
                r.equals(res.statusCode, 200);

                r.equals(XML(res.body), XML`
                    <diffResult generator="Hecate Server" version="0.6">
                        <node old_id="1"/>
                    </diffResult>
                `);

                r.end();
            });
        });

        q.test('xml#changeset#upload - delete - node - database', (r) => {
            pool.query('SELECT * FROM geo WHERE id = 1;', (err, res) => {
                r.error(err, 'no errors');
                r.deepEquals(res.rows.length, 0);
                r.end();
            });
        });

        q.end();
    });

    t.skip('xml#changeset#upload - create - way', (q) => {
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

                r.equals(XML(res.body), XML`
                    <diffResult generator="Hecate Server" version="0.6">
                        <node old_id="1" new_id="1" new_version="2"/>
                    </diffResult>
                `);

                r.end();
            });
        });

        q.test('xml#changeset#upload - create - way - database', (r) => {
            pool.query('SELECT id, version, ST_AsGeoJSON(geom) AS geom, props, deltas FROM geo WHERE id = 2 OR id = 3 ORDER BY id;', (err, res) => {
                r.error(err, 'no errors');
                r.deepEquals(res.rows[0], {
                    id: '2',
                    version: '1',
                    geom: '{"type":"LineString","coordinates":[[1.10000002384186,1.10000002384186],[2.20000004768372,2.20000004768372],[3.29999995231628,3.29999995231628]]}',
                    props: {
                        amenity: 'shop'
                    },
                    deltas: ['1']
                });
                r.deepEquals(res.rows[1], {
                    id: '3',
                    version: '1',
                    geom: '{"type":"Polygon","coordinates":[[[1.10000002384186,1.10000002384186],[2.20000004768372,2.20000004768372],[3.29999995231628,3.29999995231628],[1.10000002384186,1.10000002384186]]]}',
                    props: {
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

function XML(xml) {
    if (Array.isArray(xml)) xml = xml[0];
    return xml.replace(/[\n\s]/g, '')
}
