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

    let mut limit: Option<i64> = None;
    if *z < 10 {
        limit = Some(10)
    } else if *z < 14 {
        limit = Some(100)
    }

    match conn.query("
        SELECT
            ST_AsMVT(q, 'data', 4096, 'geom')
        FROM (
            SELECT
                id,
                ST_AsMVTGeom(geom, ST_Transform(ST_MakeEnvelope($1, $2, $3, $4, $5), 4326), 4096, 256, false) AS geom
            FROM
                geo
            WHERE
                ST_Intersects(geom, ST_Transform(ST_MakeEnvelope($1, $2, $3, $4, $5), 4326))
            LIMIT $6
        ) q
    ", &[&bbox.minx, &bbox.miny, &bbox.maxx, &bbox.maxy, &grid.srid, &limit]) {
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

///
/// If you only have a single database connection and want a tile regen
/// but no tile return, this function can be used to force a regen
///
pub fn regen(conn: &impl postgres::GenericConnection, z: u8, x: u32, y: u32) -> Option<HecateError> {
    let tile = match db_create(conn, &z, &x, &y) {
        Ok(tile) => tile,
        Err(err) => {
            return Some(err);
        }
    };

    match db_cache(conn, format!("{}/{}/{}", &z, &x, &y), &tile) {
        Ok(_) => None,
        Err(err) => Some(err)
    }
}

///
/// Database friendly connection to return a tile if it exists
/// and if not create & cache it
///
/// It uses a readonly connection where possible, only writing
/// to the master as neede to cache a tile
///
pub fn get(
    conn_read: &impl postgres::GenericConnection,
    conn_write: &impl postgres::GenericConnection,
    z: u8, x: u32, y: u32,
    regen: bool
) -> Result<Vec<u8>, HecateError> {
    if regen == false {
        match db_get(conn_read, format!("{}/{}/{}", &z, &x, &y))? {
            Some(tile) => { return Ok(tile); }
            _ => ()
        };
    }

    let tile = db_create(conn_read, &z, &x, &y)?;

    // A failing cache should be logged but not affect the returned response
    // since we have already generated a valid tile
    match db_cache(conn_write, format!("{}/{}/{}", &z, &x, &y), &tile) {
        Ok(_) => Ok(tile),
        Err(err) => {
            println!("{}", err.as_log());
            Ok(tile)
        }
    }
}
