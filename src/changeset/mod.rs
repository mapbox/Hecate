extern crate r2d2;
extern crate r2d2_postgres;
extern crate geojson;
extern crate postgres;
extern crate serde_json;

use std::collections::HashMap;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum ChangesetError {
    CreationFail,
    NotFound
}

impl ChangesetError {
    pub fn to_string(&self) -> &str {
        match *self {
            ChangesetError::CreationFail => { "Changeset Creation Failure" },
            ChangesetError::NotFound => { "Changeset not found" }
        }
    }
}

pub fn open(trans: &postgres::transaction::Transaction, props: &HashMap<String, Option<String>>, uid: &i64) -> Result<i64, ChangesetError> {
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
                Some(_e) => { Err(ChangesetError::CreationFail) },
                _ => Err(ChangesetError::CreationFail)
            }
        },
        Ok(res) => { Ok(res.get(0).get(0)) }
    }

}

pub fn create(trans: &postgres::transaction::Transaction, fc: &geojson::FeatureCollection, props: &HashMap<String, Option<String>>, uid: &i64) -> Result<i64, ChangesetError> {
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
                Some(_e) => { Err(ChangesetError::CreationFail) },
                _ => Err(ChangesetError::CreationFail)
            }
        },
        Ok(res) => { Ok(res.get(0).get(0)) }
    }
}

pub fn modify(id: &i64, trans: &postgres::transaction::Transaction, fc: &geojson::FeatureCollection, props: &HashMap<String, Option<String>>, uid: &i64) -> Result<i64, ChangesetError> {
    let fc_str = serde_json::to_string(&fc).unwrap();

    match trans.query("
        UPDATE deltas
            SET
                features = $2::TEXT::JSON,
                props = to_json($4::HSTORE),
                affected = $5
            WHERE
                id = $1
                AND uid = $3
    ", &[&id, &fc_str, &uid, &props, &affected(&fc)]) {
        Err(err) => {
            match err.as_db() {
                Some(_e) => { Err(ChangesetError::CreationFail) },
                _ => Err(ChangesetError::CreationFail)
            }
        },
        _ => { Ok(*id) }
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
