extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;
extern crate std;
extern crate rocket;
extern crate serde_json;

use stream::PGStream;
use serde_json::value::Value;
use rocket::response::content::Json;
use rocket::response::status;

#[derive(PartialEq, Debug)]
pub enum CloneError {
    GetError,
    QueryError(Value)
}

impl CloneError {
    pub fn as_json(&self) -> Value {
        match *self {
            CloneError::GetError => json!("Failed to clone"),
            CloneError::QueryError(ref value) => json!(value)
        }
    }
}

pub fn get(conn: r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>) -> Result<PGStream, CloneError> {
    match PGStream::new(conn, String::from("next_clone"), String::from(r#"
        DECLARE next_clone CURSOR FOR
            SELECT
                row_to_json(t)::TEXT
            FROM (
                SELECT
                    geo.id AS id,
                    geo.key AS key,
                    'Feature' AS type,
                    geo.version AS version,
                    ST_AsGeoJSON(geo.geom)::JSON AS geometry,
                    geo.props AS properties
                FROM
                    geo
            ) t
    "#), &[]) {
        Ok(stream) => Ok(stream),
        Err(_) =>  Err(CloneError::GetError)
    }
}

pub fn query(read_conn: r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, query: &String, limit: &Option<i64>) -> Result<PGStream, status::Custom<Json>> {
    Ok(PGStream::new(read_conn, String::from("next_clone_query"), format!(r#"
        DECLARE next_clone_query CURSOR FOR
            SELECT
                row_to_json(t)::TEXT
            FROM (
                {}
            ) t
            LIMIT $1

    "#, query), &[&limit])?)
}
