#[macro_use] extern crate clap;
extern crate iron;
extern crate router;
extern crate geojson;
extern crate hecate;
extern crate postgres;
extern crate mount;
extern crate persistent;
extern crate r2d2;
extern crate urlencoded;
extern crate r2d2_postgres;
extern crate serde_json;

use iron::typemap::Key;

//Postgres Connection Pooling
use r2d2::{Pool, PooledConnection};
use r2d2_postgres::{PostgresConnectionManager, TlsMode};

pub struct DB;
impl Key for DB { type Value = PostgresPool; }

use urlencoded::UrlEncodedQuery;
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
    let _matched = App::from_yaml(cli_cnf).get_matches();

    let cn_str = String::from("postgres://postgres@localhost:5432/hecate");
    let manager = ::r2d2_postgres::PostgresConnectionManager::new(cn_str, TlsMode::None).unwrap();
    let config = ::r2d2::Config::builder().pool_size(6).build();
    let pool = ::r2d2::Pool::new(config, manager).unwrap();

    let mut router = Router::new();
    router.get("/", index, "index");

    // Individual Feature Operations in GeoJSON Only
    router.get("/api/data/feature/:feature", feature_get, "getFeature");
    router.post("/api/data/feature", feature_post, "postFeature");
    router.patch("/api/data/feature/:feature", feature_patch, "patchFeature");
    router.delete("/api/data/feature/:feature", feature_del, "delFeature");

    // Multiple Feature Operations in GeoJSON Only (BBOX)
    router.get("/api/data/features", features_get, "getFeatures");
    router.post("/api/data/features", features_post, "postFeatures");

    router.get("/api/0.6/map", xml_map, "xml_map");

    let mut mount = Mount::new();
    mount.mount("/", router);

    let mut middleware = Chain::new(mount);
    middleware.link(persistent::Read::<DB>::both(pool));
    Iron::new(middleware).http("localhost:3000").unwrap();
}

fn index(_req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok)))
}

fn features_get(req: &mut Request) -> IronResult<Response> {
    let bbox_error = Response::with((status::BadRequest, "single bbox query param required"));

    let query = match req.get_ref::<UrlEncodedQuery>() {
        Ok(hashmap) => {
            match hashmap.get("bbox") {
                Some(bbox) => {
                    if bbox.len() != 1 { return Ok(bbox_error); }

                    let split: Vec<f64> = bbox[0].split(',').map(|s| s.parse().unwrap()).collect();

                    split
                },
                None => { return Ok(bbox_error); }
            }
        }
        Err(_) => { return Ok(bbox_error); }
    };

    let conn = req.get::<persistent::Read<DB>>().unwrap().get().unwrap();

    match feature::get_bbox(conn, query) {
        Ok(features) => Ok(Response::with((status::Ok, geojson::GeoJson::from(features).to_string()))),
        Err(err) => Ok(Response::with((status::ExpectationFailed, err.to_string())))
    }
}

fn features_post(req: &mut Request) -> IronResult<Response> {
    match req.headers.get::<iron::headers::ContentType>() {
        Some(ref header) => {
            if **header != iron::headers::ContentType::json() {
                return Ok(Response::with((status::UnsupportedMediaType, "ContentType must be application/json")));
            }
        },
        None => { return Ok(Response::with((status::UnsupportedMediaType, "ContentType must be application/json"))); }
    }

    let mut body_str = String::new();
    req.body.read_to_string(&mut body_str).unwrap();

    let geojson = match body_str.parse::<GeoJson>() {
        Err(_) => { return Ok(Response::with((status::UnsupportedMediaType, "Body must be valid GeoJSON Feature"))); },
        Ok(geo) => geo
    };

    let fc = match geojson {
        GeoJson::FeatureCollection(fc) => fc,
        _ => { return Ok(Response::with((status::UnsupportedMediaType, "Body must be valid GeoJSON Feature"))); }
    };

    Ok(Response::with((status::Ok, "true")))
}

fn xml_map(req: &mut Request) -> IronResult<Response> {
    let bbox_error = Response::with((status::BadRequest, "single bbox query param required"));

    let query = match req.get_ref::<UrlEncodedQuery>() {
        Ok(hashmap) => {
            match hashmap.get("bbox") {
                Some(bbox) => {
                    if bbox.len() != 1 { return Ok(bbox_error); }

                    let split: Vec<f64> = bbox[0].split(',').map(|s| s.parse().unwrap()).collect();

                    split
                },
                None => { return Ok(bbox_error); }
            }
        },
        Err(_) => { return Ok(bbox_error); }
    };

    let conn = req.get::<persistent::Read<DB>>().unwrap().get().unwrap();

    match feature::get_bbox(conn, query) {
        Ok(features) => Ok(Response::with((status::Ok, geojson::GeoJson::from(features).to_string()))),
        Err(err) => Ok(Response::with((status::ExpectationFailed, err.to_string())))
    }
}

