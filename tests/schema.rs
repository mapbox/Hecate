extern crate reqwest;
extern crate postgres;

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::env;
    use std::io::prelude::*;
    use postgres::{Connection, TlsMode};
    use std::process::Command;
    use std::time::Duration;
    use std::thread;
    use reqwest;

    #[test]
    fn schema() {
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

        let mut server = Command::new("cargo").args(&[
            "run",
            "--",
            "--schema", env::current_dir().unwrap().join("tests/fixtures/source_schema.json").to_str().unwrap()
        ]).spawn().unwrap();
        thread::sleep(Duration::from_secs(1));

        { // Create User
            let conn = Connection::connect("postgres://postgres@localhost:5432/hecate", TlsMode::None).unwrap();
            conn.execute("
                INSERT INTO users (username, password, email)
                    VALUES ('ingalls', crypt('yeaheh', gen_salt('bf', 10)), 'ingalls@protonmail.com')
            ", &[]).unwrap();
        }

        { //Get Schema in use
            let mut resp = reqwest::get("http://localhost:8000/api/schema").unwrap();
            assert_eq!(resp.text().unwrap(), "{\"$schema\":\"http://json-schema.org/draft-04/schema#\",\"description\":\"Validate addresses source\",\"properties\":{\"number\":{\"description\":\"Number of the building.\",\"type\":[\"string\",\"number\"]},\"source\":{\"description\":\"Name of the source where the data comes from\",\"type\":\"string\"},\"street\":{\"description\":\"Name Array of street names to which this address belongs\",\"items\":{\"properties\":{\"display\":{\"description\":\"Single name string of a potential road name\",\"type\":\"string\"},\"priority\":{\"description\":\"Used to determine the primary name of a feature\",\"type\":\"number\"}},\"required\":[\"display\"],\"type\":\"object\"},\"type\":\"array\"}},\"required\":[\"source\",\"number\",\"street\"],\"title\":\"Address source\",\"type\":\"object\"}");
            assert!(resp.status().is_success());
        }

        { //Create Point Failing Schema Validation
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
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            assert!(resp.status().is_client_error());
            assert_eq!(resp.text().unwrap(), "{\"feature\":{\"action\":\"create\",\"geometry\":{\"coordinates\":[0.0,0.0],\"type\":\"Point\"},\"message\":\"Creating a Point\",\"properties\":{\"number\":\"123\"},\"type\":\"Feature\"},\"id\":null,\"message\":\"Failed to Match Schema\"}");
        }

        { //Create Point Almost Passing Schema Validation
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "type": "Feature",
                    "action": "create",
                    "message": "Creating a Point",
                    "properties": {
                        "source": "Test Data",
                        "number": "123",
                        "street": [{ "test": "123" }]
                    },
                    "geometry": { "type": "Point", "coordinates": [ 0, 0 ] }
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            assert!(resp.status().is_client_error());
            assert_eq!(resp.text().unwrap(), "{\"feature\":{\"action\":\"create\",\"geometry\":{\"coordinates\":[0.0,0.0],\"type\":\"Point\"},\"message\":\"Creating a Point\",\"properties\":{\"number\":\"123\",\"source\":\"Test Data\",\"street\":[{\"test\":\"123\"}]},\"type\":\"Feature\"},\"id\":null,\"message\":\"Failed to Match Schema\"}");
        }

        { //Create Point Passing Schema Validation
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/data/feature")
                .body(r#"{
                    "type": "Feature",
                    "action": "create",
                    "message": "Creating a Point",
                    "properties": {
                        "source": "Test Data",
                        "number": "123",
                        "street": [{ "display": "Main Street" }]
                    },
                    "geometry": { "type": "Point", "coordinates": [ 0, 0 ] }
                }"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send()
                .unwrap();

            assert!(resp.status().is_success());
            assert_eq!(resp.text().unwrap(), "true");
        }

        { //XML Changeset Create (Node Create)
            let client = reqwest::Client::new();
            let mut resp = client.put("http://localhost:8000/api/0.6/changeset/create")
                .body(r#"<osm><changeset><tag k="created_by" v="Hecate Server"/><tag k="comment" v="Buncho Random Text"/></changeset></osm>"#)
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();
            assert_eq!(resp.text().unwrap(), "4");
            assert!(resp.status().is_success());
        }

        { //XML Node Create Failure
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/0.6/changeset/4/upload")
                .body(r#"
                    <osmChange version="0.6" generator="Hecate Server">
                        <create>
                            <node id='-1' version='1' changeset='1' lat='0' lon='0'>
                                <tag k='source' v='Test Data' />
                                <tag k='number' v='123' />
                                <tag k='street' v='[{ "test": "123" }]'/>
                            </node>
                        </create>
                    </osmChange>
                "#)
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();
            assert_eq!(resp.text().unwrap(), "{\"feature\":{\"action\":\"create\",\"geometry\":{\"coordinates\":[0.0,0.0],\"type\":\"Point\"},\"id\":-1,\"properties\":{\"number\":123,\"source\":\"Test Data\",\"street\":[{\"test\":\"123\"}]},\"type\":\"Feature\"},\"id\":-1,\"message\":\"Failed to Match Schema\"}");
            assert!(resp.status().is_client_error());
        }

        { //XML Node Create Failure
            let client = reqwest::Client::new();
            let mut resp = client.post("http://localhost:8000/api/0.6/changeset/4/upload")
                .body(r#"
                    <osmChange version="0.6" generator="Hecate Server">
                        <create>
                            <node id='-1' version='1' changeset='1' lat='0' lon='0'>
                                <tag k='source' v='Test Data' />
                                <tag k='number' v='123' />
                                <tag k='street' v='[{ "display": "Main Street" }]'/>
                            </node>
                        </create>
                    </osmChange>
                "#)
                .basic_auth("ingalls", Some("yeaheh"))
                .send()
                .unwrap();
            assert_eq!(resp.text().unwrap(), "<diffResult generator=\"Hecate Server\" version=\"0.6\"><node old_id=\"-1\" new_id=\"2\" new_version=\"1\"/></diffResult>");
            assert!(resp.status().is_success());
        }

        server.kill().unwrap();
    }
}
