extern crate geojson;
extern crate quick_xml;

use std::io::Cursor;
use self::quick_xml::writer::Writer;
use self::quick_xml::events as XMLEvents;
use std::collections::HashMap;

#[derive(PartialEq)]
pub enum XMLError {
    Unknown,
	Invalid,
    GCNotSupported,
    EncodingFailed
}

impl XMLError {
    pub fn to_string(&self) -> &str {
        match &self {
            Unknown => { "Unknown Error" },
            GCNotSupported => { "GeometryCollection are not currently supported" },
            Invalid => { "Could not parse XML - Invalid" },
            EncodingFailed => { "Encoding Failed" }
        }
    }
}

pub struct OSMTypes {
    node_it: i64,
    nodes: String,
    ways: String,
    rels: String
}

impl OSMTypes {
    pub fn new() -> OSMTypes {
        OSMTypes {
            node_it: 0,
            nodes: String::from(""),
            ways: String::from(""),
            rels: String::from("")
        }
    }
}

pub fn to_changeset_tag(xml_node: &quick_xml::events::BytesStart, map: &mut HashMap<String, Option<String>>) {
	let mut kv: (Option<String>, Option<String>) = (None, None);

    for attr in xml_node.attributes() {
		let attr = attr.unwrap();

        match attr.key {
			b"k"  => kv.0 = Some(String::from_utf8_lossy(attr.value).parse().unwrap()),
            b"v"  => kv.1 = Some(String::from_utf8_lossy(attr.value).parse().unwrap()),
            _ => ()
        }
    }

	map.insert(kv.0.unwrap(), kv.1);
}

pub fn to_changeset(body: &String) -> Result<HashMap<String, Option<String>>, XMLError> {
    let mut reader = quick_xml::reader::Reader::from_str(body);
    let mut buf = Vec::new();

	let mut map = HashMap::new();

	 loop {
        match reader.read_event(&mut buf) {
            Ok(XMLEvents::Event::Start(ref e)) => {
                match e.name() {
                    b"tag" => { to_changeset_tag(&e, &mut map) },
                    _ => (),
                }
            },
            Ok(XMLEvents::Event::Empty(ref e)) => {
                match e.name() {
                    b"tag" => { to_changeset_tag(&e, &mut map) },
                    _ => (),
                }
            },
            Ok(XMLEvents::Event::Eof) => { break; },
            Err(_) => { return Err(XMLError::Invalid); },
            _ => ()
        }

        buf.clear()
	}

    Ok(map)
}

pub fn to_features(body: &String) -> Result<geojson::FeatureCollection, XMLError> {
    Err(XMLError::Unknown)
}

pub fn from(fc: &geojson::FeatureCollection) -> Result<String, XMLError> {
    let mut xml: String = String::from(r#"<?xml version="1.0" encoding="UTF-8"?><osm version="0.6" generator="ROSM">"#);
    let mut osm = OSMTypes::new();

    for feat in &fc.features {
        match feat.geometry {
            Some(ref geom) => {
                match geom.value {
                    geojson::Value::Point(ref coords) => {
                        point(&feat, &coords, &mut osm);
                    },
                    geojson::Value::MultiPoint(ref coords) => {
                        multipoint(&feat, &coords, &mut osm);
                    },
                    geojson::Value::LineString(ref coords) => {
                        linestring(&feat, &coords, &mut osm);
                    },
                    geojson::Value::MultiLineString(ref coords) => {
                        multilinestring(&feat, &coords, &mut osm);
                    },
                    geojson::Value::Polygon(ref coords) => {
                        polygon(&feat, &coords, &mut osm);
                    },
                    geojson::Value::MultiPolygon(ref coords) => {
                        multipolygon(&feat, &coords, &mut osm);
                    },
                    _ => {
                        return Err(XMLError::GCNotSupported);
                    }
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

	match *&feat.properties {
		Some(ref props) => {
			for (k, v) in props.iter() {
				let mut xml_tag = XMLEvents::BytesStart::owned(b"tag".to_vec(), 3);
				xml_tag.push_attribute(("k", k.as_str()));
				xml_tag.push_attribute(("v", v.as_str().unwrap()));
				writer.write_event(XMLEvents::Event::Empty(xml_tag)).unwrap();
			}
		},
        None => { return Err(XMLError::Unknown); }
	};

    writer.write_event(XMLEvents::Event::End(XMLEvents::BytesEnd::borrowed(b"node"))).unwrap();

    osm.nodes.push_str(&*String::from_utf8(writer.into_inner().into_inner()).unwrap());

	Ok(true)
}

pub fn multipoint(feat: &geojson::Feature, coords: &Vec<geojson::PointType>, osm: &mut OSMTypes) { }

pub fn linestring(feat: &geojson::Feature, coords: &geojson::LineStringType, osm: &mut OSMTypes) -> Result<bool, XMLError> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    let mut xml_way = XMLEvents::BytesStart::owned(b"way".to_vec(), 3);
    xml_way.push_attribute(("id", "1"));
    xml_way.push_attribute(("version", "1"));

    writer.write_event(XMLEvents::Event::Start(xml_way)).unwrap();

    for nd in coords {
        let node = match add_node(&nd, osm) {
            Ok(node) => node,
            Err(_) => { return Err(XMLError::EncodingFailed); }
        };

        osm.nodes.push_str(&*node.0);

        let mut xml_nd = XMLEvents::BytesStart::owned(b"nd".to_vec(), 2);
        xml_nd.push_attribute(("ref", &*node.1.to_string()));
        writer.write_event(XMLEvents::Event::Empty(xml_nd)).unwrap();
    }

	match *&feat.properties {
		Some(ref props) => {
			for (k, v) in props.iter() {
				let mut xml_tag = XMLEvents::BytesStart::owned(b"tag".to_vec(), 3);
				xml_tag.push_attribute(("k", k.as_str()));
				xml_tag.push_attribute(("v", v.as_str().unwrap()));
				writer.write_event(XMLEvents::Event::Empty(xml_tag)).unwrap();
			}
		},
        None => { return Err(XMLError::Unknown); }
	};

    writer.write_event(XMLEvents::Event::End(XMLEvents::BytesEnd::borrowed(b"way"))).unwrap();

    osm.ways.push_str(&*String::from_utf8(writer.into_inner().into_inner()).unwrap());

	Ok(true)
}
pub fn multilinestring(feat: &geojson::Feature, coords: &Vec<geojson::LineStringType>, osm: &mut OSMTypes) { }

pub fn polygon(feat: &geojson::Feature, coords: &geojson::PolygonType, osm: &mut OSMTypes) {

}
pub fn multipolygon(feat: &geojson::Feature, coords: &Vec<geojson::PolygonType>, osm: &mut OSMTypes) { }

pub fn add_node(coords: &geojson::PointType, osm: &mut OSMTypes) -> Result<(String, i64), XMLError> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    let mut xml_node = XMLEvents::BytesStart::owned(b"node".to_vec(), 4);

    osm.node_it = osm.node_it - 1;

    xml_node.push_attribute(("id", &*osm.node_it.to_string()));
    xml_node.push_attribute(("version", "1"));
    xml_node.push_attribute(("lat", &*coords[0].to_string()));
    xml_node.push_attribute(("lon", &*coords[1].to_string()));

    writer.write_event(XMLEvents::Event::Empty(xml_node)).unwrap();

	Ok((String::from_utf8(writer.into_inner().into_inner()).unwrap(), osm.node_it))
}
