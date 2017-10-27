extern crate r2d2;
extern crate r2d2_postgres;
extern crate geojson;
extern crate postgres;
extern crate serde_json;

pub enum FeatureError {
    NotFound,
    NoGeometry,
    InvalidBBOX,
    InvalidFeature,
    NoProps
}

impl FeatureError {
    pub fn to_string(&self) -> &str {
        match &self {
            NoGeometry => {
                "Null or Invalid Geometry"
            },
            NoProps => {
                "Null or Invalid Properties"
            },
            InvalidBBox => {
                "Invalid Bounding Box"
            },
            InvalidFeature => {
                "Could not parse Feature - Feature is invalid"
            },
            NotFound => {
                "Geometry Not Found For Given ID"
            },
        }
    }
}

pub fn action(trans: &postgres::transaction::Transaction, feat: geojson::Feature, hash: &i64) -> Result<bool, FeatureError> {

    Ok(true)
}

pub fn put(trans: &postgres::transaction::Transaction, feat: geojson::Feature, hash: &i64) -> Result<bool, FeatureError> {
    let geom = match feat.geometry {
        None => { return Err(FeatureError::NoGeometry); },
        Some(geom) => geom
    };

    let props = match feat.properties {
        None => { return Err(FeatureError::NoProps); },
        Some(props) => props
    };

    let geom_str = serde_json::to_string(&geom).unwrap();
    let props_str = serde_json::to_string(&props).unwrap();

    trans.execute("
        INSERT INTO geo (version, geom, props, hashes)
            VALUES (
                1,
                ST_SetSRID(ST_GeomFromGeoJSON($1), 4326),
                $2::TEXT::JSON,
                array[$3::BIGINT]
            );
    ", &[&geom_str, &props_str, &hash]).unwrap();

    Ok(true)
}

pub fn patch(trans: &postgres::transaction::Transaction, feat: geojson::Feature, hash: &i64) -> Result<bool, FeatureError> {
    let geom = match feat.geometry {
        None => { return Err(FeatureError::NoGeometry); },
        Some(geom) => geom
    };

    let props = match feat.properties {
        None => { return Err(FeatureError::NoProps); },
        Some(props) => props
    };

    let geom_str = serde_json::to_string(&geom).unwrap();
    let props_str = serde_json::to_string(&props).unwrap();

    trans.execute("
        UPDATE geo
            SET
                version = 1,
                geom = ST_SetSRID(ST_GeomFromGeoJSON($1), 4326),
                props = $2::TEXT::JSON,
                hashes = array_append(hashes, $3::BIGINT)
    ", &[&geom_str, &props_str, &hash]).unwrap();

    Ok(true)
}

pub fn get(trans: &postgres::transaction::Transaction>, id: &i64) -> Result<geojson::Feature, FeatureError> {
    let res = trans.query("
        SELECT
            row_to_json(f)::TEXT AS feature
        FROM (
            SELECT
                id AS id,
                'Feature' AS type,
                version AS version,
                ST_AsGeoJSON(geom)::JSON AS geometry,
                props AS properties
            FROM geo
            WHERE id = $1
        ) f;
    ", &[&id]).unwrap();

    if res.len() != 1 {
        return Err(FeatureError::NotFound);
    }

    let feat: postgres::rows::Row = res.get(0);
    let feat: String = feat.get(0);
    let feat: geojson::Feature = match feat.parse() {
        Ok(feat) => match feat {
            geojson::GeoJson::Feature(feat) => feat,
            _ => { return Err(FeatureError::InvalidFeature); }
        },
        Err(_) => { return Err(FeatureError::InvalidFeature); }
    };

    Ok(feat)
}

pub fn delete(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, id: &i64) -> Result<bool, FeatureError> {
    conn.query("
        DELETE FROM geo WHERE id = $1;
    ", &[&id]).unwrap();

    Ok(true)
}

pub fn get_bbox(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, bbox: Vec<f64>) -> Result<geojson::FeatureCollection, FeatureError> {
    if bbox.len() != 4 {
        return Err(FeatureError::InvalidBBOX);
    }

    let res = conn.query("
        SELECT
            row_to_json(f)::TEXT AS feature
        FROM (
            SELECT
                id AS id,
                'Feature' AS type,
                version AS version,
                ST_AsGeoJSON(geom)::JSON AS geometry,
                props AS properties
            FROM geo
            WHERE
                ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
                OR ST_Within(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
        ) f;
    ", &[&bbox[0], &bbox[1], &bbox[2], &bbox[3]]).unwrap();

    let mut fc = geojson::FeatureCollection {
        bbox: None,
        features: vec![],
        foreign_members: None
    };

    for row in res.iter() {
        let feat: String = row.get(0);
        let feat: geojson::Feature = match feat.parse().unwrap() {
            geojson::GeoJson::Feature(feat) => feat,
            _ => {
                return Err(FeatureError::InvalidFeature);
            }
        };

        fc.features.push(feat);
    }

    Ok(fc)
}
