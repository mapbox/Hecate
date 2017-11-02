extern crate r2d2;
extern crate r2d2_postgres;
extern crate geojson;
extern crate postgres;
extern crate serde_json;

use std::collections::HashMap;

#[derive(PartialEq)]
pub enum ChangesetError {
    CreationFail,
    NotFound
}

impl ChangesetError {
    pub fn to_string(&self) -> &str {
        match &self {
            CreationFail => { "Changeset Creation Failure" },
            NotFound => { "Changeset not found" }
        }
    }
}

pub fn create_history(trans: &postgres::transaction::Transaction, fc: &geojson::FeatureCollection, props: &HashMap<String, Option<String>>, uid: &i64) -> Result<bool, ChangesetError> {
    let fc_str = serde_json::to_string(&fc).unwrap();

    match trans.execute("
        INSERT INTO deltas (created, features, uid, props) VALUES (
            current_timestamp,
            $1::TEXT::JSON,
            $2,
            to_json($3::hstore)
        );
    ", &[&fc_str, &uid, &props]) {
        Err(err) => { return Err(ChangesetError::CreationFail); },
        Ok(_) => Ok(true)
    }
}

pub fn create(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, props: &HashMap<String, Option<String>>, uid: &i64) -> Result<i64, ChangesetError> {
    let fc = geojson::FeatureCollection {
        bbox: None,
        features: vec![ ],
        foreign_members: None,
    };

    let fc_str = serde_json::to_string(&fc).unwrap();

    match conn.query("
        INSERT INTO deltas (created, features, uid, props) VALUES (
            current_timestamp,
            $1::TEXT::JSON,
            $2,
            to_json($3::HSTORE)
        ) RETURNING id::BIGINT AS id;
    ", &[&fc_str, &uid, &props]) {
        Err(err) => { return Err(ChangesetError::CreationFail); },
        Ok(row) => {
            let id: i64 = row.get(0).get("id");
            Ok(id)
        }
    }
}
