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
use std::collections::HashMap;
use geojson::GeoJson;
use hecate::feature;
use hecate::changeset;
use hecate::xml;
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
    router.get("/api/data/feature/:feature", feature_get, "feature_get");
    router.post("/api/data/feature", feature_post, "feature_post");
    router.patch("/api/data/feature/:feature", feature_patch, "feature_patch");
    router.delete("/api/data/feature/:feature", feature_del, "feature_del");

    // Multiple Feature Operations in GeoJSON Only (BBOX)
    router.get("/api/data/features", features_get, "features_get");
    router.post("/api/data/features", features_post, "features_post");

    router.get("/api/capabilities", xml_capabilities, "xml_capabilities");
    router.get("/api/0.6/capabilities", xml_capabilities, "xml_06capabilities");
    router.get("/api/0.6/map", xml_map, "xml_map");
    router.put("/api/0.6/changeset/create", xml_changeset_create, "xml_createChangeset");
    router.put("/api/0.6/changeset/:id/upload", xml_changeset_upload, "xml_putChangeset");

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
        },
        Err(_) => { return Ok(bbox_error); }
    };

    let conn = req.get::<persistent::Read<DB>>().unwrap().get().unwrap();

    match feature::get_bbox(&conn, query) {
        Ok(features) => Ok(Response::with((status::Ok, geojson::GeoJson::from(features).to_string()))),
        Err(err) => Ok(Response::with((status::ExpectationFailed, err.to_string())))
    }
}

fn features_post(req: &mut Request) -> IronResult<Response> {
    let fc = match get_geojson(req) {
        Ok(GeoJson::FeatureCollection(fc)) => fc,
        Ok(typ) => { return Ok(Response::with((status::UnsupportedMediaType, "Body must be valid GeoJSON FeatureCollection"))); }
        Err(err) => { return Ok(err); }
    };

    let conn = req.get::<persistent::Read<DB>>().unwrap().get().unwrap();
    let trans = conn.transaction().unwrap();

    for feat in fc.features {
        feature::action(&trans, feat, &1);
    }

    trans.commit().unwrap();
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

    let fc = match feature::get_bbox(&conn, query) {
        Ok(features) => features,
        Err(err) => { return Ok(Response::with((status::ExpectationFailed, err.to_string()))) }
    };

    let xml_str = match xml::from(&fc) {
        Ok(xml_str) => xml_str,
        Err(err) => { return Ok(Response::with((status::ExpectationFailed, err.to_string()))) }
    };

    Ok(Response::with((status::Ok, xml_str)))
}

fn  xml_changeset_create(req: &mut Request) -> IronResult<Response> {
    let mut body_str = String::new();
    req.body.read_to_string(&mut body_str).unwrap();

    let map = match xml::to_changeset(&body_str) {
        Ok(map) => map,
        Err(err) => { return Ok(Response::with((status::InternalServerError, err.to_string()))); }
    };

    let conn = req.get::<persistent::Read<DB>>().unwrap().get().unwrap();

    let id = match changeset::create(&conn, &map, &1) {
        Ok(id) => id,
        Err(err) => { println!("{}", err.to_string()); return Ok(Response::with((status::InternalServerError, err.to_string()))); }
    };

    Ok(Response::with((status::Ok, id.to_string())))
}

fn  xml_changeset_upload(req: &mut Request) -> IronResult<Response> {
    let mut body_str = String::new();
    req.body.read_to_string(&mut body_str).unwrap();

    let fc = match xml::to_features(&body_str) {
        Ok(fc) => fc,
        Err(err) => { return Ok(Response::with((status::InternalServerError, err.to_string()))); }
    };

    let conn = req.get::<persistent::Read<DB>>().unwrap().get().unwrap();

    Ok(Response::with((status::Ok)))
}

fn xml_capabilities(_req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "
        <osm version=\"0.6\" generator=\"Hecate Server\">
            <api>
                <version minimum=\"0.6\" maximum=\"0.6\"/>
                <area maximum=\"0.25\"/>
                <waynodes maximum=\"2000\"/>
                <changesets maximum_elements=\"10000\"/>
                <timeout seconds=\"300\"/>
                <status database=\"online\" api=\"online\"/>
            </api>
        </osm>
    ")))
}

