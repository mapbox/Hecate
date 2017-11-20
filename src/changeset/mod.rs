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

pub fn create(trans: &postgres::transaction::Transaction, fc: &geojson::FeatureCollection, props: &HashMap<String, Option<String>>, uid: &i64) -> Result<bool, ChangesetError> {
    let fc_str = serde_json::to_string(&fc).unwrap();

    match trans.execute("
        INSERT INTO deltas (id, created, features, uid, props) VALUES (
            nextval('deltas_id_seq'),
            current_timestamp,
            $1::TEXT::JSON,
            $2,
            to_json($3::HSTORE)
        );
    ", &[&fc_str, &uid, &props]) {
        Err(err) => {
            match err.as_db() {
                Some(_e) => { Err(ChangesetError::CreationFail) },
                _ => Err(ChangesetError::CreationFail)
            }
        },
        Ok(_) => Ok(true)
    }
}
