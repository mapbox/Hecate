extern crate r2d2;
extern crate r2d2_postgres;
extern crate geojson;
extern crate postgres;
extern crate serde_json;

use self::geojson::Feature;

pub enum FeatureError {
    NotFoundError,
    NoGeometryError,
    NoPropsError
}

impl FeatureError {
    pub fn to_string(&self) -> &str {
        match (&self) {
            NotFoundError => {
                "Geometry Not Found For Given ID"
            },
            NoGeometryError => {
                "Null or Invalid Geometry"
            },
            NoPropsError => {
                "Null or Invalid Properties"
            }
        }
    }
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
    let props_str = serde_json::to_string(&props).unwrap();

    conn.execute("
        INSERT INTO geo (geom, props)
            VALUES (
                ST_SetSRID(ST_GeomFromGeoJSON($1), 4326),
                TO_JSON($2)
            );
    ", &[&geom_str, &props_str]).unwrap();

    Ok(true)
}

pub fn get(conn: r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, id: &i64) -> Result<(), FeatureError> {
    let res = conn.query("SELECT ST_AsGeoJSON(geom) FROM geo WHERE id = $1;", &[&id]).unwrap();

    if res.len() != 1 {
        return Err(FeatureError::NotFoundError);
    }

    let res = res.get(0);

    println!("{:?}", res);

    Ok(())
}


