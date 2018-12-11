use stream::PGStream;
use err::HecateError;

pub fn get(conn: r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>) -> Result<PGStream, HecateError> {
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
        Err(err) =>  Err(err)
    }
}

pub fn query(read_conn: r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, query: &String, limit: &Option<i64>) -> Result<PGStream, HecateError> {
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
