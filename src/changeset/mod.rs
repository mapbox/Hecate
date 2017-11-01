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

pub fn create_history(trans: &postgres::transaction::Transaction, fc: &geojson::FeatureCollection, tags: &HashMap<String, String>, uid: &i64) -> Result<bool, ChangesetError> {
    let fc_str = serde_json::to_string(&fc).unwrap();

    trans.execute("
        INSERT INTO deltas (created, features, uid) VALUES (
			current_timestamp,
			$1::TEXT::JSON,
			$2
		) RETURNING id;
    ", &[&fc_str, &uid]).unwrap();

    Ok(true)
}

pub fn create(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, tags: &HashMap<String, String>, uid: &i64) -> Result<u64, ChangesetError> {
    let fc = geojson::FeatureCollection {
        bbox: None,
        features: vec![ ],
        foreign_members: None,
    };

    let fc_str = serde_json::to_string(&fc).unwrap();

    match conn.execute("
        INSERT INTO deltas (created, features, uid) VALUES (
			current_timestamp,
			$1::TEXT::JSON,
			$2
		) RETURNING id::BIGINT;
    ", &[&fc_str, &uid]) {
        Err(err) => { return Err(ChangesetError::CreationFail); },
        Ok(row) => Ok(row)
    }
}
