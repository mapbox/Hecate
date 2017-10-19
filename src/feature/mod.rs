extern crate r2d2;
extern crate r2d2_postgres;
extern crate geojson;
extern crate postgres;
extern crate serde_json;

use self::geojson::Feature;

pub enum FeatureError {
    NoGeometryError,
    NoPropsError
}

pub fn put(conn: r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, feat: Feature) -> Result<bool, FeatureError> {
    let geom = match feat.geometry {
        None => {
            return Err(FeatureError::NoGeometryError);
        },
        Some(geom) => {
            geom
        }
    };

    let props = match feat.properties {
        None => {
            return Err(FeatureError::NoPropsError);
        },
        Some(props) => {
            props
        }
    };

    let geom_str = serde_json::to_string(&geom).unwrap();

    conn.execute("
        INSERT INTO geo (geom) VALUES (ST_SetSRID(ST_GeomFromGeoJSON($1), 4326));
    ", &[&geom_str]).unwrap();

    Ok(true)
}
