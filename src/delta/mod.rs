extern crate geojson;
extern crate serde_json;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate chrono;
use postgres;

use std::collections::HashMap;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum DeltaError {
    CreationFail,
    ListFail,
    GetFail,
    FinalizeFail,
    NotFound
}

impl DeltaError {
    pub fn to_string(&self) -> String {
        match *self {
            DeltaError::CreationFail => { String::from("Delta Creation Failure") },
            DeltaError::ListFail => { String::from("Delta Listing Failed") },
            DeltaError::GetFail => { String::from("Delta Get Failed") },
            DeltaError::FinalizeFail => { String::from("Finalization Failure") },
            DeltaError::NotFound => { String::from("Delta not found") }
        }
    }
}

///Get the history of a particular feature
pub fn history(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, feat_id: &i64) -> Result<serde_json::Value, DeltaError> {
    match conn.query("
        SELECT json_agg(row_to_json(t))
        FROM (
            SELECT
                deltas.id,
                deltas.uid,
                JSON_Array_Elements((deltas.features -> 'features')::JSON) AS feat,
                users.username
            FROM
                deltas,
                users
            WHERE
                affected @> ARRAY[$1]::BIGINT[]
                AND users.id = deltas.uid
            ORDER BY id DESC
        ) t
        WHERE
            (feat->>'id')::BIGINT = $1;
    ", &[&feat_id]) {
        Ok(res) => {
            if res.len() == 0 { return Err(DeltaError::GetFail) }

            let history: serde_json::Value = res.get(0).get(0);
            Ok(history)
        },
        Err(err) => {
            match err.as_db() {
                Some(_e) => { Err(DeltaError::GetFail) },
                _ => Err(DeltaError::GetFail)
            }
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

pub fn list_by_date(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, start: Option<chrono::DateTime<chrono::Utc>>, end: Option<chrono::DateTime<chrono::Utc>>, limit: Option<i64>) -> Result<serde_json::Value, DeltaError> {
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
                        $1 IS NOT NULL
                        AND $2 IS NOT NULL
                        AND deltas.created < $1
                        AND deltas.created > $2
                    ) OR (
                        $1 IS NOT NULL
                        AND deltas.created < $1
                    ) OR (
                        $2 IS NOT NULL
                        AND deltas.created > $2
                    ))
                ORDER BY id DESC
                LIMIT $3
            ) d
        ) djson;
    ", &[&start, &end, &limit]) {
        Err(err) => {
            match err.as_db() {
                Some(_e) => { Err(DeltaError::ListFail) },
                _ => Err(DeltaError::ListFail)
            }
        },
        Ok(res) => {
            let d_json: serde_json::Value = res.get(0).get(0);
            Ok(d_json)
        }
    }
}

pub fn list_by_offset(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, offset: Option<i64>, limit: Option<i64>) -> Result<serde_json::Value, DeltaError> {
    let offset = match offset {
        None => String::from("Infinity"),
        Some(offset) => offset.to_string()
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
                LIMIT COALESCE($2, 20)
            ) d
        ) djson;
    ", &[&offset, &limit]) {
        Err(err) => {
            match err.as_db() {
                Some(_e) => { Err(DeltaError::ListFail) },
                _ => Err(DeltaError::ListFail)
            }
        },
        Ok(res) => {
            let d_json: serde_json::Value = res.get(0).get(0);
            Ok(d_json)
        }
    }
}

pub fn get_json(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, id: &i64) -> Result<serde_json::Value, DeltaError> {
    match conn.query("
        SELECT COALESCE(row_to_json(d), 'false'::JSON)
        FROM (
            SELECT
                deltas.id,
                deltas.uid,
                users.username,
                deltas.features,
                deltas.affected,
                deltas.props,
                deltas.created,
                deltas.props
            FROM
                deltas,
                users
            WHERE
                deltas.uid = users.id
                AND deltas.id = $1
        ) d
    ", &[&id]) {
        Err(err) => {
            match err.as_db() {
                Some(_e) => { Err(DeltaError::GetFail) },
                _ => Err(DeltaError::GetFail)
            }
        },
        Ok(res) => {
            let d_json: serde_json::Value = res.get(0).get(0);
            Ok(d_json)
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

