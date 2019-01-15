use stream::PGStream;
use err::HecateError;

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

pub fn import_error(feat: &geojson::Feature, error: &str) -> HecateError {
    HecateError::from_json(400, json!({
        "id": &feat.id,
        "message": error,
        "feature": &feat
    }), String::from("Import Error"), None)
}

///
/// Check if the feature has the force: true flag set and if so
/// validate that it meets the force:true acceptions
///
pub fn is_force(feat: &geojson::Feature) -> Result<bool, HecateError> {
    match feat.foreign_members {
        None => Ok(false),
        Some(ref members) => match members.get("force") {
            Some(force) => {
                match force.as_bool() {
                    Some(true) => {
                        if get_action(&feat)? != Action::Create {
                            return Err(import_error(&feat, "force can only be used on create"));
                        }

                        match get_key(&feat)? {
                            None => {
                                Err(import_error(&feat, "force can only be used with a key value"))
                            },
                            Some(_) => {
                                Ok(true)
                            }
                        }
                    },
                    Some(false) => Ok(false),
                    None => Err(import_error(&feat, "force must be a boolean"))
                }
            },
            None => Ok(false)
        }
    }
}

pub fn del_version(feat: &mut geojson::Feature) {
    match feat.foreign_members {
        None => (),
        Some(ref mut members) => {
            members.remove("version");
        }
    }
}

pub fn get_version(feat: &geojson::Feature) -> Result<i64, HecateError> {
    match feat.foreign_members {
        None => { return Err(import_error(&feat, "Version Required")); },
        Some(ref members) => match members.get("version") {
            Some(version) => {
                match version.as_i64() {
                    Some(version) => Ok(version),
                    None => { return Err(import_error(&feat, "Version Required")); },
                }
            },
            None => { return Err(import_error(&feat, "Version Required")); },
        }
    }
}

pub fn get_geom_str(feat: &geojson::Feature) -> Result<String, HecateError> {
    let geom = match feat.geometry {
        None => { return Err(import_error(&feat, "Geometry Required")); },
        Some(ref geom) => geom
    };

    fn is_valid_coord(feat: &geojson::Feature, pt: &geojson::PointType) -> Result<bool, HecateError> {
        if pt.len() > 3 {
            return Err(import_error(&feat, "Coordinate Array has > 3 coords"));
        }

        let lon = pt[0];
        let lat = pt[1];

        if lon < -180.0 {
            return Err(import_error(&feat, "longitude < -180"));
        } else if lon > 180.0 {
            return Err(import_error(&feat, "longitude > 180"));
        } else if lat < -90.0 {
            return Err(import_error(&feat, "latitude < -90"));
        } else if lat > 90.0 {
            return Err(import_error(&feat, "latitude > 90"));
        } else {
            Ok(true)
        }
    }

    //Ensure coordinates are within bounds
    match geom.value {
        geojson::Value::Point(ref pt) => {
            is_valid_coord(&feat, &pt)?;
        },
        geojson::Value::MultiPoint(ref pts) => {
            for pt in pts {
                is_valid_coord(&feat, &pt)?;
            }
        },
        geojson::Value::LineString(ref ln) => {
            for pt in ln {
                is_valid_coord(&feat, &pt)?;
            }
        },
        geojson::Value::MultiLineString(ref lns) => {
            for ln in lns {
                for pt in ln {
                    is_valid_coord(&feat, &pt)?;
                }
            }
        },
        geojson::Value::Polygon(ref ply) => {
            for pl in ply {
                for pt in pl {
                    is_valid_coord(&feat, &pt)?;
                }
            }
        },
        geojson::Value::MultiPolygon(ref plys) => {
            for ply in plys {
                for pl in ply {
                    for pt in pl {
                        is_valid_coord(&feat, &pt)?;
                    }
                }
            }
        },
        _ => ()
    };

    match serde_json::to_string(&geom) {
        Ok(geom) => Ok(geom),
        Err(_) => Err(import_error(&feat, "Failed to stringify geometry"))
    }
}

pub fn get_id(feat: &geojson::Feature) -> Result<i64, HecateError> {
    match feat.id {
        None => { return Err(import_error(&feat, "ID Required")); },
        Some(ref id) => match id {
            geojson::feature::Id::Number(id) => {
                if id.is_i64() {
                    Ok(id.as_i64().unwrap())
                } else {
                    return Err(import_error(&feat, "Integer ID Required"));
                }
            },
            _ => { return Err(import_error(&feat, "Integer ID Required")); },
        }
    }
}

