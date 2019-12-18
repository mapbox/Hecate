use postgres;
use geo::prelude::*;
use std::collections::HashMap;
use crate::err::HecateError;
use serde_json::Value;

pub struct Delta {
    pub id: Option<i64>,
    pub uid: i64,
    pub props: HashMap<String, Option<String>>,
    pub features: HashMap<i64, Value>
}

impl Delta {
    ///
    /// Create a new Delta object given the user id, properties, & features the delta should modify
    ///
    pub fn new(uid: i64, props: HashMap<String, Option<String>>, features: HashMap<i64, Value>) -> Self {
        Delta {
            id: None,
            uid: uid,
            props: props,
            features: features
        }
    }

    ///
    /// Load and return a delta from the database given a connection and delta id
    ///
    pub fn load(_conn: &impl postgres::GenericConnection, delta_id: i64) -> Self {
        Delta {
            id: Some(delta_id),
            uid: 1,
            props: HashMap::new(),
            features: HashMap::new()
        }
    }
}

pub fn open(trans: &postgres::transaction::Transaction, props: &HashMap<String, Option<String>>, uid: &i64) -> Result<i64, HecateError> {
    match trans.query("
        INSERT INTO deltas (id, created, props, uid) VALUES (
            nextval('deltas_id_seq'),
            current_timestamp,
            to_json($1::HSTORE),
            $2
        ) RETURNING id;
    ", &[&props, &uid]) {
        Err(err) => Err(HecateError::from_db(err)),
        Ok(res) => { Ok(res.get(0).get(0)) }
    }

}

pub fn create(trans: &postgres::transaction::Transaction, fc: &geojson::FeatureCollection, props: &HashMap<String, Option<String>>, uid: &i64) -> Result<i64, HecateError> {
    match trans.query("
        INSERT INTO deltas (id, created, uid, props, affected) VALUES (
            nextval('deltas_id_seq'),
            current_timestamp,
            $1::TEXT::JSON,
            $2,
            to_json($3::HSTORE),
            $4
        ) RETURNING id;
    ", &[&uid, &props, &affected(&fc)]) {
        Err(err) => Err(HecateError::from_db(err)),
        Ok(res) => { Ok(res.get(0).get(0)) }
    }
}

pub fn list_by_date(conn: &impl postgres::GenericConnection, start: Option<chrono::NaiveDateTime>, end: Option<chrono::NaiveDateTime>, limit: Option<i64>) -> Result<serde_json::Value, HecateError> {
    match conn.query("
        SELECT COALESCE(array_to_json(Array_Agg(djson.delta)), '[]')::JSON
        FROM (
            SELECT row_to_json(d) as delta
            FROM (
                SELECT
                    deltas.id,
                    deltas.uid,
                    users.username,
                    deltas.created,
                    deltas.props
                FROM
                    deltas,
                    users
                WHERE
                    deltas.uid = users.id
                    AND ((
                        $1::TIMESTAMP IS NOT NULL
                        AND $2::TIMESTAMP IS NOT NULL
                        AND deltas.created < $1::TIMESTAMP
                        AND deltas.created > $2::TIMESTAMP
                    ) OR (
                        $1::TIMESTAMP IS NOT NULL
                        AND $2::TIMESTAMP IS NULL
                        AND deltas.created < $1::TIMESTAMP
                    ) OR (
                        $1::TIMESTAMP IS NULL
                        AND $2::TIMESTAMP IS NOT NULL
                        AND deltas.created > $2::TIMESTAMP
                    ))
                ORDER BY id DESC
                LIMIT $3
            ) d
        ) djson;
    ", &[&start, &end, &limit]) {
        Err(err) => Err(HecateError::from_db(err)),
        Ok(res) => {
            let d_json: serde_json::Value = res.get(0).get(0);
            Ok(d_json)
        }
    }
}

pub fn list_by_offset(conn: &impl postgres::GenericConnection, offset: Option<i64>, limit: Option<i64>) -> Result<serde_json::Value, HecateError> {
    let offset = match offset {
        None => String::from("Infinity"),
        Some(offset) => offset.to_string()
    };

    let limit = match limit {
        None => Some(20),
        Some(limit) => {
            if limit > 100 {
                Some(100)
            } else {
                Some(limit)
            }
        }
    };

    match conn.query("
        SELECT COALESCE(array_to_json(Array_Agg(djson.delta)), '[]')::JSON
        FROM (
            SELECT row_to_json(d) as delta
            FROM (
                SELECT
                    deltas.id,
                    deltas.uid,
                    users.username,
                    deltas.created,
                    deltas.props
                FROM
                    deltas,
                    users
                WHERE
                    deltas.uid = users.id
                    AND deltas.id < $1::TEXT::FLOAT8
                ORDER BY id DESC
                LIMIT $2
            ) d
        ) djson;
    ", &[&offset, &limit]) {
        Err(err) => Err(HecateError::from_db(err)),
        Ok(res) => {
            let d_json: serde_json::Value = res.get(0).get(0);
            Ok(d_json)
        }
    }
}

