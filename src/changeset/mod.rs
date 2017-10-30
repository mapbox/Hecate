extern crate r2d2;
extern crate r2d2_postgres;
extern crate geojson;
extern crate postgres;
extern crate serde_json;

pub enum ChangesetError {
    NotFound
}

impl ChangesetError {
    pub fn to_string(&self) -> &str {
        match &self {
            NotFound => { "Changeset not found" }
        }
    }
}

pub fn create(trans: &postgres::transaction::Transaction, fc: &geojson::FeatureCollection, uid: &i64) -> Result<bool, ChangesetError> {
    let fc_str = serde_json::to_string(&fc).unwrap();

    trans.execute("
        INSERT INTO deltas (created, features, uid) VALUES (
			current_timestamp,
			$1::TEXT::JSON,
			$2::BIGINT
		)
    ", &[&fc_str, &uid]).unwrap();

    Ok(true)
}