pub fn get_action(feat: &geojson::Feature) -> Result<Action, HecateError> {
    match feat.foreign_members {
        None => { return Err(import_error(&feat, "Action Required")); },
        Some(ref members) => match members.get("action") {
            Some(action) => {
                match action.as_str() {
                    Some("create") => Ok(Action::Create),
                    Some("modify") => Ok(Action::Modify),
                    Some("delete") => Ok(Action::Delete),
                    Some("restore") => Ok(Action::Restore),
                    Some(_) => { return Err(import_error(&feat, "Action Required")); },
                    None => { return Err(import_error(&feat, "Action Required")); }
                }
            },
            None => { return Err(import_error(&feat, "Action Required")); },
        }
    }
}

pub fn get_key(feat: &geojson::Feature) -> Result<Option<String>, HecateError> {
    match feat.foreign_members {
        None => Ok(None),
        Some(ref members) => {
            match members.get("key") {
                None => Ok(None),
                Some(key) => {
                    if key.is_null() { return Ok(None); }

                    match key.as_str() {
                        Some(ref key) => Ok(Some(String::from(*key))),
                        None => Err(import_error(&feat, "key must be a string value"))
                    }
                }
            }
        }
    }
}

pub fn action(trans: &postgres::transaction::Transaction, schema_json: &Option<serde_json::value::Value>, feat: &geojson::Feature, delta: &Option<i64>) -> Result<Response, HecateError> {
    let action = get_action(&feat)?;

    let mut scope = valico::json_schema::Scope::new();
    let schema = match schema_json {
        &Some(ref schema) => {
            match scope.compile_and_return(schema.clone(), false) {
                Ok(schema) => Some(schema),
                Err(_) => { return Err(HecateError::new(400, String::from("Schema Error"), None)); }
            }
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

pub fn create(trans: &postgres::transaction::Transaction, schema: &Option<valico::json_schema::schema::ScopedSchema>, feat: &geojson::Feature, delta: &Option<i64>) -> Result<Response, HecateError> {
    if get_version(&feat).is_ok() {
        return Err(import_error(&feat, "Cannot have Version"));
    }

    let props = match feat.properties {
        None => { return Err(import_error(&feat, "Properties Required")); },
        Some(ref props) => props
    };

    let valid = match schema {
        &Some(ref schema) => {
            schema.validate(&json!(props)).is_valid()
        },
        &None => true
    };

    if !valid { return Err(import_error(&feat, "Failed to Match Schema")) };

    let geom_str = get_geom_str(&feat)?;

    let props_str = match serde_json::to_string(&props) {
        Ok(props) => props,
        Err(_) => { return Err(import_error(&feat, "Failed to stringify properties")) }
    };

    let key = get_key(&feat)?;

    let id: Option<i64> = match get_id(&feat) {
        Err(_) => None,
        Ok(id) => Some(id)
    };

    if is_force(&feat)? == true {
        match trans.query("
            INSERT INTO geo (version, geom, props, deltas, key)
                VALUES (
                    1,
                    ST_SetSRID(ST_GeomFromGeoJSON($1), 4326),
                    $2::TEXT::JSON,
                    array[COALESCE($3, currval('deltas_id_seq')::BIGINT)],
                    $4
                )
                ON CONFLICT (key) DO UPDATE
                    SET
                        version = geo.version + 1,
                        geom = ST_SetSRID(ST_GeomFromGeoJSON($1), 4326),
                        props = $2::TEXT::JSON,
                        deltas = array_append(geo.deltas, COALESCE($3, currval('deltas_id_seq')::BIGINT))
                RETURNING id;
        ", &[&geom_str, &props_str, &delta, &key]) {
            Ok(res) => Ok(Response {
                old: id,
                new: Some(res.get(0).get(0)),
                version: Some(1)
            }),
            Err(err) => {
                match err.as_db() {
                    Some(e) => {
                        Err(import_error(&feat, e.message.as_str()))
                    },
                    _ => Err(import_error(&feat, "Generic Error"))
                }
            }
        }
    } else {
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
                old: id,
                new: Some(res.get(0).get(0)),
                version: Some(1)
            }),
            Err(err) => {
                match err.as_db() {
                    Some(e) => {
                        if e.message == "duplicate key value violates unique constraint \"geo_key_key\"" {
                            Err(import_error(&feat, "Duplicate Key Value"))
                        } else {
                            Err(import_error(&feat, e.message.as_str()))
                        }
                    },
                    _ => Err(import_error(&feat, "Generic Error"))
                }
            }
        }
    }
}

