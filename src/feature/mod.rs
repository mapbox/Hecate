extern crate r2d2;
extern crate r2d2_postgres;
extern crate geojson;
extern crate postgres;
extern crate serde_json;
extern crate valico;

use stream::PGStream;

#[derive(PartialEq, Debug)]
pub enum FeatureError {
    NotFound,
    NoProps,
    NoMembers,
    NoGeometry,
    VersionRequired,
    SchemaMisMatch,
    CreateError(String),
    DeleteVersionMismatch,
    DeleteError(String),
    ModifyError(String),
    RestoreError(String),
    ModifyVersionMismatch,
    RestoreVersionMismatch,
    IdRequired,
    ActionRequired,
    InvalidBBOX,
    InvalidFeature
}

#[derive(PartialEq, Debug)]
pub enum Action {
    Create,
    Modify,
    Delete,
    Restore
}

#[derive(PartialEq, Debug)]
pub struct Response {
    pub old: Option<i64>,
    pub new: Option<i64>,
    pub version: Option<i64>
}

impl FeatureError {
    pub fn to_string(&self) -> String {
        match *self {
            FeatureError::NotFound => String::from("Feature Not Found"),
            FeatureError::NoProps => String::from("No Properties"),
            FeatureError::NoMembers => String::from("No Members"),
            FeatureError::NoGeometry => String::from("No Geometry"),
            FeatureError::VersionRequired => String::from("Version Required"),
            FeatureError::SchemaMisMatch => String::from("Feature properties do not pass schema definition"),
            FeatureError::CreateError(ref msg) => format!("Create Error: {}", msg),
            FeatureError::DeleteVersionMismatch => String::from("Delete Version Mismatch"),
            FeatureError::DeleteError(ref msg) => format!("Delete Error: {}", msg),
            FeatureError::ModifyVersionMismatch => String::from("Modify Version Mismatch"),
            FeatureError::RestoreVersionMismatch => String::from("Restore Version Mismatch"),
            FeatureError::ModifyError(ref msg) => format!("Modify Error: {}", msg),
            FeatureError::RestoreError(ref msg) => format!("Restore Error: {}", msg),
            FeatureError::IdRequired => String::from( "ID Required"),
            FeatureError::ActionRequired => String::from( "Action Required"),
            FeatureError::InvalidBBOX => String::from( "Invalid BBOX"),
            FeatureError::InvalidFeature => String::from( "Invalid Feature")
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
                    Some("restore") => Ok(Action::Restore),
                    Some(_) => { return Err(FeatureError::ActionRequired); },
                    None => { return Err(FeatureError::ActionRequired); }
                }
            },
            None => { return Err(FeatureError::ActionRequired); },
        }
    }
}

pub fn get_key(feat: &geojson::Feature) -> Option<String> {
    match feat.foreign_members {
        None => None,
        Some(ref members) => {
            match members.get("key") {
                None => None,
                Some(key) => {
                    Some(key.to_string())
                }
            }
        }
    }
}

pub fn action(trans: &postgres::transaction::Transaction, schema_json: &Option<serde_json::value::Value>, feat: &geojson::Feature, delta: &Option<i64>) -> Result<Response, FeatureError> {
    let action = get_action(&feat)?;

    let mut scope = valico::json_schema::Scope::new();
    let schema = match schema_json {
        &Some(ref schema) => {
            Some(scope.compile_and_return(schema.clone(), false).unwrap())
        },
        &None => None
    };

    let res = match action {
        Action::Create => create(&trans, &schema, &feat, &delta)?,
        Action::Modify => modify(&trans, &schema, &feat, &delta)?,
        Action::Restore => restore(&trans, &schema, &feat, &delta)?,
        Action::Delete => delete(&trans, &feat)?
    };

    Ok(res)
}