fn feature_post(req: &mut Request) -> IronResult<Response> {
    let feat = match get_geojson(req) {
        Ok(GeoJson::Feature(feat)) => feat,
        Ok(_) => { return Ok(Response::with((status::UnsupportedMediaType, "Body must be valid GeoJSON Feature"))); }
        Err(err) => { return Ok(err); }
    };

    let fc = geojson::FeatureCollection {
        bbox: None,
        features: vec![ feat.clone() ],
        foreign_members: None,
    };

    let conn = req.get::<persistent::Read<DB>>().unwrap().get().unwrap();
    let trans = conn.transaction().unwrap();

    let map: HashMap<String, String> = HashMap::new();
    if changeset::create_history(&trans, &fc, &map, &1).is_err() {
        return Ok(Response::with((status::InternalServerError, "Could not create changeset")));
    }

    match feature::put(&trans, &feat, &1) {
        Ok(_) => {
            trans.commit().unwrap();
            Ok(Response::with((status::Ok)))
        },
        Err(err) => Ok(Response::with((status::ExpectationFailed, err.to_string())))
    }
}

fn feature_patch(req: &mut Request) -> IronResult<Response> {
    let feat = match get_geojson(req) {
        Ok(GeoJson::Feature(feat)) => feat,
        Ok(_) => { return Ok(Response::with((status::UnsupportedMediaType, "Body must be valid GeoJSON Feature"))); }
        Err(err) => { return Ok(err); }
    };

    let fc = geojson::FeatureCollection {
        bbox: None,
        features: vec![ feat.clone() ],
        foreign_members: None,
    };

    let conn = req.get::<persistent::Read<DB>>().unwrap().get().unwrap();
    let trans = conn.transaction().unwrap();

    let map: HashMap<String, String> = HashMap::new();
    if changeset::create_history(&trans, &fc, &map, &1).is_err() {
        return Ok(Response::with((status::InternalServerError, "Could not create changeset")));
    }


    match feature::patch(&trans, &feat, &1) {
        Ok(_) => {
            trans.commit().unwrap();
            Ok(Response::with((status::Ok)))
        },
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

    match feature::get(&conn, &feature_id) {
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

    let feat = geojson::Feature {
        id: Some(serde_json::Value::Number(serde_json::Number::from_f64(feature_id as f64).unwrap())),
        bbox: None,
        geometry: None,
        properties: None,
        foreign_members: None
    };

    let fc = geojson::FeatureCollection {
        bbox: None,
        features: vec![ feat ],
        foreign_members: None,
    };

    let conn = req.get::<persistent::Read<DB>>().unwrap().get().unwrap();
    let trans = conn.transaction().unwrap();

    let map: HashMap<String, String> = HashMap::new();
    if changeset::create_history(&trans, &fc, &map, &1).is_err() {
        return Ok(Response::with((status::InternalServerError, "Could not create changeset")));
    }

    match feature::delete(&trans, &feature_id) {
        Ok(_) => {
            trans.commit().unwrap();
            Ok(Response::with((status::Ok, "true")))
        },
        Err(err) => Ok(Response::with((status::ExpectationFailed, err.to_string())))
    }
}

fn get_geojson(req: &mut Request) -> Result<geojson::GeoJson, Response> {
    match req.headers.get::<iron::headers::ContentType>() {
        Some(ref header) => {
            if **header != iron::headers::ContentType::json() {
                return Err(Response::with((status::UnsupportedMediaType, "ContentType must be application/json")));
            }
        },
        None => { return Err(Response::with((status::UnsupportedMediaType, "ContentType must be application/json"))); }
    }

    let mut body_str = String::new();
    req.body.read_to_string(&mut body_str).unwrap();

    let geojson = match body_str.parse::<GeoJson>() {
        Err(_) => { return Err(Response::with((status::UnsupportedMediaType, "Body must be valid GeoJSON Feature"))); },
        Ok(geo) => geo
    };

    Ok(geojson)
}