pub fn modify(trans: &postgres::transaction::Transaction, schema: &Option<valico::json_schema::schema::ScopedSchema>, feat: &geojson::Feature, delta: &Option<i64>) -> Result<Response, HecateError> {
    let props = match feat.properties {
        None => { return Err(import_error(&feat, "Properties Required")); },
        Some(ref props) => props
    };

    let valid = match schema {
        &Some(ref schema) => {
            schema.validate(&json!(props)).is_valid()
        },
        &None => true
    };

    if !valid { return Err(import_error(&feat, "Failed to Match Schema")) };

    let id = get_id(&feat)?;
    let version = get_version(&feat)?;
    let key = get_key(&feat)?;

    let geom_str = get_geom_str(&feat)?;

    let props_str = match serde_json::to_string(&props) {
        Ok(props) => props,
        Err(_) => { return Err(import_error(&feat, "Failed to stringify properties")) }
    };

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
                        Err(import_error(&feat, "Modify Version Mismatch"))
                    } else if e.message == "duplicate key value violates unique constraint \"geo_key_key\"" {
                        Err(import_error(&feat, "Duplicate Key Value"))
                    } else {
                        Err(import_error(&feat, e.message.as_str()))
                    }
                },
                _ => Err(import_error(&feat, "Generic Error"))
            }
        }
    }
}

pub fn delete(trans: &postgres::transaction::Transaction, feat: &geojson::Feature) -> Result<Response, HecateError> {
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
                        Err(import_error(&feat, "Delete Version Mismatch"))
                    } else {
                        Err(import_error(&feat, e.message.as_str()))
                    }
                },
                _ => Err(import_error(&feat, "Generic Error"))
            }
        }
    }
}

pub fn query_by_key(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, key: &String) -> Result<geojson::Feature, HecateError> {
    match conn.query("
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
            WHERE key = $1
        ) f;
    ", &[&key]) {
        Ok(res) => {
            if res.len() != 1 { return Err(HecateError::new(404, String::from("Feature not found"), None)); }

            let feat: postgres::rows::Row = res.get(0);
            let feat: String = feat.get(0);
            let feat: geojson::Feature = match feat.parse() {
                Ok(feat) => match feat {
                    geojson::GeoJson::Feature(feat) => feat,
                    _ => { return Err(HecateError::new(400, String::from("Invalid Feature"), None)); }
                },
                Err(_) => { return Err(HecateError::new(400, String::from("Invalid Feature"), None)); }
            };

            Ok(feat)
        },
        Err(_) => Err(HecateError::new(400, String::from("Invalid feature"), None))
    }
}

pub fn query_by_point(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, point: &String) -> Result<geojson::Feature, HecateError> {
    let lnglat = point.split(":").collect::<Vec<&str>>();

    if lnglat.len() != 2 {
        return Err(HecateError::new(400, String::from("Point must be Lng,Lat"), None));
    }

    let lng: f32 = match lnglat[0].parse() {
        Ok(lng) => lng,
        _ => { return Err(HecateError::new(400, String::from("Longitude coordinate must be numeric"), None)); }
    };
    let lat: f32 = match lnglat[1].parse() {
        Ok(lat) => lat,
        _ => { return Err(HecateError::new(400, String::from("Latitude coordinate must be numeric"), None)); }
    };

    if lng < -180.0 || lng > 180.0 {
        return Err(HecateError::new(400, String::from("Longitude exceeds bounds"), None));
    } else if lat < -90.0 || lat > 90.0 {
        return Err(HecateError::new(400, String::from("Latitude exceeds bounds"), None));
    }

    match conn.query("
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
                ST_Intersects(geom, ST_SetSRID(ST_MakePoint($1, $2), 4326)
        ) f
        LIMIT 1
    ", &[&lng, &lat]) {
        Ok(res) => {
            if res.len() != 1 { return Err(HecateError::new(404, String::from("Feature not found"), None)); }

            let feat: postgres::rows::Row = res.get(0);
            let feat: String = feat.get(0);
            let feat: geojson::Feature = match feat.parse() {
                Ok(feat) => match feat {
                    geojson::GeoJson::Feature(feat) => feat,
                    _ => { return Err(HecateError::new(400, String::from("Invalid Feature"), None)); }
                },
                Err(_) => { return Err(HecateError::new(400, String::from("Invalid Feature"), None)); }
            };

            Ok(feat)
        },
        Err(_) => Err(HecateError::new(400, String::from("Invalid feature"), None))
    }
}