pub fn tiles(conn: &impl postgres::GenericConnection, id: &i64, min_zoom: u8, max_zoom: u8) -> Result<Vec<(i32, i32, u8)>, HecateError> {
    match conn.query("
        SELECT
            geom
        FROM
            geo_history
        WHERE
            delta = $1
    ", &[&id]) {
        Err(err) => Err(HecateError::from_db(err)),
        Ok(results) => {
            if results.len() == 0 {
                return Ok(Vec::new());
            }

            let mut tiles: HashMap<(i32, i32, u8), bool> = HashMap::new();

            for res in results.iter() {
                let geom: Option<postgis::ewkb::GeometryT<postgis::ewkb::Point>> = res.get(0);

                let geom: Option<geo::Geometry<f64>> = match geom {
                    Some(geom) => FromPostgis::from_postgis(&geom),
                    None => continue
                };

                match geom {
                    Some(geom) => {
                        for zoom in min_zoom..max_zoom+1 {
                            let geomtiles = match tilecover::tiles(&geom, zoom) {
                                Ok(gt) => gt,
                                Err(_err) => {
                                    return Err(HecateError::new(500, String::from("Could not generate tilecover"), None));
                                }
                            };

                            for geomtile in geomtiles {
                                tiles.insert(geomtile, true);
                            }
                        }
                    },
                    None => ()
                };
            }

            Ok(tiles.keys().map(|key| {
                key.clone()
            }).collect())
        }
    }

}

pub fn get_json(conn: &impl postgres::GenericConnection, id: &i64) -> Result<serde_json::Value, HecateError> {
    match conn.query("
        SELECT COALESCE(row_to_json(t), 'false'::JSON)
        FROM (
            SELECT
                deltas.id,
                deltas.uid,
                users.username,
                (
                    SELECT row_to_json(featCollection)
                    FROM (
                        SELECT
                            'FeatureCollection' as type,
                            (
                                SELECT json_agg(row_to_json(d))
                                FROM (
                                    SELECT
                                        id,
                                        action,
                                        key,
                                        'Feature' as type,
                                        version,
                                        ST_AsGeoJSON(geom)::JSON as geometry,
                                        props as properties
                                    FROM geo_history
                                    WHERE delta = $1
                                    ORDER BY id
                                ) d
                            ) as features
                    ) as featCollection
                ) as features,
                deltas.affected,
                deltas.props,
                deltas.created
            FROM
                deltas,
                users
            WHERE
                deltas.id = $1
                AND deltas.uid = users.id
        ) t
    ", &[&id]) {
        Err(err) => Err(HecateError::from_db(err)),
        Ok(res) => {
            let d_json: serde_json::Value = res.get(0).get(0);
            Ok(d_json)
        }
    }
}

pub fn modify_props(id: &i64, trans: &postgres::transaction::Transaction, props: &HashMap<String, Option<String>>, uid: &i64) -> Result<i64, HecateError> {
    match trans.query("
        UPDATE deltas
            SET
                props = to_json($3::HSTORE)
            WHERE
                id = $1
                AND uid = $2
                AND finalized = false;
    ", &[&id, &uid, &props]) {
        Err(err) => Err(HecateError::from_db(err)),
        _ => { Ok(*id) }
    }
}

pub fn modify(id: &i64, trans: &postgres::transaction::Transaction, fc: &geojson::FeatureCollection, uid: &i64) -> Result<i64, HecateError> {
    match trans.query("
        UPDATE deltas
            SET
                affected = $3
            WHERE
                id = $1
                AND uid = $2
                AND finalized = false;
    ", &[&id, &uid, &affected(&fc)]) {
        Err(err) => Err(HecateError::from_db(err)),
        _ => { Ok(*id) }
    }
}

pub fn finalize(id: &i64, trans: &postgres::transaction::Transaction) -> Result<i64, HecateError> {
    match trans.query("
        UPDATE deltas
            SET finalized = true
            WHERE id = $1
    ", &[&id]) {
        Err(err) => Err(HecateError::from_db(err)),
        _ => Ok(*id)
    }
}

pub fn is_open(id: &i64, trans: &postgres::transaction::Transaction) -> Result<bool, HecateError> {
    match trans.query("
        SELECT NOT finalized FROM deltas WHERE id = $1
    ", &[&id]) {
        Err(err) => Err(HecateError::from_db(err)),
        Ok(row) => {
            if row.is_empty() { return Ok(false); }
            Ok(row.get(0).get(0))
        }
    }
}

pub fn affected(fc: &geojson::FeatureCollection) -> Vec<i64> {
    let mut affected: Vec<i64> = Vec::new();
    for feat in &fc.features {
        match feat.id {
            Some(ref id) => match id {
                geojson::feature::Id::Number(num_id) => affected.push(num_id.as_i64().unwrap()),
                _ => ()
            },
            None => ()
        }
    }

    return affected;
}
