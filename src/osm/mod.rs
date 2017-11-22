extern crate quick_xml;

mod node;
mod way;
mod rel;
pub use osm::node::Node;
pub use osm::way::Way;
pub use osm::rel::Rel;

use std::fmt;
use std::collections::HashMap;
use std::string;
use std::num;

pub enum OSMError {
    InternalError,
    ParsingError,
    InvalidNode,
    InvalidNodeRef,
    InvalidWay,
    InvalidWayRef,
    InvalidRel,
    InvalidXML,
    NotFoundError,
    StringParsing(string::ParseError),
    IntParsing(num::ParseIntError),
    FloatParsing(num::ParseFloatError)
}

impl From<string::FromUtf8Error> for OSMError {
    fn from(error: string::FromUtf8Error) -> OSMError {
        OSMError::ParsingError
    }
}

impl From<quick_xml::errors::Error> for OSMError {
    fn from(error: quick_xml::errors::Error) -> OSMError {
        OSMError::InvalidXML
    }
}

impl From<string::ParseError> for OSMError {
    fn from(error: string::ParseError) -> OSMError {
        OSMError::StringParsing(error)
    }
}

impl From<num::ParseFloatError> for OSMError {
    fn from(error: num::ParseFloatError) -> OSMError {
        OSMError::FloatParsing(error)
    }
}

impl From<num::ParseIntError> for OSMError {
    fn from(error: num::ParseIntError) -> OSMError {
        OSMError::IntParsing(error)
    }
}

pub struct OSMTree {
    pub meta: HashMap<String, String>,
    nodes: HashMap<i64, Node>,
    ways: HashMap<i64, Way>,
    rels: HashMap<i64, Rel>
}

#[derive(PartialEq)]
pub enum Value {
    None,
    Node,
    Way,
    Rel
}

pub trait Generic {
    fn new() -> Self;
    fn value(&self) -> Value;
    fn set_tag(&mut self, k: String, v: String);
    fn is_valid(&self) -> bool;
}

impl OSMTree {
    pub fn new() -> OSMTree {
        OSMTree {
            meta: HashMap::new(),
            nodes: HashMap::new(),
            ways: HashMap::new(),
            rels: HashMap::new()
        }
    }

    pub fn add_node(&mut self, node: Node) -> Result<bool, OSMError> {
        if !node.is_valid() {
            return Err(OSMError::InvalidNode);
        }

        self.nodes.insert(node.id.unwrap(), node);
        Ok(true)
    }
    pub fn get_nodes(& self) -> &HashMap<i64, Node> {
        &self.nodes
    }
    pub fn get_nodes_mut(&mut self) -> &mut HashMap<i64, Node> {
        &mut self.nodes
    }
    pub fn get_node_mut(&mut self, id: &i64) -> Result<&mut Node, OSMError> {
        match self.nodes.get_mut(&id) {
            Some(n) => Ok(n),
            None => Err(OSMError::NotFoundError)
        }
    }
    pub fn get_node(&self, id: &i64) -> Result<&Node, OSMError> {
        match self.nodes.get(&id) {
            Some(n) => Ok(n),
            None => Err(OSMError::NotFoundError)
        }
    }
    pub fn add_way(&mut self, way: Way) -> Result<bool, OSMError> {
        if !way.is_valid() {
            return Err(OSMError::InvalidWay);
        }

        for nd in &way.nodes {
            if !self.nodes.contains_key(&nd) {
                return Err(OSMError::InvalidNodeRef);
            }

            match self.nodes.get_mut(&nd) {
                Some(nd) => nd.parents.push(way.id.unwrap()),
                None => { return Err(OSMError::InternalError) }
            }
        }

        self.ways.insert(way.id.unwrap(), way);
        Ok(true)
    }
    pub fn get_ways(&self) -> &HashMap<i64, Way> {
        &self.ways
    }
    pub fn get_ways_mut(&mut self) -> &mut HashMap<i64, Way> {
        &mut self.ways
    }
    pub fn get_way(&self, id: &i64) -> Result<&Way, OSMError> {
        match self.ways.get(&id) {
            Some(n) => Ok(n),
            None => Err(OSMError::NotFoundError)
        }
    }
    pub fn get_way_mut(&mut self, id: &i64) -> Result<&mut Way, OSMError> {
        match self.ways.get_mut(&id) {
            Some(n) => Ok(n),
            None => Err(OSMError::NotFoundError)
        }
    }

    pub fn add_rel(&mut self, rel: Rel) -> Result<bool, OSMError> {
        if !rel.is_valid() {
            return Err(OSMError::InvalidRel);
        }

        self.rels.insert(rel.id.unwrap(), rel);
        Ok(true)
    }
    pub fn get_rels(&self) -> &HashMap<i64, Rel> {
        &self.rels
    }
    pub fn get_rels_mut(&mut self) -> &mut HashMap<i64, Rel> {
        &mut self.rels
    }
    pub fn get_rel(&self, id: &i64) -> Result<&Rel, OSMError> {
        match self.rels.get(&id) {
            Some(n) => Ok(n),
            None => Err(OSMError::NotFoundError)
        }
    }
    pub fn get_rel_mut(&mut self, id: &i64) -> Result<&mut Rel, OSMError> {
        match self.rels.get_mut(&id) {
            Some(n) => Ok(n),
            None => Err(OSMError::NotFoundError)
        }
    }
}

impl fmt::Display for OSMTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[OSMTree: nodes={} ways={} rels={}]", self.nodes.len(), self.ways.len(), self.rels.len())
    }
}
