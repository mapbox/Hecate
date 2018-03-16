extern crate reqwest;
extern crate postgres;

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::prelude::*;
    use postgres::{Connection, TlsMode};
    use std::process::Command;
    use reqwest;

    #[test]
    fn features() {
        let mut server = Command::new("../hecate/target/debug/hecate").spawn().unwrap();

        {
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

            let conn = Connection::connect("postgres://postgres@localhost:5432/hecate", TlsMode::None).unwrap();

            let mut file = File::open("./src/schema.sql").unwrap();
            let mut table_sql = String::new();
            file.read_to_string(&mut table_sql).unwrap();
            conn.batch_execute(&*table_sql).unwrap();
        }

        { //Create Username
            let mut resp = reqwest::get("http://localhost:8000/api/user/create?username=ingalls&password=yeaheh&email=ingalls@protonmail.com").unwrap();
            assert_eq!(resp.text().unwrap(), "true");
            assert!(resp.status().is_success());
        }

        { //Create Points
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/features")
                .body(r#"{
                    "type": "FeatureCollection",
                    "message": "Basic Creation",
                    "features": [{
                        "type": "Feature",
                        "action": "create",
                        "properties": {
                            "shop": true
                        },
                        "geometry": {
                            "type": "Point",
                            "coordinates": [ 1,1 ]
                        }
                    }, {
                        "type": "Feature",
                        "action": "create",
                        "properties": {
                            "shop": true
                        },
                        "geometry": {
                            "type": "Point",
                            "coordinates": [ 1.1,1.1 ]
                        }
                    }, {
                        "type": "Feature",
                        "action": "create",
                        "properties": {
                            "shop": true
                        },
                        "geometry": {
                            "type": "Point",
                            "coordinates": [ 1.2,1.2 ]
                        }
                    }]
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert!(resp.status().is_success());
            assert_eq!(resp.text().unwrap(), "true");
        }

        {
            let resp = reqwest::get("http://localhost:8000/api/data/feature/1").unwrap();
            assert!(resp.status().is_success());
            //TODO check body
        }

        {
            let resp = reqwest::get("http://localhost:8000/api/data/feature/2").unwrap();
            assert!(resp.status().is_success());
            //TODO check body
        }

        {
            let resp = reqwest::get("http://localhost:8000/api/data/feature/3").unwrap();
            assert!(resp.status().is_success());
            //TODO check body
        }

        { //Modify Points
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/features")
                .body(r#"{
                    "type": "FeatureCollection",
                    "message": "Basic Modify",
                    "features": [{
                        "id": 1,
                        "type": "Feature",
                        "version": 1,
                        "action": "modify",
                        "properties": {
                            "shop": false
                        },
                        "geometry": {
                            "type": "Point",
                            "coordinates": [ 2,2 ]
                        }
                    }, {
                        "id": 2,
                        "type": "Feature",
                        "version": 1,
                        "action": "modify",
                        "properties": {
                            "shop": true,
                            "building": true
                        },
                        "geometry": {
                            "type": "Point",
                            "coordinates": [ 0.1, 0.1 ]
                        }
                    }, {
                        "id": 3,
                        "type": "Feature",
                        "version": 1,
                        "action": "modify",
                        "properties": {
                            "shop": true
                        },
                        "geometry": {
                            "type": "Point",
                            "coordinates": [ 2.2, 2.2 ]
                        }
                    }]
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert_eq!(resp.text().unwrap(), "true");
            assert!(resp.status().is_success());
        }

        {
            let resp = reqwest::get("http://localhost:8000/api/data/feature/1").unwrap();
            assert!(resp.status().is_success());
            //TODO check body
        }

        {
            let resp = reqwest::get("http://localhost:8000/api/data/feature/2").unwrap();
            assert!(resp.status().is_success());
            //TODO check body
        }

        {
            let resp = reqwest::get("http://localhost:8000/api/data/feature/3").unwrap();
            assert!(resp.status().is_success());
            //TODO check body
        }

        { //Delete Points
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/features")
                .body(r#"{
                    "type": "FeatureCollection",
                    "message": "Basic Creation",
                    "features": [{
                        "id": 1,
                        "type": "Feature",
                        "version": 2,
                        "action": "delete",
                        "properties": null,
                        "geometry": null
                    }, {
                        "id": 2,
                        "type": "Feature",
                        "version": 2,
                        "action": "delete",
                        "properties": null,
                        "geometry": null
                    }, {
                        "id": 3,
                        "type": "Feature",
                        "version": 2,
                        "action": "delete",
                        "properties": null,
                        "geometry": null
                    }]
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert!(resp.status().is_success());
            assert_eq!(resp.text().unwrap(), "true");
        }

        {
            let resp = reqwest::get("http://localhost:8000/api/data/feature/1").unwrap();
            assert!(resp.status().is_client_error());
        }

        {
            let resp = reqwest::get("http://localhost:8000/api/data/feature/2").unwrap();
            assert!(resp.status().is_client_error());
        }

        {
            let resp = reqwest::get("http://localhost:8000/api/data/feature/3").unwrap();
            assert!(resp.status().is_client_error());
        }

        server.kill().unwrap();
    }
}
