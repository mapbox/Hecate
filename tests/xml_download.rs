extern crate curl;
extern crate postgres;

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::prelude::*;
    use postgres::{Connection, TlsMode};
    use curl::easy::Easy;

    fn reset_database() {
        let conn = Connection::connect("postgres://postgres@localhost:5432", TlsMode::None).unwrap();
        conn.execute("
            SELECT pg_terminate_backend(pg_stat_activity.pid)
            FROM pg_stat_activity
            WHERE
                pg_stat_activity.datname = 'hecate'
                AND pid <> pg_backend_pid();
        ", &[]).unwrap();
        conn.execute("
            DROP DATABASE hecate;
        ", &[]).unwrap();
        conn.execute("
            CREATE DATABASE hecate;
        ", &[]).unwrap();

        let mut file = File::open("./src/schema.sql").unwrap();
        let mut table_sql = String::new();
        file.read_to_string(&mut table_sql).unwrap();
        conn.batch_execute(&*table_sql).unwrap();
    }


    #[test]
    fn create_user() {
        let mut easy = Easy::new();
        easy.url("http://localhost:8000/api/user/create?username=ingalls&password=yeaheh&email=ingalls@protonmail.com").unwrap();
        easy.perform().unwrap();

        assert_eq!(easy.response_code(), Ok(200));
    }
}
/*
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
                message: 'Create Point',
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
                message: 'Create MultiPoint',
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
                message: 'Create LineString',
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
                message: 'Create MultiLineString',
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
                message: 'Create Polygon',
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
                message: 'Create Polygon-Inner',
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

    t.test('xml#Download#MultiPolygon', q => {
        request.post({
            url: 'http://ingalls:yeaheh@localhost:8000/api/data/feature',
            headers: { 'content-type' : 'application/json' },
            body: JSON.stringify({
                type: 'Feature',
                message: 'Create MultiPolygon',
                action: 'create',
                properties: {
                    building: true,
                    amenity: 'hospital'
                },
                geometry: {
                    type: 'MultiPolygon',
                    coordinates: [
                        [ [
                            [ -79.45878982543945, 43.67362593129495 ],
                            [ -79.45830166339874, 43.67362593129495 ],
                            [ -79.45830166339874, 43.67394021076283 ],
                            [ -79.45878982543945, 43.67394021076283 ],
                            [ -79.45878982543945, 43.67362593129495 ]
                        ], [
                            [ -79.45868790149689, 43.67371517131118 ],
                            [ -79.45868790149689, 43.673866491035405 ],
                            [ -79.45843577384949, 43.673866491035405 ],
                            [ -79.45843577384949, 43.67371517131118 ],
                            [ -79.45868790149689, 43.67371517131118 ]
                        ] ], [ [
                            [ -79.45853769779205, 43.67316032905796 ],
                            [ -79.45803344249725, 43.67316032905796 ],
                            [ -79.45803344249725, 43.67347461096418 ],
                            [ -79.45853769779205, 43.67347461096418 ],
                            [ -79.45853769779205, 43.67316032905796 ]
                        ], [
                            [ -79.45838212966919, 43.673284490007696 ],
                            [ -79.45821583271027, 43.673284490007696 ],
                            [ -79.45821583271027, 43.67340089066479 ],
                            [ -79.45838212966919, 43.67340089066479 ],
                            [ -79.45838212966919, 43.673284490007696 ]
                        ] ]
                    ]
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
            q.equals(res.body, '<?xml version="1.0" encoding="UTF-8"?><osm version="0.6" generator="ROSM"><node id="1" version="1" lon="-79.46014970541" lat="43.6726345821896"><tag k="addr:housenumber" v="1234"/><tag k="addr:street" v="Main St"/></node><node id="7000000000000000001" version="1" lat="43.6724366984115" lon="-79.4584304094314"/><node id="7000000000000000002" version="1" lat="43.6724269982095" lon="-79.4582131505013"/><node id="7000000000000000003" version="1" lat="43.6731292887804" lon="-79.4608980417252"/><node id="7000000000000000004" version="1" lat="43.6718760241634" lon="-79.4603669643402"/><node id="7000000000000000005" version="1" lat="43.6720622694773" lon="-79.4596266746521"/><node id="7000000000000000006" version="1" lat="43.6721553919176" lon="-79.4590795040131"/><node id="7000000000000000007" version="1" lat="43.6720661495819" lon="-79.458532333374"/><node id="7000000000000000008" version="1" lat="43.6720040678789" lon="-79.4583123922348"/><node id="7000000000000000009" version="1" lat="43.6717906615348" lon="-79.4575130939484"/><node id="7000000000000000010" version="1" lat="43.6734687909438" lon="-79.4609811902046"/><node id="7000000000000000011" version="1" lat="43.6734687909438" lon="-79.4606646895409"/><node id="7000000000000000012" version="1" lat="43.6736744313202" lon="-79.4606646895409"/><node id="7000000000000000013" version="1" lat="43.6736744313202" lon="-79.4609811902046"/><node id="7000000000000000014" version="1" lat="43.6729954273994" lon="-79.4596239924431"/><node id="7000000000000000015" version="1" lat="43.6729954273994" lon="-79.4588783383369"/><node id="7000000000000000016" version="1" lat="43.6734920710218" lon="-79.4588783383369"/><node id="7000000000000000017" version="1" lat="43.6734920710218" lon="-79.4596239924431"/><node id="7000000000000000018" version="1" lat="43.6731137686356" lon="-79.4594416022301"/><node id="7000000000000000019" version="1" lat="43.6731137686356" lon="-79.4590553641319"/><node id="7000000000000000020" version="1" lat="43.6733601504605" lon="-79.4590553641319"/><node id="7000000000000000021" version="1" lat="43.6733601504605" lon="-79.4594416022301"/><node id="7000000000000000022" version="1" lat="43.6736259312949" lon="-79.4587898254395"/><node id="7000000000000000023" version="1" lat="43.6736259312949" lon="-79.4583016633987"/><node id="7000000000000000024" version="1" lat="43.6739402107628" lon="-79.4583016633987"/><node id="7000000000000000025" version="1" lat="43.6739402107628" lon="-79.4587898254395"/><node id="7000000000000000026" version="1" lat="43.6737151713112" lon="-79.4586879014969"/><node id="7000000000000000027" version="1" lat="43.6738664910354" lon="-79.4586879014969"/><node id="7000000000000000028" version="1" lat="43.6738664910354" lon="-79.4584357738495"/><node id="7000000000000000029" version="1" lat="43.6737151713112" lon="-79.4584357738495"/><node id="7000000000000000030" version="1" lat="43.673160329058" lon="-79.4585376977921"/><node id="7000000000000000031" version="1" lat="43.673160329058" lon="-79.4580334424973"/><node id="7000000000000000032" version="1" lat="43.6734746109642" lon="-79.4580334424973"/><node id="7000000000000000033" version="1" lat="43.6734746109642" lon="-79.4585376977921"/><node id="7000000000000000034" version="1" lat="43.6732844900077" lon="-79.4583821296692"/><node id="7000000000000000035" version="1" lat="43.6732844900077" lon="-79.4582158327103"/><node id="7000000000000000036" version="1" lat="43.6734008906648" lon="-79.4582158327103"/><node id="7000000000000000037" version="1" lat="43.6734008906648" lon="-79.4583821296692"/><way id="3" version="1"><nd ref="7000000000000000003"/><nd ref="7000000000000000004"/><tag k="highway" v="residential"/><tag k="name" v="Main St E"/></way><way id="8000000000000000001" version="1"><nd ref="7000000000000000005"/><nd ref="7000000000000000006"/><nd ref="7000000000000000007"/></way><way id="8000000000000000002" version="1"><nd ref="7000000000000000008"/><nd ref="7000000000000000009"/></way><way id="5" version="1"><nd ref="7000000000000000010"/><nd ref="7000000000000000011"/><nd ref="7000000000000000012"/><nd ref="7000000000000000013"/><nd ref="7000000000000000010"/><tag k="building" v="yes"/></way><way id="8000000000000000003" version="1"><nd ref="7000000000000000014"/><nd ref="7000000000000000015"/><nd ref="7000000000000000016"/><nd ref="7000000000000000017"/><nd ref="7000000000000000014"/></way><way id="8000000000000000004" version="1"><nd ref="7000000000000000018"/><nd ref="7000000000000000019"/><nd ref="7000000000000000020"/><nd ref="7000000000000000021"/><nd ref="7000000000000000018"/></way><way id="8000000000000000005" version="1"><nd ref="7000000000000000022"/><nd ref="7000000000000000023"/><nd ref="7000000000000000024"/><nd ref="7000000000000000025"/><nd ref="7000000000000000022"/></way><way id="8000000000000000006" version="1"><nd ref="7000000000000000026"/><nd ref="7000000000000000027"/><nd ref="7000000000000000028"/><nd ref="7000000000000000029"/><nd ref="7000000000000000026"/></way><way id="8000000000000000007" version="1"><nd ref="7000000000000000030"/><nd ref="7000000000000000031"/><nd ref="7000000000000000032"/><nd ref="7000000000000000033"/><nd ref="7000000000000000030"/></way><way id="8000000000000000008" version="1"><nd ref="7000000000000000034"/><nd ref="7000000000000000035"/><nd ref="7000000000000000036"/><nd ref="7000000000000000037"/><nd ref="7000000000000000034"/></way><relation id="2" version="1"><tag k="addr:housenumber" v="1234"/><tag k="addr:street" v="yet another street"/><tag k="type" v="multipoint"/><member ref="7000000000000000001" type="node" role="point"/><member ref="7000000000000000002" type="node" role="point"/></relation><relation id="4" version="1"><tag k="highway" v="service"/><tag k="name" v="Don&apos;t drive on me"/><tag k="type" v="multilinestring"/><member ref="8000000000000000001" role="line" type="way"/><member ref="8000000000000000002" role="line" type="way"/></relation><relation id="6" version="1"><tag k="building" v="yes"/><tag k="type" v="multipolygon"/><member ref="8000000000000000003" role="outer" type="way"/><member ref="8000000000000000004" role="inner" type="way"/></relation><relation id="7" version="1"><tag k="amenity" v="hospital"/><tag k="building" v="yes"/><tag k="type" v="multipolygon"/><member ref="8000000000000000005" role="outer" type="way"/><member ref="8000000000000000006" role="inner" type="way"/><member ref="8000000000000000007" role="outer" type="way"/><member ref="8000000000000000008" role="inner" type="way"/></relation></osm>');
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
*/
