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

test('xml - create user', t => {
    request.get({
        url: 'http://localhost:8000/api/user/create?username=ingalls&password=yeaheh&email=ingalls@protonmail.com'
    }, (err, res) => {
        t.error(err, 'no errrors');
        t.end();
    });
});

test('xml#Download', (t) => {
    t.test('xml#Download#Point', q => {
        request.post({
            url: 'http://ingalls:yeaheh@localhost:8000/api/data/feature',
            headers: { 'content-type' : 'application/json' },
            body: JSON.stringify({
                type: 'Feature',
                action: 'create',
                properties: {
                    'addr:housenumber': '1234',
                    'addr:street': 'Main St'
                },
                geometry: {
                    type: 'Point',
                    coordinates: [ -79.46014970541, 43.67263458218963 ]
                }
            })
        }, (err, res) => {
            q.error(err);
            q.equals(res.statusCode, 200)
            q.end();
        });
    });

    t.test('xml#Download#MultiPoint', q => {
        request.post({
            url: 'http://ingalls:yeaheh@localhost:8000/api/data/feature',
            headers: { 'content-type' : 'application/json' },
            body: JSON.stringify({
                type: 'Feature',
                action: 'create',
                properties: {
                    'addr:housenumber': '1234',
                    'addr:street': 'yet another street'
                },
                geometry: {
                    type: 'MultiPoint',
                    coordinates: [[ -79.45843040943144, 43.67243669841148 ], [ -79.45821315050125, 43.67242699820951 ] ]
                }
            })
        }, (err, res) => {
            q.error(err);
            q.equals(res.statusCode, 200)
            q.end();
        });
    });

    t.test('xml#Download#LineString', q => {
        request.post({
            url: 'http://ingalls:yeaheh@localhost:8000/api/data/feature',
            headers: { 'content-type' : 'application/json' },
            body: JSON.stringify({
                type: 'Feature',
                action: 'create',
                properties: {
                    highway: 'residential',
                    name: 'Main St E'
                },
                geometry: {
                    type: 'LineString',
                    coordinates: [ [ -79.46089804172516, 43.67312928878038 ], [ -79.46036696434021, 43.67187602416343 ] ]
                }
            })
        }, (err, res) => {
            q.error(err);
            q.equals(res.statusCode, 200)
            q.end();
        });
    });
    
    t.test('xml#Download#MultiLineString', q => {
        request.post({
            url: 'http://ingalls:yeaheh@localhost:8000/api/data/feature',
            headers: { 'content-type' : 'application/json' },
            body: JSON.stringify({
                type: 'Feature',
                action: 'create',
                properties: {
                    highway: 'service',
                    name: 'Don\'t drive on me'
                },
                geometry: {
                    type: 'MultiLineString',
                    coordinates: [[
                        [ -79.4596266746521, 43.672062269477344 ],
                        [ -79.45907950401306, 43.67215539191757 ],
                        [ -79.45853233337401, 43.6720661495819 ]
                    ],[
                        [ -79.4583123922348, 43.67200406787885 ],
                        [ -79.45751309394836, 43.67179066153475 ]
                    ]]
                }
            })
        }, (err, res) => {
            q.error(err);
            q.equals(res.statusCode, 200)
            q.end();
        });
    });

    t.test('xml#Download#Polygon', q => {
        request.post({
            url: 'http://ingalls:yeaheh@localhost:8000/api/data/feature',
            headers: { 'content-type' : 'application/json' },
            body: JSON.stringify({
                type: 'Feature',
                action: 'create',
                properties: {
                    building: true
                },
                geometry: {
                    type: 'Polygon',
                    coordinates: [[
                        [ -79.46098119020462, 43.6734687909438 ],
                        [ -79.46066468954086, 43.6734687909438 ],
                        [ -79.46066468954086, 43.673674431320244 ],
                        [ -79.46098119020462, 43.673674431320244 ],
                        [ -79.46098119020462, 43.6734687909438 ]
                    ]]
                }
            })
        }, (err, res) => {
            q.error(err);
            q.equals(res.statusCode, 200)
            q.end();
        });
    });

    t.test('xml#Download#Polygon-Inner', q => {
        request.post({
            url: 'http://ingalls:yeaheh@localhost:8000/api/data/feature',
            headers: { 'content-type' : 'application/json' },
            body: JSON.stringify({
                type: 'Feature',
                action: 'create',
                properties: {
                    building: true
                },
                geometry: {
                    type: 'Polygon',
                    coordinates: [ [
                        [ -79.45962399244308, 43.67299542739944 ],
                        [ -79.45887833833694, 43.67299542739944 ],
                        [ -79.45887833833694, 43.67349207102181 ],
                        [ -79.45962399244308, 43.67349207102181 ],
                        [ -79.45962399244308, 43.67299542739944 ]
                    ], [
                        [ -79.45944160223007, 43.67311376863557 ],
                        [ -79.45905536413191, 43.67311376863557 ],
                        [ -79.45905536413191, 43.673360150460475 ],
                        [ -79.45944160223007, 43.673360150460475 ],
                        [ -79.45944160223007, 43.67311376863557 ]
                    ] ]
                }
            })
        }, (err, res) => {
            q.error(err);
            q.equals(res.statusCode, 200)
            q.end();
        });
    });

    t.test('xml#Download#get', q => {
        request.get({
            url: 'http://localhost:8000/api/0.6/map?bbox=-79.463264,43.670270,-79.456344,43.674693'
        }, (err, res) => {
            q.error(err);
            q.equals(res.body, '<?xml version="1.0" encoding="UTF-8"?><osm version="0.6" generator="ROSM"><node id="1" version="1" lon="-79.46014970541" lat="43.6726345821896"></node></osm>');
            q.end();
        });
    });

    t.end();
});


test('Disconnect', (t) => {
    pool.end(t.end);
});

function XML(xml) {
    if (Array.isArray(xml)) xml = xml[0];
    return xml.replace(/[\n\s]/g, '')
}