pub fn create(trans: &postgres::transaction::Transaction, schema: &Option<valico::json_schema::schema::ScopedSchema>, feat: &geojson::Feature, delta: &Option<i64>) -> Result<Response, FeatureError> {
    let geom = match feat.geometry {
        None => { return Err(FeatureError::NoGeometry); },
        Some(ref geom) => geom
    };

    let props = match feat.properties {
        None => { return Err(FeatureError::NoProps); },
        Some(ref props) => props
    };

    let valid = match schema {
        &Some(ref schema) => {
            schema.validate(&json!(props)).is_valid()
        },
        &None => true
    };

    if !valid { return Err(FeatureError::SchemaMisMatch) };

    let geom_str = serde_json::to_string(&geom).unwrap();
    let props_str = serde_json::to_string(&props).unwrap();

    let key = get_key(&feat);

    match trans.query("
        INSERT INTO geo (version, geom, props, deltas, key)
            VALUES (
                1,
                ST_SetSRID(ST_GeomFromGeoJSON($1), 4326),
                $2::TEXT::JSON,
                array[COALESCE($3, currval('deltas_id_seq')::BIGINT)],
                $4
            ) RETURNING id;
    ", &[&geom_str, &props_str, &delta, &key]) {
        Ok(res) => Ok(Response {
            old: match feat.id {
                Some(ref id) => id.as_i64(),
                _ => None
            },
            new: Some(res.get(0).get(0)),
            version: Some(1)
        }),
        Err(err) => {
            match err.as_db() {
                Some(e) => { Err(FeatureError::CreateError(e.message.clone())) },
                _ => Err(FeatureError::CreateError(String::from("generic")))
            }
        }
    }
}

pub fn modify(trans: &postgres::transaction::Transaction, schema: &Option<valico::json_schema::schema::ScopedSchema>, feat: &geojson::Feature, delta: &Option<i64>) -> Result<Response, FeatureError> {
    let geom = match feat.geometry {
        None => { return Err(FeatureError::NoGeometry); },
        Some(ref geom) => geom
    };

    let props = match feat.properties {
        None => { return Err(FeatureError::NoProps); },
        Some(ref props) => props
    };

    let valid = match schema {
        &Some(ref schema) => {
            schema.validate(&json!(props)).is_valid()
        },
        &None => true
    };

    if !valid { return Err(FeatureError::SchemaMisMatch) };

    let id = get_id(&feat)?;
    let version = get_version(&feat)?;
    let key = get_key(&feat);

    let geom_str = serde_json::to_string(&geom).unwrap();
    let props_str = serde_json::to_string(&props).unwrap();

    match trans.query("SELECT modify_geo($1, $2, COALESCE($5, currval('deltas_id_seq')::BIGINT), $3, $4, $6);", &[&geom_str, &props_str, &id, &version, &delta, &key]) {
        Ok(_) => Ok(Response {
            old: Some(id),
            new: Some(id),
            version: Some(version + 1)
        }),
        Err(err) => {
            match err.as_db() {
                Some(e) => {
                    if e.message == "MODIFY: ID or VERSION Mismatch" {
                        Err(FeatureError::ModifyVersionMismatch)
                    } else {
                        Err(FeatureError::ModifyError(e.message.clone()))
                    }
                },
                _ => Err(FeatureError::ModifyError(String::from("generic")))
            }
        }
    }
}

