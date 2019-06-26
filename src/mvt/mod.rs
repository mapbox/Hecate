#[cfg_attr(rustfmt, rustfmt_skip)]
pub mod grid;

use crate::err::HecateError;
pub use self::grid::{Grid};

pub fn db_get(conn: &impl postgres::GenericConnection, coord: String) -> Result<Option<Vec<u8>>, HecateError> {
    match conn.query("
        SELECT tile
        FROM tiles
        WHERE
            ref = $1
            AND NOW() > created + INTERVAL '4 hours'
    ", &[&coord]) {
        Ok(rows) => {
            if rows.len() == 0 { return Ok(None); }

            let tile: Vec<u8> = rows.get(0).get(0);

            Ok(Some(tile))
        },
        Err(err) => Err(HecateError::from_db(err))
    }
}

pub fn db_create(conn: &impl postgres::GenericConnection, z: &u8, x: &u32, y: &u32) -> Result<Vec<u8>, HecateError> {
    let grid = Grid::web_mercator();
    let bbox = grid.tile_extent(*z, *x, *y);

    println!("
        SELECT
            ST_AsMVT(q, 'data', 4096, 'geom')
        FROM (
            SELECT
                id,
                ST_AsMVTGeom(geom, ST_MakeEnvelope({}, {}, {}, {}, 4326), 4096, 256, false) AS geom
            FROM
                geo
        ) q
    ")


    match conn.query("
        SELECT
            ST_AsMVT(q, 'data', 4096, 'geom')
        FROM (
            SELECT
                id,
                ST_AsMVTGeom(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326), 4096, 256, false) AS geom
            FROM
                geo
        ) q
    ", &[&bbox.minx, &bbox.miny, &bbox.maxx, &bbox.maxy]) {
        Ok(res) => {
            let tile: Vec<u8> = res.get(0).get(0);
            Ok(tile)
        },
        Err(err) => Err(HecateError::from_db(err))
    }
}


pub fn db_cache(conn: &impl postgres::GenericConnection, coord: String, tile: &Vec<u8>) -> Result<(), HecateError> {
    match conn.query("
        INSERT INTO tiles (ref, tile, created)
            VALUES ($1, $2, NOW())
                ON CONFLICT (ref) DO UPDATE SET tile = $2;
    ", &[&coord, &tile]) {
        Err(err) => Err(HecateError::from_db(err)),
        _ => Ok(())
    }
}

pub fn wipe(conn: &impl postgres::GenericConnection) -> Result<serde_json::Value, HecateError> {
    match conn.execute("
        DELETE FROM tiles;
    ", &[]) {
        Ok(_) => Ok(json!(true)),
        Err(err) => Err(HecateError::from_db(err))
    }
}

pub fn meta(conn: &impl postgres::GenericConnection, z: u8, x: u32, y: u32) -> Result<serde_json::Value, HecateError> {
    match conn.query("
        SELECT
            COALESCE(row_to_json(t), '{}'::JSON)
        FROM (
            SELECT
                created AS created
            FROM
                tiles
            WHERE
                ref = $1
        ) t;
    ", &[&format!("{}/{}/{}", &z, &x, &y)]) {
        Ok(rows) => {
            if rows.len() != 1 {
                Err(HecateError::new(404, String::from("Metadata Not Found"), None))
            } else {
                let meta: serde_json::Value = rows.get(0).get(0);
                Ok(meta)
            }
        },
        Err(err) => Err(HecateError::from_db(err))
    }
}

pub fn get(conn: &impl postgres::GenericConnection, z: u8, x: u32, y: u32, regen: bool) -> Result<Vec<u8>, HecateError> {
    if regen == false {
        match db_get(conn, format!("{}/{}/{}", &z, &x, &y))? {
            Some(tile) => { return Ok(tile); }
            _ => ()
        };
    }

    let tile = db_create(conn, &z, &x, &y)?;

    db_cache(conn, format!("{}/{}/{}", &z, &x, &y), &tile)?;

    Ok(tile)
}
