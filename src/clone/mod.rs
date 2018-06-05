extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;
extern crate std;
extern crate rocket;

use stream::PGStream;

#[derive(PartialEq, Debug)]
pub enum CloneError {
    GetError
}

impl CloneError {
    pub fn to_string(&self) -> String {
        match *self {
            CloneError::GetError => String::from("Failed to clone")
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

pub fn query(read_conn: &str, query: &String) -> Result<PGStream, CloneError> {
    let conn = postgres::Connection::connect(read_conn, postgres::TlsMode::None).unwrap();

    /*
    match PGStream::new(conn, String::from("next_clone_query"), format!(r#"

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
    */

    Err(CloneError::GetError)
}