pub fn delete(trans: &postgres::transaction::Transaction, feat: &geojson::Feature) -> Result<Response, FeatureError> {
    let id = get_id(&feat)?;
    let version = get_version(&feat)?;

    match trans.query("SELECT delete_geo($1, $2);", &[&id, &version]) {
        Ok(_) => Ok(Response {
            old: Some(id),
            new: None,
            version: None
        }),
        Err(err) => {
            match err.as_db() {
                Some(e) => {
                    if e.message == "DELETE: ID or VERSION Mismatch" {
                        Err(FeatureError::DeleteVersionMismatch)
                    } else {
                        Err(FeatureError::DeleteError(e.message.clone()))
                    }
                },
                _ => Err(FeatureError::DeleteError(String::from("generic")))
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
                key AS key,
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

pub fn restore(trans: &postgres::transaction::Transaction, schema: &Option<valico::json_schema::schema::ScopedSchema>, feat: &geojson::Feature, delta: &Option<i64>) -> Result<Response, FeatureError> {
    let geom = match feat.geometry {
        None => { return Err(FeatureError::NoGeometry); },
        Some(ref geom) => geom
    };

    let props = match feat.properties {
        None => { return Err(FeatureError::NoProps); },
        Some(ref props) => props
    };

    let valid = match schema {
        &Some(ref schema) => {
            schema.validate(&json!(props)).is_valid()
        },
        &None => true
    };

    if !valid { return Err(FeatureError::SchemaMisMatch) };

    let id = get_id(&feat)?;
    let version = get_version(&feat)?;
    let key = get_key(&feat);

    let geom_str = serde_json::to_string(&geom).unwrap();
    let props_str = serde_json::to_string(&props).unwrap();

    //Get the previous version of a given feature
    match trans.query("
        SELECT
            ARRAY_AGG(id ORDER BY id) AS delta_ids,
            MAX(feat->>'version')::BIGINT + 1 AS max_version
        FROM (
            SELECT
                deltas.id,
                JSON_Array_Elements((deltas.features -> 'features')::JSON) AS feat 
            FROM
                deltas
            WHERE
                affected @> ARRAY[$1]::BIGINT[]
            ORDER BY id DESC
        ) f
        WHERE
            (feat->>'id')::BIGINT = $1
        GROUP BY feat->>'id'
    ", &[&id]) {
        Ok(history) => {

            if history.len() != 1 {
                return Err(FeatureError::RestoreError(format!("Feature id: {} does not exist", &id)));
            }

            //Version will be None if the feature was created but has never been modified since the
            //original create does not need a version
            let prev_version: Option<i64> = history.get(0).get(1);
            match prev_version {
                None => {
                    return Err(FeatureError::RestoreError(format!("Feature id: {} cannot restore an existing feature", &id)));
                },
                Some(prev_version) => {
                    if prev_version != version {
                        return Err(FeatureError::RestoreVersionMismatch);
                    }
                }
            };

            let affected: Vec<i64> = history.get(0).get(0);

            //Create Delta History Array
            match trans.query("
                INSERT INTO geo (id, version, geom, props, deltas, key)
                    VALUES (
                        $1::BIGINT,
                        $2::BIGINT + 1,
                        ST_SetSRID(ST_GeomFromGeoJSON($3), 4326),
                        $4::TEXT::JSON,
                        array_append($5::BIGINT[], COALESCE($6, currval('deltas_id_seq')::BIGINT)),
                        $7
                    );
            ", &[&id, &prev_version, &geom_str, &props_str, &affected, &delta, &key]) {
                Ok(_) => Ok(Response {
                    old: Some(id),
                    new: Some(id),
                    version: Some(version + 1)
                }),
                Err(err) => {
                    match err.as_db() {
                        Some(e) => {
                            println!("{}", e.message);
                            if e.message == "duplicate key value violates unique constraint \"geo_id_key\"" {
                                Err(FeatureError::RestoreError(format!("Feature id: {} cannot restore an existing feature", &id)))
                            } else {
                                Err(FeatureError::RestoreError(String::from("generic")))
                            }
                        }
                        _ => Err(FeatureError::RestoreError(String::from("generic")))
                    }
                }
            }
        },
        Err(_) => {
            Err(FeatureError::RestoreError(format!("Error fetching feature history for: {}", &id)))
        }
    }
}

pub fn get_bbox_stream(conn: r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, bbox: Vec<f64>) -> Result<PGStream, FeatureError> {
    if bbox.len() != 4 {
        return Err(FeatureError::InvalidBBOX);
    }

    match PGStream::new(conn, String::from("next_features"), String::from(r#"
        DECLARE next_features CURSOR FOR
            SELECT
                row_to_json(f)::TEXT AS feature
            FROM (
                SELECT
                    id AS id,
                    key AS key,
                    'Feature' AS type,
                    version AS version,
                    ST_AsGeoJSON(geom)::JSON AS geometry,
                    props AS properties
                FROM geo
                WHERE
                    ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
                    OR ST_Within(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
            ) f;
    "#), &[&bbox[0], &bbox[1], &bbox[2], &bbox[3]]) {
        Ok(stream) => Ok(stream),
        Err(_) => Err(FeatureError::NotFound)
    }
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
                key AS key,
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
            _ => { return Err(FeatureError::InvalidFeature); }
        };

        fc.features.push(feat);
    }

    Ok(fc)
}
