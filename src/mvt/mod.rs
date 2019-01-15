mod builder;
mod encoder;
pub mod geom_encoder;
pub mod geom;
pub mod grid;
pub mod screen;

#[cfg_attr(rustfmt, rustfmt_skip)]
pub mod proto; // protoc --rust_out . proto.proto

#[cfg(test)]
mod builder_test;

#[cfg(test)]
mod geom_encoder_test;

use err::HecateError;
pub use self::builder::{Tile, Layer, Feature, Value};
pub use self::encoder::{Decode, Encode};
pub use self::grid::{Grid};

pub fn db_get(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, coord: String) -> Result<Option<proto::Tile>, HecateError> {
    match conn.query("
        SELECT tile
        FROM tiles
        WHERE
            ref = $1
            AND NOW() > created + INTERVAL '4 hours'
    ", &[&coord]) {
        Ok(rows) => {
            if rows.len() == 0 { return Ok(None); }

            let bytes: Vec<u8> = rows.get(0).get(0);
            let tile = proto::Tile::from_bytes(&bytes).unwrap();

            Ok(Some(tile))
        },
        Err(err) => Err(HecateError::from_db(err))
    }
}

pub fn db_create(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, z: &u8, x: &u32, y: &u32) -> Result<proto::Tile, HecateError> {
    let grid = Grid::web_mercator();
    let bbox = grid.tile_extent(*z, *x, *y);
    let mut tile = Tile::new(&bbox);

    let mut layer = Layer::new("data");

    let mut limit: Option<i64> = None;
    if *z < 10 { limit = Some(10) }
    else if *z < 14 { limit = Some(100) }

    let rows = conn.query("
        SELECT
            id,
            props,
            ST_Transform(geom::geometry,3857)
        FROM geo
        WHERE
            geom && ST_Transform(ST_MakeEnvelope($1, $2, $3, $4, $5), 4326)
        ORDER BY ST_Area(geom)
        LIMIT $6
    ", &[&bbox.minx, &bbox.miny, &bbox.maxx, &bbox.maxy, &grid.srid, &limit]).unwrap();

    for row in rows.iter() {
        let id: i64 = row.get(0);
        let mut feature = Feature::new(row.get(2));
        feature.set_id(id as u64);

        feature.add_property("hecate:id", Value::String(id.to_string()));

        let props: serde_json::Value = row.get(1);

        let props = props.as_object().unwrap();

        for (k, v) in props.iter() {
            feature.add_property(k.to_string(), Value::String(v.to_string()));
        }

        layer.add_feature(feature);
    }

    tile.add_layer(layer);

    Ok(tile.encode(&grid))
}


pub fn db_cache(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, coord: String, tile: &proto::Tile) -> Result<(), HecateError> {
    match conn.query("
        INSERT INTO tiles (ref, tile, created)
            VALUES ($1, $2, NOW())
                ON CONFLICT (ref) DO UPDATE SET tile = $2;
    ", &[&coord, &tile.to_bytes().unwrap()]) {
        Err(err) => Err(HecateError::from_db(err)),
        _ => Ok(())
    }
}

pub fn wipe(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>) -> Result<serde_json::Value, HecateError> {
    match conn.execute("
        DELETE FROM tiles;
    ", &[]) {
        Ok(_) => Ok(json!(true)),
        Err(err) => Err(HecateError::from_db(err))
    }
}

pub fn meta(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, z: u8, x: u32, y: u32) -> Result<serde_json::Value, HecateError> {
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

pub fn get(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, z: u8, x: u32, y: u32, regen: bool) -> Result<proto::Tile, HecateError> {
    if regen == false {
        match db_get(&conn, format!("{}/{}/{}", &z, &x, &y))? {
            Some(tile) => { return Ok(tile); }
            _ => ()
        };
    }

    let tile = db_create(&conn, &z, &x, &y)?;

    db_cache(&conn, format!("{}/{}/{}", &z, &x, &y), &tile)?;

    Ok(tile)
}
