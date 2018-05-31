extern crate reqwest;
extern crate postgres;

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::prelude::*;
    use postgres::{Connection, TlsMode};
    use std::process::Command;
    use std::time::Duration;
    use std::thread;
    use reqwest;

    #[test]
    fn feature() {
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
                DROP DATABASE IF EXISTS hecate;
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

        let mut server = Command::new("cargo").arg("run").spawn().unwrap();
        thread::sleep(Duration::from_secs(1));

        { //Create Username
            let mut resp = reqwest::get("http://localhost:8000/api/user/create?username=ingalls&password=yeaheh&email=ingalls@protonmail.com").unwrap();
            assert_eq!(resp.text().unwrap(), "true");
            assert!(resp.status().is_success());
        }

        { //Create Point - No Props - No Geom
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "type": "Feature",
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert!(resp.status().is_client_error());
            assert_eq!(resp.text().unwrap(), "Body must be valid GeoJSON Feature");
        }

        { //Create Point - No Geometry
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "type": "Feature",
                    "properties": { "number": "1234" }
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert!(resp.status().is_client_error());
            assert_eq!(resp.text().unwrap(), "Body must be valid GeoJSON Feature");
        }

        { //Create Point - No Props
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "type": "Feature",
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert!(resp.status().is_client_error());
            assert_eq!(resp.text().unwrap(), "Body must be valid GeoJSON Feature");
        }

        { //Create Point - No Geom
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "type": "Feature",
                    "geometry": { "type": "Point", "coordinates": [ 0, 0 ] }
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert!(resp.status().is_client_error());
            assert_eq!(resp.text().unwrap(), "Body must be valid GeoJSON Feature");
        }

        { //Create Point - No Props - Geom
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "type": "Feature",
                    "geometry": {
                        "type": "Point",
                        "coordinates": [ 0, 0 ]
                    }
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert!(resp.status().is_client_error());
            assert_eq!(resp.text().unwrap(), "Body must be valid GeoJSON Feature");
        }

        { //Create Point - No Message
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "type": "Feature",
                    "properties": { },
                    "geometry": { "type": "Point", "coordinates": [ 0, 0 ] }
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert!(resp.status().is_client_error());
            assert_eq!(resp.text().unwrap(), "Feature Must have message property for delta");
        }

        { //Create Point - Invalid version
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "version": 15,
                    "type": "Feature",
                    "action": "create",
                    "message": "Creating a Point",
                    "properties": { },
                    "geometry": { "type": "Point", "coordinates": [ 0, 0 ] }
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert!(resp.status().is_client_error());
            assert_eq!(resp.text().unwrap(), "Create Error: Should not have \'version\' property");
        }

        { //Create Point
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "type": "Feature",
                    "action": "create",
                    "message": "Creating a Point",
                    "properties": { "number": "123" },
                    "geometry": { "type": "Point", "coordinates": [ 0, 0 ] }
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

        { //Create MultiPoint
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "type": "Feature",
                    "action": "create",
                    "message": "Creating a MultiPoint",
                    "properties": { "number": "123" },
                    "geometry": { "type": "MultiPoint", "coordinates": [[ 0, 0 ], [ 1,1 ]] }
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert!(resp.status().is_success());
            assert_eq!(resp.text().unwrap(), "true");
        }

        {
            let resp = reqwest::get("http://localhost:8000/api/data/feature/2").unwrap();
            assert!(resp.status().is_success());
            //TODO check body
        }

        { //Create LineString
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "type": "Feature",
                    "action": "create",
                    "message": "Creating a LineString",
                    "properties": { "building": true },
                    "geometry": { "type": "LineString", "coordinates": [[ 0, 0 ], [ 1,1 ]] }
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert!(resp.status().is_success());
            assert_eq!(resp.text().unwrap(), "true");
        }

        {
            let resp = reqwest::get("http://localhost:8000/api/data/feature/3").unwrap();
            assert!(resp.status().is_success());
            //TODO check body
        }

        { //Create MultiLineString
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "type": "Feature",
                    "action": "create",
                    "message": "Creating a MultiLineString",
                    "properties": { "building": true },
                    "geometry": { "type": "MultiLineString", "coordinates": [[[ 0, 0 ], [ 1,1 ]], [[ 1,1 ], [ 2, 2 ]]] }
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert!(resp.status().is_success());
            assert_eq!(resp.text().unwrap(), "true");
        }

        {
            let resp = reqwest::get("http://localhost:8000/api/data/feature/4").unwrap();
            assert!(resp.status().is_success());
            //TODO check body
        }

        {
            let resp = reqwest::get("http://localhost:8000/api/data/features?bbox=-107.578125,-30.600094,56.162109,46.377254").unwrap();
            assert!(resp.status().is_success());
            //TODO check body
        }

        { //Restore Point - Feature Exists! (Should be right after create as there is a short circuit for newly created features that are attempted to be restored)
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "id": 1,
                    "type": "Feature",
                    "version": 1,
                    "action": "restore",
                    "message": "Restore previously deleted point",
                    "properties": { "number": "123" },
                    "geometry": { "type": "Point", "coordinates": [ 1, 1 ] }
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert_eq!(resp.text().unwrap(), "Restore Error: Feature id: 1 cannot restore an existing feature");
            assert!(resp.status().is_client_error());
        }

        { //Modify Point
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "id": 1,
                    "type": "Feature",
                    "version": 1,
                    "action": "modify",
                    "message": "Modify a point",
                    "properties": { "number": "123" },
                    "geometry": { "type": "Point", "coordinates": [ 1, 1 ] }
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

        { //Restore Point - Feature Exists! (Should be right after a create & >= 1 Modify - fails due to unique id check)
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "id": 1,
                    "type": "Feature",
                    "version": 2,
                    "action": "restore",
                    "message": "Restore previously deleted point",
                    "properties": { "number": "123" },
                    "geometry": { "type": "Point", "coordinates": [ 1, 1 ] }
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert_eq!(resp.text().unwrap(), "Restore Error: Feature id: 1 cannot restore an existing feature");
            assert!(resp.status().is_client_error());
        }

        { //Modify MultiPoint
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "id": 2,
                    "type": "Feature",
                    "version": 1,
                    "action": "modify",
                    "message": "Modify a MultiPoint",
                    "properties": { "number": "321" },
                    "geometry": { "type": "MultiPoint", "coordinates": [[ 1, 1 ], [ 0, 0 ]] }
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert!(resp.status().is_success());
            assert_eq!(resp.text().unwrap(), "true");
        }

        {
            let resp = reqwest::get("http://localhost:8000/api/data/feature/2").unwrap();
            assert!(resp.status().is_success());
            //TODO check body
        }

        { //Modify LineString
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "id": 3,
                    "type": "Feature",
                    "version": 1,
                    "action": "modify",
                    "message": "Modify a LineString",
                    "properties": { "building": false },
                    "geometry": { "type": "LineString", "coordinates": [[ 1, 1 ], [ 0, 0 ]] }
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert!(resp.status().is_success());
            assert_eq!(resp.text().unwrap(), "true");
        }

        {
            let resp = reqwest::get("http://localhost:8000/api/data/feature/3").unwrap();
            assert!(resp.status().is_success());
            //TODO check body
        }

        { //Modify MultiLineString
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "id": 4,
                    "type": "Feature",
                    "version": 1,
                    "action": "modify",
                    "message": "Modify a MultiLineString",
                    "properties": { "building": false },
                    "geometry": { "type": "MultiLineString", "coordinates": [[[ 1, 1 ], [ 0, 0 ]], [[ 2, 2 ], [ 1, 1 ]]] }
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert!(resp.status().is_success());
            assert_eq!(resp.text().unwrap(), "true");
        }

        {
            let resp = reqwest::get("http://localhost:8000/api/data/feature/4").unwrap();
            assert!(resp.status().is_success());
            //TODO check body
        }

        { //Delete Point
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "id": 1,
                    "type": "Feature",
                    "version": 2,
                    "action": "delete",
                    "message": "Delete a point",
                    "properties": null,
                    "geometry": null
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

        { //Delete MultiPoint
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "id": 2,
                    "type": "Feature",
                    "version": 2,
                    "action": "delete",
                    "message": "Delete a MultiPoint",
                    "properties": null,
                    "geometry": null
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert!(resp.status().is_success());
            assert_eq!(resp.text().unwrap(), "true");
        }

        {
            let resp = reqwest::get("http://localhost:8000/api/data/feature/2").unwrap();
            assert!(resp.status().is_client_error());
        }

        { //Delete LineString
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "id": 3,
                    "type": "Feature",
                    "version": 2,
                    "action": "delete",
                    "message": "Delete a LineString",
                    "properties": null,
                    "geometry": null
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert!(resp.status().is_success());
            assert_eq!(resp.text().unwrap(), "true");
        }

        {
            let resp = reqwest::get("http://localhost:8000/api/data/feature/3").unwrap();
            assert!(resp.status().is_client_error());
        }

        { //Delete MultiLineString
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "id": 4,
                    "type": "Feature",
                    "version": 2,
                    "action": "delete",
                    "message": "Delete a MultiLineString",
                    "properties": null,
                    "geometry": null
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert!(resp.status().is_success());
            assert_eq!(resp.text().unwrap(), "true");
        }

        {
            let resp = reqwest::get("http://localhost:8000/api/data/feature/4").unwrap();
            assert!(resp.status().is_client_error());
        }

        { //Restore Point - Error Wrong Version
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "id": 1,
                    "type": "Feature",
                    "version": 2,
                    "action": "restore",
                    "message": "Restore previously deleted point",
                    "properties": { "number": "123" },
                    "geometry": { "type": "Point", "coordinates": [ 1, 1 ] }
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert_eq!(resp.text().unwrap(), "Restore Version Mismatch");
            assert!(resp.status().is_client_error());
        }

        { //Restore Point - Feature Doesn't Exist
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "id": 1000,
                    "type": "Feature",
                    "version": 2,
                    "action": "restore",
                    "message": "Restore previously deleted point",
                    "properties": { "number": "123" },
                    "geometry": { "type": "Point", "coordinates": [ 1, 1 ] }
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::ContentType::json())
                .send()
                .unwrap();

            assert_eq!(resp.text().unwrap(), "Restore Error: Feature id: 1000 does not exist");
            assert!(resp.status().is_client_error());
        }


        { //Restore Point
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "id": 1,
                    "type": "Feature",
                    "version": 3,
                    "action": "restore",
                    "message": "Restore previously deleted point",
                    "properties": { "number": "123" },
                    "geometry": { "type": "Point", "coordinates": [ 1, 1 ] }
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
        }

        server.kill().unwrap();
    }
}
