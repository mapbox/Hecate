use std::fmt;
use std::collections::HashMap;
use xml::*;

pub struct OSMTree {
    pub meta: HashMap<String, String>,
    nodes: HashMap<i64, Node>,
    ways: HashMap<i64, Way>,
    rels: HashMap<i64, Rel>
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

    pub fn add_node(&mut self, node: Node) -> Result<bool, XMLError> {
        match node.is_valid() {
            Err(err) => { return Err(XMLError::InvalidNode(err)); }
            _ => ()
        }

        self.nodes.insert(node.id.unwrap(), node);
        Ok(true)
    }

    pub fn get_nodes(&self) -> &HashMap<i64, Node> {
        &self.nodes
    }

    pub fn get_nodes_mut(&mut self) -> &mut HashMap<i64, Node> {
        &mut self.nodes
    }

    pub fn get_node_mut(&mut self, id: &i64) -> Result<&mut Node, XMLError> {
        match self.nodes.get_mut(&id) {
            Some(n) => Ok(n),
            None => Err(XMLError::NotFoundError)
        }
    }

    pub fn get_node(&self, id: &i64) -> Result<&Node, XMLError> {
        match self.nodes.get(&id) {
            Some(n) => Ok(n),
            None => Err(XMLError::NotFoundError)
        }
    }

    pub fn add_way(&mut self, way: Way) -> Result<bool, XMLError> {
        match way.is_valid() {
            Err(err) => { return Err(XMLError::InvalidWay(err)); },
            _ => ()
        }

        for nd in &way.nodes {
            if !self.nodes.contains_key(&nd) {
                return Err(XMLError::InvalidNodeRef);
            }

            match self.nodes.get_mut(&nd) {
                Some(nd) => nd.parents.push(way.id.unwrap()),
                None => { return Err(XMLError::InternalError) }
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

    pub fn get_way(&self, id: &i64) -> Result<&Way, XMLError> {
        match self.ways.get(&id) {
            Some(n) => Ok(n),
            None => Err(XMLError::NotFoundError)
        }
    }

    pub fn get_way_mut(&mut self, id: &i64) -> Result<&mut Way, XMLError> {
        match self.ways.get_mut(&id) {
            Some(n) => Ok(n),
            None => Err(XMLError::NotFoundError)
        }
    }

    pub fn add_rel(&mut self, rel: Rel) -> Result<bool, XMLError> {
        match rel.is_valid() {
            Err(err) => { return Err(XMLError::InvalidRel(err)); },
            _ => ()
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

    pub fn get_rel(&self, id: &i64) -> Result<&Rel, XMLError> {
        match self.rels.get(&id) {
            Some(n) => Ok(n),
            None => Err(XMLError::NotFoundError)
        }
    }

    pub fn get_rel_mut(&mut self, id: &i64) -> Result<&mut Rel, XMLError> {
        match self.rels.get_mut(&id) {
            Some(n) => Ok(n),
            None => Err(XMLError::NotFoundError)
        }
    }
}

impl fmt::Display for OSMTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[OSMTree: nodes={} ways={} rels={}]", self.nodes.len(), self.ways.len(), self.rels.len())
    }
}
