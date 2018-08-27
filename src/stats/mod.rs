extern crate serde_json;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate chrono;

#[derive(PartialEq, Debug)]
pub enum StatsError {
    GetFail
}

impl StatsError {
    pub fn to_string(&self) -> String {
        match *self {
            StatsError::GetFail => { String::from("Stats Get Failed") }
        }
    }
}

pub fn get_json(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>) -> Result<serde_json::Value, StatsError> {
    match conn.query("
        SELECT COALESCE(row_to_json(d), 'false'::JSON)
        FROM (
            SELECT
                json_build_array(
                    ST_XMin(extent.extent),
                    ST_YMin(extent.extent),
                    ST_XMax(extent.extent),
                    ST_YMax(extent.extent)
                ) AS bbox,
                total.total AS total
            FROM
                (
                    SELECT
                        ST_EstimatedExtent('geo', 'geom') AS extent
                ) as extent,
                (
                    SELECT
                        pg_class.reltuples::bigint as total
                    FROM
                        pg_class
                    WHERE
                        oid = 'public.geo'::regclass
                ) as total
        ) d;
    ", &[]) {
        Err(err) => {
            match err.as_db() {
                Some(_e) => { Err(StatsError::GetFail) },
                _ => Err(StatsError::GetFail)
            }
        },
        Ok(res) => {
            let d_json: serde_json::Value = res.get(0).get(0);
            Ok(d_json)
        }
    }
}
