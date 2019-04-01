use postgres;
use geo::prelude::*;
use std::collections::HashMap;
use err::HecateError;

///Get the history of a particular feature
pub fn history(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, feat_id: &i64) -> Result<serde_json::Value, HecateError> {
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
            if res.len() == 0 {
                return Err(HecateError::new(400, String::from("Could not find history for given id"), None))
            }

            let history: serde_json::Value = res.get(0).get(0);
            Ok(history)
        },
        Err(err) => Err(HecateError::from_db(err))
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
        Err(err) => Err(HecateError::from_db(err)),
        Ok(res) => { Ok(res.get(0).get(0)) }
    }
}

pub fn list_by_date(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, start: Option<chrono::NaiveDateTime>, end: Option<chrono::NaiveDateTime>, limit: Option<i64>) -> Result<serde_json::Value, HecateError> {
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

pub fn list_by_offset(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, offset: Option<i64>, limit: Option<i64>) -> Result<serde_json::Value, HecateError> {
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

pub fn tiles(conn: &impl postgres::GenericConnection, id: &i64) -> Result<Vec<(i32, i32, u8)>, HecateError> {
    match conn.query("
        SELECT
            ST_FlipCoordinates(ST_GeomFromGeoJSON(json_array_elements((features->>'features')::JSON)->>'geometry'))
        FROM
            deltas
        WHERE
            id = $1
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
                        let geomtiles = match tilecover::tiles(&geom, 14) {
                            Ok(geomtiles) => geomtiles,
                            Err(err) => {
                                return Err(HecateError::new(500, String::from("Could not generate tilecover"), None));
                            }
                        };

                        for geomtile in geomtiles {
                            tiles.insert(geomtile, true);
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

pub fn get_json(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, id: &i64) -> Result<serde_json::Value, HecateError> {
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
