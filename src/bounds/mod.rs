extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;
extern crate std;
extern crate rocket;
extern crate serde_json;

use stream::PGStream;

#[derive(PartialEq, Debug)]
pub enum BoundsError {
    NotFound,
    DeleteError(String),
    SetError(String),
    ListError(String),
    GetError(String)
}

impl BoundsError {
    pub fn to_string(&self) -> String {
        match *self {
            BoundsError::NotFound => String::from("Bounds Not Found"),
            BoundsError::DeleteError(ref msg) => String::from(format!("Could not delete bounds: {}", msg)),
            BoundsError::SetError(ref msg) => String::from(format!("Could not set bounds: {}", msg)),
            BoundsError::ListError(ref msg) => String::from(format!("Could not list bounds: {}", msg)),
            BoundsError::GetError(ref msg) => String::from(format!("Could not get bounds: {}", msg))
        }
    }
}

pub fn set(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, name: &String, feat: &serde_json::Value) -> Result<bool, BoundsError> {
    match conn.execute("
        INSERT INTO bounds (name, geom) VALUES ($1 , ST_Multi(ST_SetSRID(ST_GeomFromGeoJSON($2::JSON->>'geometry'), 4326)))
            ON CONFLICT (name) DO
                UPDATE
                    SET geom = ST_Multi(ST_SetSRID(ST_GeomFromGeoJSON($2::JSON->>'geometry'), 4326))
                    WHERE bounds.name = $1;
    ", &[ &name, &feat ]) {
        Ok(_) => Ok(true),
        Err(err) => {
            match err.as_db() {
                Some(e) => { Err(BoundsError::SetError(e.message.clone())) },
                _ => Err(BoundsError::SetError(String::from("generic")))
            }
        }
    }
}

pub fn delete(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, name: &String) -> Result<bool, BoundsError> {
    match conn.execute("
        DELETE FROM bounds WHERE name = $1
    ", &[ &name ]) {
        Ok(_) => Ok(true),
        Err(err) => {
            match err.as_db() {
                Some(e) => { Err(BoundsError::DeleteError(e.message.clone())) },
                _ => Err(BoundsError::DeleteError(String::from("generic")))
            }
        }
    }
}

pub fn filter(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, prefix: &String, limit: &Option<i16>) -> Result<Vec<String>, BoundsError> {
    let limit: i16 = match limit {
        None => 100,
        Some(limit) => if *limit > 100 { 100 } else { *limit }
    };

    match conn.query("
        SELECT name
            FROM bounds
            WHERE name iLIKE $1||'%'
            ORDER BY name
            LIMIT $2::SmallInt
    ", &[ &prefix, &limit ]) {
        Ok(rows) => {
            let mut names = Vec::<String>::new();

            for row in rows.iter() {
                names.push(row.get(0));
            }

            Ok(names)
        },
        Err(err) => {
            match err.as_db() {
                Some(e) => { Err(BoundsError::ListError(e.message.clone())) },
                _ => Err(BoundsError::ListError(String::from("generic")))
            }
        }
    }
}

pub fn list(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, limit: &Option<i16>) -> Result<Vec<String>, BoundsError> {
    match conn.query("
        SELECT name
        FROM bounds
        ORDER BY name
        LIMIT $1::SmallInt
    ", &[ &limit ]) {
        Ok(rows) => {
            let mut names = Vec::<String>::new();

            for row in rows.iter() {
                names.push(row.get(0));
            }

            Ok(names)
        },
        Err(err) => {
            match err.as_db() {
                Some(e) => { Err(BoundsError::ListError(e.message.clone())) },
                _ => Err(BoundsError::ListError(String::from("generic")))
            }
        }
    }
}

pub fn get(conn: r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, bounds: String) -> Result<PGStream, BoundsError> {
    match PGStream::new(conn, String::from("next_bounds"), String::from(r#"
        DECLARE next_bounds CURSOR FOR
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
                    geo,
                    bounds
                WHERE
                    bounds.name = $1
                    AND ST_Intersects(geo.geom, bounds.geom)
            ) t
    "#), &[&bounds]) {
        Ok(stream) => Ok(stream),
        Err(_) =>  Err(BoundsError::GetError(String::from("db error")))
    }
}

pub fn stats_json(conn: r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, bounds: String) -> Result<serde_json::Value, BoundsError> {
    match conn.query("
        SELECT
            row_to_json(t)
        FROM (
            SELECT
                count(*) AS total,
                json_build_array(
                    ST_XMin(ST_Extent(bounds.geom)),
                    ST_YMin(ST_Extent(bounds.geom)),
                    ST_XMax(ST_Extent(bounds.geom)),
                    ST_YMax(ST_Extent(bounds.geom))
                ) AS bbox,
                to_char(now(), 'YYYY-MM-DD HH24:MI:SS') AS last_calc
            FROM
                geo,
                bounds
            WHERE
                bounds.name = $1
                AND ST_Intersects(geo.geom, bounds.geom)
        ) t
    ", &[ &bounds ]) {
        Ok(rows) => Ok(rows.get(0).get(0)),
        Err(err) => {
            match err.as_db() {
                Some(e) => { Err(BoundsError::ListError(e.message.clone())) },
                _ => Err(BoundsError::ListError(String::from("generic")))
            }
        }
    }
}
