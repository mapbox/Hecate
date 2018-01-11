extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;
extern crate std;
extern crate rocket;

#[derive(PartialEq, Debug)]
pub enum BoundsError {
    NotFound,
    ListError(String),
    GetError(String)
}

impl BoundsError {
    pub fn to_string(&self) -> String {
        match *self {
            BoundsError::NotFound => String::from("User Not Found"),
            BoundsError::ListError(ref msg) => String::from(format!("Could not list bounds: {}", msg)),
            BoundsError::GetError(ref msg) => String::from(format!("Could not get bounds: {}", msg))
        }
    }
}

pub fn list(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>) -> Result<Vec<String>, BoundsError> {
    match conn.query("
        SELECT name FROM bounds;
    ", &[ ]) {
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

pub fn get_query() -> &'static str {
    "SELECT row_to_json(t)
        FROM (
            SELECT
                geo.id AS id,
                'Feature' AS type,
                geo.version AS version,
                ST_AsGeoJSON(geo.geom)::JSON AS geometry,
                geo.props AS properties
            FROM
                geo,
                bounds
            WHERE
                bounds.name = 'dc'
                AND ST_Intersects(geo.geom, bounds.geom)
            ORDER BY id
        ) t"
}
