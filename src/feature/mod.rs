extern crate r2d2;
extern crate r2d2_postgres;
extern crate geojson;
extern crate postgres;
extern crate serde_json;

#[derive(PartialEq)]
pub enum FeatureError {
    NotFound,
    NoProps,
    NoMembers,
    NoGeometry,
    VersionRequired,
    PutError,
    DeleteVersionMismatch,
    DeleteError,
    PatchError,
    PatchVersionMismatch,
    IdRequired,
    ActionRequired,
    InvalidBBOX,
    InvalidFeature
}

#[derive(PartialEq)]
pub enum Action {
    Create,
    Modify,
    Delete
}

impl FeatureError {
    pub fn to_string(&self) -> &str {
        match *self {
            FeatureError::NotFound => { "Feature Not Found" },
            FeatureError::NoProps => { "No Properties" },
            FeatureError::NoMembers => { "No Members" },
            FeatureError::NoGeometry => { "No Geometry" },
            FeatureError::VersionRequired => { "Version Required" },
            FeatureError::PutError => { "Put Error" },
            FeatureError::DeleteVersionMismatch => { "Delete Version Mismatch" },
            FeatureError::DeleteError => { "Internal Delete Error" },
            FeatureError::PatchVersionMismatch => { "Patch Version Mismatch" },
            FeatureError::PatchError => { "Internal Patch Error" },
            FeatureError::IdRequired => { "ID Required" }
            FeatureError::ActionRequired => { "Action Required" },
            FeatureError::InvalidBBOX => { "Invalid BBOX" },
            FeatureError::InvalidFeature => { "Invalid Feature" },
            _ => { "Generic FeatureError" }
        }
    }
}

pub fn get_version(feat: &geojson::Feature) -> Result<i64, FeatureError> {
    match feat.foreign_members {
        None => { return Err(FeatureError::VersionRequired); },
        Some(ref members) => match members.get("version") {
            Some(version) => {
                match version.as_i64() {
                    Some(version) => Ok(version),
                    None => { return Err(FeatureError::VersionRequired); },
                }
            },
            None => { return Err(FeatureError::VersionRequired); },
        }
    }
}

pub fn get_id(feat: &geojson::Feature) -> Result<i64, FeatureError> {
    match feat.id {
        None => { return Err(FeatureError::IdRequired); },
        Some(ref id) => match id.as_i64() {
            Some(id) => Ok(id),
            None => { return Err(FeatureError::IdRequired); },
        }
    }
}

pub fn get_action(feat: &geojson::Feature) -> Result<Action, FeatureError> {
    match feat.foreign_members {
        None => { return Err(FeatureError::ActionRequired); },
        Some(ref members) => match members.get("action") {
            Some(action) => {
                match action.as_str() {
                    Some("create") => Ok(Action::Create),
                    Some("modify") => Ok(Action::Modify),
                    Some("delete") => Ok(Action::Delete),
                    Some(_) => { return Err(FeatureError::ActionRequired); },
                    None => { return Err(FeatureError::ActionRequired); }
                }
            },
            None => { return Err(FeatureError::ActionRequired); },
        }
    }
}

pub fn action(trans: &postgres::transaction::Transaction, feat: geojson::Feature, delta: &i64) -> Result<bool, FeatureError> {
    match get_action(&feat)? {
        Action::Create => { put(&trans, &feat)?; },
        Action::Modify => { patch(&trans, &feat)?; },
        Action::Delete => { delete(&trans, &feat, &delta)?; }
    }

    Ok(true)
}

pub fn put(trans: &postgres::transaction::Transaction, feat: &geojson::Feature) -> Result<bool, FeatureError> {
    let geom = match feat.geometry {
        None => { return Err(FeatureError::NoGeometry); },
        Some(ref geom) => geom
    };

    let props = match feat.properties {
        None => { return Err(FeatureError::NoProps); },
        Some(ref props) => props
    };

    let geom_str = serde_json::to_string(&geom).unwrap();
    let props_str = serde_json::to_string(&props).unwrap();

    match trans.execute("
        INSERT INTO geo (version, geom, props, deltas)
            VALUES (
                1,
                ST_SetSRID(ST_GeomFromGeoJSON($1), 4326),
                $2::TEXT::JSON,
                array[currval('deltas_id_seq')::BIGINT]
            );
    ", &[&geom_str, &props_str]) {
        Ok(_) => Ok(true),
        Err(err) => {
            match err.as_db() {
                Some(e) => {
                    println!("{:?}", e);
                    Err(FeatureError::PutError)
                },
                _ => Err(FeatureError::PutError)
            }
        }
    }
}

pub fn patch(trans: &postgres::transaction::Transaction, feat: &geojson::Feature) -> Result<bool, FeatureError> {
    let geom = match feat.geometry {
        None => { return Err(FeatureError::NoGeometry); },
        Some(ref geom) => geom
    };

    let props = match feat.properties {
        None => { return Err(FeatureError::NoProps); },
        Some(ref props) => props
    };

    let id = get_id(&feat)?;
    let version = get_version(&feat)?;

    let geom_str = serde_json::to_string(&geom).unwrap();
    let props_str = serde_json::to_string(&props).unwrap();

    let delta = 1;

    match trans.query("SELECT patch_geo($1, $2, $3, $4, $5);", &[&geom_str, &props_str, &delta, &id, &version]) {
        Ok(_) => Ok(true),
        Err(err) => {
            match err.as_db() {
                Some(e) => {
                    if e.message == "DELETE: ID or VERSION Mismatch" {
                        Err(FeatureError::PatchVersionMismatch)
                    } else {
                        println!("{:?}", e);
                        Err(FeatureError::PatchError)
                    }
                },
                _ => Err(FeatureError::PatchError)
            }
        }
    }
}

pub fn delete(trans: &postgres::transaction::Transaction, feat: &geojson::Feature, delta: &i64) -> Result<bool, FeatureError> {
    let id = get_id(&feat)?;
    let version = get_version(&feat)?;

    match trans.query("SELECT delete_geo($1, $2);", &[&id, &version]) {
        Ok(_) => Ok(true),
        Err(err) => {
            match err.as_db() {
                Some(e) => {
                    if e.message == "DELETE: ID or VERSION Mismatch" {
                        Err(FeatureError::DeleteVersionMismatch)
                    } else {
                        Err(FeatureError::DeleteError)
                    }
                },
                _ => Err(FeatureError::DeleteError)
            }
        }
    }
}

pub fn get(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, id: &i64) -> Result<geojson::Feature, FeatureError> {
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
            WHERE id = $1
        ) f;
    ", &[&id]).unwrap();

    if res.len() != 1 { return Err(FeatureError::NotFound); }

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
