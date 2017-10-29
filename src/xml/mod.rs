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

pub struct OSMTypes {
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
            Some(ref geom) => {
                match geom.value {
                    geojson::Value::Point(ref coords) => point(&feat, &coords, &mut osm),
                    geojson::Value::MultiPoint(ref coords) => multipoint(&feat, &coords, &osm),
                    geojson::Value::LineString(ref coords) => linestring(&feat, &coords, &osm),
                    geojson::Value::MultiLineString(ref coords) => multilinestring(&feat, &coords, &osm),
                    geojson::Value::Polygon(ref coords) => polygon(&feat, &coords, &osm),
                    geojson::Value::MultiPolygon(ref coords) => multipolygon(&feat, &coords, &osm),
                    _ => { return Err(XMLError::GCNotSupported); }
				}
            },
            None => { return Err(XMLError::Unknown); }
        }
    }

    xml.push_str(&*osm.nodes);
    xml.push_str(&*osm.ways);
    xml.push_str(&*osm.rels);
    xml.push_str("</osm>");

    Ok(xml)
}

pub fn point(feat: &geojson::Feature, coords: &geojson::PointType, osm: &mut OSMTypes) -> Result<bool, XMLError> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    let mut xml_node = XMLEvents::BytesStart::owned(b"node".to_vec(), 4);
    xml_node.push_attribute(("id", "1"));
    xml_node.push_attribute(("version", "1"));
    xml_node.push_attribute(("lat", &*coords[0].to_string()));
    xml_node.push_attribute(("lon", &*coords[1].to_string()));

    writer.write_event(XMLEvents::Event::Start(xml_node)).unwrap();

	match feat.properties {
		Some(props) => {
			for (k, v) in props.iter() {
				//let mut xml_tag = XMLEvents::BytesStart::owned(b"tag".to_vec(), 3);
				//xml_tag.push_attribute(("k", &*k.to_string()));
				//xml_tag.push_attribute(("v", &*v.to_string()));

				//writer.write_event(XMLEvents::Event::Empty(xml_tag)).unwrap();
			}
		},
		None => ()
	};

    writer.write_event(XMLEvents::Event::End(XMLEvents::BytesEnd::borrowed(b"node"))).unwrap();

    osm.nodes.push_str(&*String::from_utf8(writer.into_inner().into_inner()).unwrap());

	Ok(true)
}
pub fn multipoint(feat: &geojson::Feature, coords: &Vec<geojson::PointType>, osm: &OSMTypes) {
}
pub fn linestring(feat: &geojson::Feature, coords: &geojson::LineStringType, osm: &OSMTypes) {
}
pub fn multilinestring(feat: &geojson::Feature, coords: &Vec<geojson::LineStringType>, osm: &OSMTypes) {
}
pub fn polygon(feat: &geojson::Feature, coords: &geojson::PolygonType, osm: &OSMTypes) {
}
pub fn multipolygon(feat: &geojson::Feature, coords: &Vec<geojson::PolygonType>, osm: &OSMTypes) {
}