fn feature_post(req: &mut Request) -> IronResult<Response> {
    match req.headers.get::<iron::headers::ContentType>() {
        Some(ref header) => {
            if **header != iron::headers::ContentType::json() {
                return Ok(Response::with((status::UnsupportedMediaType, "ContentType must be application/json")));
            }
        },
        None => { return Ok(Response::with((status::UnsupportedMediaType, "ContentType must be application/json"))); }
    }

    let mut body_str = String::new();
    req.body.read_to_string(&mut body_str).unwrap();

    let geojson = match body_str.parse::<GeoJson>() {
        Err(_) => { return Ok(Response::with((status::UnsupportedMediaType, "Body must be valid GeoJSON Feature"))); },
        Ok(geo) => geo
    };

    let geojson = match geojson {
        GeoJson::Feature(feat) => feat,
        _ => { return Ok(Response::with((status::UnsupportedMediaType, "Body must be valid GeoJSON Feature"))); }
    };

    let conn = req.get::<persistent::Read<DB>>().unwrap().get().unwrap();

    match feature::put(conn, geojson, &1) {
        Ok(_) => Ok(Response::with((status::Ok))),
        Err(err) => Ok(Response::with((status::ExpectationFailed, err.to_string())))
    }
}

fn feature_patch(req: &mut Request) -> IronResult<Response> {
    match req.headers.get::<iron::headers::ContentType>() {
        Some(ref header) => {
            if **header != iron::headers::ContentType::json() {
                return Ok(Response::with((status::UnsupportedMediaType, "ContentType must be application/json")));
            }
        },
        None => { return Ok(Response::with((status::UnsupportedMediaType, "ContentType must be application/json"))); }
    }

    let mut body_str = String::new();
    req.body.read_to_string(&mut body_str).unwrap();

    let geojson = match body_str.parse::<GeoJson>() {
        Err(_) => { return Ok(Response::with((status::UnsupportedMediaType, "Body must be valid GeoJSON Feature"))); },
        Ok(geo) => geo
    };

    let geojson = match geojson {
        GeoJson::Feature(feat) => feat,
        _ => { return Ok(Response::with((status::UnsupportedMediaType, "Body must be valid GeoJSON Feature"))); }
    };

    let conn = req.get::<persistent::Read<DB>>().unwrap().get().unwrap();

    match feature::patch(conn, geojson, &1) {
        Ok(_) => Ok(Response::with((status::Ok))),
        Err(err) => Ok(Response::with((status::ExpectationFailed, err.to_string())))
    }
}

fn feature_get(req: &mut Request) -> IronResult<Response> {
    let feature_id: i64 = match req.extensions.get::<Router>().unwrap().find("feature") {
        Some(id) => match id.parse() {
            Ok(id) => id,
            Err(_) =>  { return Ok(Response::with((status::ExpectationFailed, "Feature ID Must be numeric"))); }
        },
        None =>  { return Ok(Response::with((status::ExpectationFailed, "Feature ID Must be provided"))); }
    };

    let conn = req.get::<persistent::Read<DB>>().unwrap().get().unwrap();

    match feature::get(conn, &feature_id) {
        Ok(features) => Ok(Response::with((status::Ok, geojson::GeoJson::from(features).to_string()))),
        Err(err) => Ok(Response::with((status::ExpectationFailed, err.to_string())))
    }
}

fn feature_del(req: &mut Request) -> IronResult<Response> {
    let feature_id: i64 = match req.extensions.get::<Router>().unwrap().find("feature") {
        Some(id) => match id.parse() {
            Ok(id) => id,
            Err(_) =>  { return Ok(Response::with((status::ExpectationFailed, "Feature ID Must be numeric"))); }
        },
        None =>  { return Ok(Response::with((status::ExpectationFailed, "Feature ID Must be provided"))); }
    };

    let conn = req.get::<persistent::Read<DB>>().unwrap().get().unwrap();

    match feature::delete(conn, &feature_id) {
        Ok(_) => Ok(Response::with((status::Ok, "true"))),
        Err(err) => Ok(Response::with((status::ExpectationFailed, err.to_string())))
    }
}
