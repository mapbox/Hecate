extern crate postgis;
extern crate protobuf;
extern crate serde_json;

use r2d2; 
use r2d2_postgres;

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

pub use self::builder::{Tile, Layer, Feature, Value};
pub use self::encoder::{Decode, Encode};
pub use self::grid::{Grid};

#[derive(Debug, PartialEq)]
pub enum MVTError {
    NotFound,
    DB,
}

impl MVTError {
    pub fn to_string(&self) -> String {
        match *self {
            MVTError::NotFound => { String::from("Tile not found") },
            MVTError::DB => { String::from("Tile Database Error") }

        }
    }
}

pub fn db_get(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, coord: String) -> Result<Option<proto::Tile>, MVTError> {
    let rows = match conn.query("
        SELECT tile
        FROM tiles
        WHERE
            ref = $1
            AND NOW() > created + INTERVAL '4 hours'
    ", &[&coord]) {
        Ok(rows) => rows,
        Err(err) => match err.as_db() {
            Some(_e) => { return Err(MVTError::DB); },
            _ => { return Err(MVTError::DB); }
        }
    };

    if rows.len() == 0 { return Ok(None); }

    let bytes: Vec<u8> = rows.get(0).get(0);
    let tile = proto::Tile::from_bytes(&bytes).unwrap();

    Ok(Some(tile))
}

pub fn db_create(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, z: &u8, x: &u32, y: &u32) -> Result<proto::Tile, MVTError> {
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
            let v = serde_json::to_string(v).unwrap();
            feature.add_property(k.clone(), Value::String(v));
        }

        layer.add_feature(feature);
    }

    tile.add_layer(layer);

    Ok(tile.encode(&grid))
}


pub fn db_cache(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, coord: String, tile: &proto::Tile) -> Result<(), MVTError> {
    match conn.query("
        INSERT INTO tiles (ref, tile, created)
            VALUES ($1, $2, NOW())
                ON CONFLICT (ref) DO UPDATE SET tile = $2;
    ", &[&coord, &tile.to_bytes().unwrap()]) {
        Err(err) => match err.as_db() {
            Some(_e) => Err(MVTError::DB),
            _ => Err(MVTError::DB),
        }
        _ => Ok(())
    }
}

pub fn get(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, z: u8, x: u32, y: u32, regen: bool) -> Result<proto::Tile, MVTError> {
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
