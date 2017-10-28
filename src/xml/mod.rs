extern crate geojson;
extern crate quick_xml;

use std::io::Cursor;
use self::quick_xml::writer::Writer;
use self::quick_xml::events as XMLEvents;

pub enum XMLError {
    Unknown,
    GCNotSupported
}

impl XMLError {
    pub fn to_string(&self) -> &str {
        match &self {
            Unknown => {
                "Unknown Error"
            },
            GCNotSupported => {
                "GeometryCollection are not currently supported"
            }
        }
    }
}

struct OSMTypes {
    nodes: String,
    ways: String,
    rels: String
}

impl OSMTypes {
    pub fn new() -> OSMTypes {
        OSMTypes {
            nodes: String::from(""),
            ways: String::from(""),
            rels: String::from("")
        }
    }
}

pub fn from(fc: &geojson::FeatureCollection) -> Result<String, XMLError> {
    let mut xml: String = String::from(r#"<?xml version='1.0' encoding='UTF-8'?><osm version="0.6" generator="ROSM">"#);
    let mut osm = OSMTypes::new();

    for feat in &fc.features {
        match feat.geometry {
            Some(geom) => {
                match geom {
                    geojson::Value::Point => point(&feat, &osm),
                    //MultiPoint(mpt) => multipoint(&feat, &mpt, &osm),
                    //LineString(ln) => linestring(&feat, &ln, &osm),
                    //MultiLineString(mln) => multilinestring(&feat, &mln, &osm),
                    //Polygon(py) => polygon(&feat, &py, &osm),
                    //MultiPolygon(mpy) => multipolygon(&feat, &mpy, &osm),
                    _ => { return Err(XMLError::GCNotSupported) },
                }
            }
            None => ()
        }
    }

    xml.push_str("</osm>");

    Ok(xml)
}

pub fn point(feat: &geojson::Feature, osm: &OSMTypes) {
}

pub fn multipoint(feat: &geojson::Feature, osm: &OSMTypes) {
}
pub fn linestring(feat: &geojson::Feature, osm: &OSMTypes) {
}
pub fn multilinestring(feat: &geojson::Feature, osm: &OSMTypes) {
}
pub fn polygon(feat: &geojson::Feature, osm: &OSMTypes) {
}
pub fn multipolygon(feat: &geojson::Feature, osm: &OSMTypes) {
}
