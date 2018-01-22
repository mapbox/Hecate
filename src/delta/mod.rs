extern crate geojson;
extern crate serde_json;
extern crate r2d2;
extern crate  r2d2_postgres;
extern crate rocket_contrib;

use self::rocket_contrib::Json as Json;
use postgres;

use std::collections::HashMap;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum DeltaError {
    CreationFail,
    ListFail,
    FinalizeFail,
    NotFound
}

impl DeltaError {
    pub fn to_string(&self) -> String {
        match *self {
            DeltaError::CreationFail => { String::from("Delta Creation Failure") },
            DeltaError::ListFail => { String::from("Delta Listing Failed") },
            DeltaError::FinalizeFail => { String::from("Finalization Failure") },
            DeltaError::NotFound => { String::from("Delta not found") }
        }
    }
}

pub fn open(trans: &postgres::transaction::Transaction, props: &HashMap<String, Option<String>>, uid: &i64) -> Result<i64, DeltaError> {
    match trans.query("
        INSERT INTO deltas (id, created, props, uid) VALUES (
            nextval('deltas_id_seq'),
            current_timestamp,
            to_json($1::HSTORE),
            $2
        ) RETURNING id;
    ", &[&props, &uid]) {
        Err(err) => {
            match err.as_db() {
                Some(_e) => { Err(DeltaError::CreationFail) },
                _ => Err(DeltaError::CreationFail)
            }
        },
        Ok(res) => { Ok(res.get(0).get(0)) }
    }

}

pub fn create(trans: &postgres::transaction::Transaction, fc: &geojson::FeatureCollection, props: &HashMap<String, Option<String>>, uid: &i64) -> Result<i64, DeltaError> {
    let fc_str = serde_json::to_string(&fc).unwrap();

    match trans.query("
        INSERT INTO deltas (id, created, features, uid, props, affected) VALUES (
            nextval('deltas_id_seq'),
            current_timestamp,
            $1::TEXT::JSON,
            $2,
            to_json($3::HSTORE),
            $4
        ) RETURNING id;
    ", &[&fc_str, &uid, &props, &affected(&fc)]) {
        Err(err) => {
            match err.as_db() {
                Some(_e) => { Err(DeltaError::CreationFail) },
                _ => Err(DeltaError::CreationFail)
            }
        },
        Ok(res) => { Ok(res.get(0).get(0)) }
    }
}

pub fn list_json(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, offset: Option<i64>) -> Result<Json, DeltaError> {
    let offset = match offset {
        None => 0,
        Some(offset) => offset
    };

    match conn.query("
        SELECT array_to_json(Array_Agg(djson.delta))::JSON
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
                    AND deltas.id > $1
                ORDER BY id DESC
                LIMIT 20
            ) d
        ) djson;
    ", &[&offset]) {
        Err(err) => {
            match err.as_db() {
                Some(_e) => { Err(DeltaError::ListFail) },
                _ => Err(DeltaError::ListFail)
            }
        },
        Ok(res) => {
            let d_json: serde_json::Value = res.get(0).get(0);
            Ok(Json(d_json))
        }
    }
}

pub fn modify_props(id: &i64, trans: &postgres::transaction::Transaction, props: &HashMap<String, Option<String>>, uid: &i64) -> Result<i64, DeltaError> {
    match trans.query("
        UPDATE deltas
            SET
                props = to_json($3::HSTORE)
            WHERE
                id = $1
                AND uid = $2
                AND finalized = false;
    ", &[&id, &uid, &props]) {
        Err(err) => {
            match err.as_db() {
                Some(_e) => { Err(DeltaError::CreationFail) },
                _ => Err(DeltaError::CreationFail)
            }
        },
        _ => { Ok(*id) }
    }
}

pub fn modify(id: &i64, trans: &postgres::transaction::Transaction, fc: &geojson::FeatureCollection, uid: &i64) -> Result<i64, DeltaError> {
    let fc_str = serde_json::to_string(&fc).unwrap();

    match trans.query("
        UPDATE deltas
            SET
                features = $2::TEXT::JSON,
                affected = $4
            WHERE
                id = $1
                AND uid = $3
                AND finalized = false;
    ", &[&id, &fc_str, &uid, &affected(&fc)]) {
        Err(err) => {
            match err.as_db() {
                Some(_e) => { Err(DeltaError::CreationFail) },
                _ => Err(DeltaError::CreationFail)
            }
        },
        _ => { Ok(*id) }
    }
}

pub fn finalize(id: &i64, trans: &postgres::transaction::Transaction) -> Result<i64, DeltaError> {
    match trans.query("
        UPDATE deltas
            SET finalized = true
            WHERE id = $1
    ", &[&id]) {
        Err(err) => {
            match err.as_db() {
                Some(_e) => { Err(DeltaError::FinalizeFail) },
                _ => Err(DeltaError::FinalizeFail)
            }
        },
        _ => Ok(*id)
    }
}

pub fn is_open(id: &i64, trans: &postgres::transaction::Transaction) -> Result<bool, DeltaError> {
    match trans.query("
        SELECT NOT finalized FROM deltas WHERE id = $1
    ", &[&id]) {
        Err(err) => {
            match err.as_db() {
                Some(_e) => { Err(DeltaError::FinalizeFail) },
                _ => Err(DeltaError::FinalizeFail)
            }
        },
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
            Some(ref id) => {
                match id.as_i64() {
                    Some(id) => affected.push(id),
                    None => ()
                }
            },
            None => ()
        }
    }

    return affected;
}

