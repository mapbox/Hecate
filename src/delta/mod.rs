extern crate r2d2;
extern crate r2d2_postgres;
extern crate geojson;
extern crate postgres;
extern crate serde_json;

use std::collections::HashMap;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum DeltaError {
    CreationFail,
    FinalizeFail,
    NotFound
}

impl DeltaError {
    pub fn to_string(&self) -> &str {
        match *self {
            DeltaError::CreationFail => { "Delta Creation Failure" },
            DeltaError::FinalizeFail => { "Finalization Failure" },
            DeltaError::NotFound => { "Delta not found" }
        }
    }
}

pub fn create(trans: &postgres::transaction::Transaction, fc: &geojson::FeatureCollection, props: &HashMap<String, Option<String>>, uid: &i64) -> Result<i64, DeltaError> {
    let fc_str = serde_json::to_string(&fc).unwrap();

    match trans.query("
        INSERT INTO deltas (id, created, features, uid, props) VALUES (
            nextval('deltas_id_seq'),
            current_timestamp,
            $1::TEXT::JSON,
            $2,
            to_json($3::HSTORE)
        ) RETURNING id;
    ", &[&fc_str, &uid, &props]) {
        Err(err) => {
            match err.as_db() {
                Some(_e) => { Err(DeltaError::CreationFail) },
                _ => Err(DeltaError::CreationFail)
            }
        },
        Ok(res) => {
            Ok(res.get(0).get(0))
        }
    }
}

pub fn modify(id: &i64, trans: &postgres::transaction::Transaction, fc: &geojson::FeatureCollection, props: &HashMap<String, Option<String>>, uid: &i64) -> Result<i64, DeltaError> {
    let fc_str = serde_json::to_string(&fc).unwrap();

    match trans.query("
        UPDATE deltas
            SET
                features = $2::TEXT::JSON,
                props = to_json($4::HSTORE)
            WHERE
                id = $1
                AND uid = $3
                AND finalized = false;
    ", &[&id, &fc_str, &uid, &props]) {
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
