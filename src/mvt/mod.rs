extern crate postgis;
extern crate protobuf;

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
    NotFound
}

impl MVTError {
    pub fn to_string(&self) -> String {
        match *self {
            MVTError::NotFound => { String::from("Tile not found") }

        }
    }
}

pub fn get(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, z: u8, x: u32, y: u32) -> Result<proto::Tile, MVTError> {
    let grid = Grid::web_mercator();
    let bbox = grid.tile_extent(z, x, y);
    let mut tile = Tile::new(&bbox);

    let mut layer = Layer::new("data");

    let mut limit: Option<i64> = None;
    if z < 10 { limit = Some(10) }
    else if z < 14 { limit = Some(100) }

    let rows = conn.query("
        SELECT
            id,
            ST_Transform(geom::geometry,3857),
            GeometryType(geom)
        FROM geo
        WHERE
            geom && ST_Transform(ST_MakeEnvelope($1, $2, $3, $4, $5), 4326)
        ORDER BY ST_Area(geom)
        LIMIT $6
    ", &[&bbox.minx, &bbox.miny, &bbox.maxx, &bbox.maxy, &grid.srid, &limit]).unwrap();

    for row in rows.iter() {
        let id: i64 = row.get(0);
        let mut feature = Feature::new(row.get(1));
        feature.set_id(id as u64);
        feature.add_property("id", Value::String(id.to_string()));
        layer.add_feature(feature);
    }

    tile.add_layer(layer);

    let encoded = tile.encode(&grid);

    Ok(encoded)
}