pub fn get(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, id: &i64) -> Result<geojson::Feature, HecateError> {
    match conn.query("
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
    ", &[&id]) {
        Ok(res) => {
            if res.len() != 1 { return Err(HecateError::new(404, String::from("Not Found"), None)); }

            let feat: postgres::rows::Row = res.get(0);
            let feat: String = feat.get(0);
            let feat: geojson::Feature = match feat.parse() {
                Ok(feat) => match feat {
                    geojson::GeoJson::Feature(feat) => feat,
                    _ => { return Err(HecateError::new(400, String::from("Invalid Feature"), None)); }
                },
                Err(_) => { return Err(HecateError::new(400, String::from("Invalid Feature"), None)); }
            };

            Ok(feat)
        },
        Err(_) => Err(HecateError::new(400, String::from("Invalid Feature"), None))
    }
}

pub fn restore(trans: &postgres::transaction::Transaction, schema: &Option<valico::json_schema::schema::ScopedSchema>, feat: &geojson::Feature, delta: &Option<i64>) -> Result<Response, HecateError> {
    let props = match feat.properties {
        None => { return Err(import_error(&feat, "Properties Required")); },
        Some(ref props) => props
    };

    let valid = match schema {
        &Some(ref schema) => {
            schema.validate(&json!(props)).is_valid()
        },
        &None => true
    };

    if !valid { return Err(import_error(&feat, "Failed to Match Schema")) };

    let id = get_id(&feat)?;
    let version = get_version(&feat)?;
    let key = get_key(&feat)?;

    let geom_str = get_geom_str(&feat)?;

    let props_str = match serde_json::to_string(&props) {
        Ok(props) => props,
        Err(_) => { return Err(import_error(&feat, "Failed to stringify properties")) }
    };

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
                return Err(import_error(&feat, "Feature Not Found"));
            }

            //Version will be None if the feature was created but has never been modified since the
            //original create does not need a version
            let prev_version: Option<i64> = history.get(0).get(1);
            match prev_version {
                None => {
                    return Err(import_error(&feat, "Feature Not In Deleted State"));
                },
                Some(prev_version) => {
                    if prev_version != version {
                        return Err(import_error(&feat, "Restore Version Mismatch"));
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
                            if e.message == "duplicate key value violates unique constraint \"geo_id_key\"" {
                                Err(import_error(&feat, "Feature Not In Deleted State"))
                            } else if e.message == "duplicate key value violates unique constraint \"geo_key_key\"" {
                                Err(import_error(&feat, "Duplicate Key Value"))
                            } else {
                                Err(import_error(&feat, "Generic Error"))
                            }
                        }
                        _ => Err(import_error(&feat, "Generic Error"))
                    }
                }
            }
        },
        Err(_) => {
            Err(import_error(&feat, "Error Fetching History"))
        }
    }
}

pub fn get_bbox_stream(conn: r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, bbox: Vec<f64>) -> Result<PGStream, HecateError> {
    if bbox.len() != 4 {
        return Err(HecateError::new(400, String::from("Invalid BBOX"), None));
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
        Err(_) => Err(HecateError::new(400, String::from("Not Found"), None))
    }
}

pub fn get_bbox(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, bbox: Vec<f64>) -> Result<geojson::FeatureCollection, HecateError> {
    if bbox.len() != 4 {
        return Err(HecateError::new(400, String::from("Invalid BBOX"), None));
    }

    match conn.query("
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
    ", &[&bbox[0], &bbox[1], &bbox[2], &bbox[3]]) {
        Ok(res) => {
            let mut fc = geojson::FeatureCollection {
                bbox: None,
                features: vec![],
                foreign_members: None
            };

            for row in res.iter() {
                let feat: String = row.get(0);
                let feat: geojson::Feature = match feat.parse() {
                    Ok(geojson::GeoJson::Feature(feat)) => feat,
                    _ => { return Err(HecateError::new(400, String::from("Invalid Feature"), None)); }
                };

                fc.features.push(feat);
            }

            Ok(fc)
        },
        Err(_) => Err(HecateError::new(400, String::from("Invalid Feature"), None))
    }
}
