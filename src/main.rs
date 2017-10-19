#[macro_use] extern crate clap;
extern crate iron;
extern crate router;
extern crate geojson;
extern crate hecate;
extern crate postgres;
extern crate mount;
extern crate persistent;
extern crate r2d2;
extern crate r2d2_postgres;

use iron::typemap::Key;

//Postgres Connection Pooling
use r2d2::{Pool, PooledConnection};
use r2d2_postgres::{PostgresConnectionManager, TlsMode};

pub struct DB;
impl Key for DB { type Value = PostgresPool; }

use clap::App;
use iron::prelude::*;
use iron::status;
use router::Router;
use std::io::Read;
use geojson::GeoJson;
use hecate::feature;
use mount::Mount;

pub type PostgresPool = Pool<PostgresConnectionManager>;
pub type PostgresPooledConnection = PooledConnection<PostgresConnectionManager>;

fn main() {
    let cli_cnf = load_yaml!("cli.yml");
    let matched = App::from_yaml(cli_cnf).get_matches();

    let cn_str = String::from("postgres://postgres@localhost:5432/hecate");
    let manager = ::r2d2_postgres::PostgresConnectionManager::new(cn_str, TlsMode::None).unwrap();
    let config = ::r2d2::Config::builder().pool_size(6).build();
    let pool = ::r2d2::Pool::new(config, manager).unwrap();

    let mut router = Router::new();
    router.get("/", index, "index");

    // Individual Feature Operations in GeoJSON Only
    router.post("/api/data/feature", feature_post, "postFeature");
    router.get("/api/data/feature/:feature", feature_get, "getFeature");
    router.delete("/api/data/delete", feature_del, "Feature");

    let mut mount = Mount::new();
    mount.mount("/", router);

    let mut middleware = Chain::new(mount);
    middleware.link(persistent::Read::<DB>::both(pool));
    Iron::new(middleware).http("localhost:3000").unwrap();
}

fn index(_req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok)))
}

fn feature_post(req: &mut Request) -> IronResult<Response> {
    match req.headers.get::<iron::headers::ContentType>() {
        Some(ref header) => {
            if **header != iron::headers::ContentType::json() {
                return Ok(Response::with((status::UnsupportedMediaType, "ContentType must be application/json")));
            }
        },
        None => {
            return Ok(Response::with((status::UnsupportedMediaType, "ContentType must be application/json")));
        }
    }

    let mut body_str = String::new();
    req.body.read_to_string(&mut body_str).unwrap();

    let geojson = body_str.parse::<GeoJson>().unwrap();

    let geojson = match geojson {
        GeoJson::Feature(feat) => {
            feat
        },
        _ => {
            return Ok(Response::with((status::UnsupportedMediaType, "Body must be valid GeoJSON Feature")));
        }
    };

    let pool = req.get::<persistent::Read<DB>>().unwrap();
    let conn = pool.get().unwrap();

    feature::put(conn, geojson);

    Ok(Response::with((status::Ok)))
}

fn feature_get(_req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok)))
}

fn feature_del(_req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok)))
}
