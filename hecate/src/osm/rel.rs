use std::fmt;
use crate::osm::*;

pub struct Rel {
    pub id: Option<i64>,
    pub user: Option<String>,
    pub uid: Option<i32>,
    pub modified: bool,
    pub tags: serde_json::Map<String, serde_json::Value>,
    pub action: Option<Action>,
    pub version: Option<i32>,
    pub parents: Vec<i64>,
    pub members: Vec<RelMem>
}

pub struct RelMem {
    pub rtype: Option<Value>,
    pub rref: Option<i64>,
    pub rrole: Option<String>
}

impl Rel {
    pub fn set_member(&mut self, _rtype: Option<Value>, _rref: Option<i64>, _rrole: Option<String>) {

    }

}

impl Generic for Rel {
    fn new() -> Rel {
        Rel {
            id: None,
            modified: false,
            user: None,
            uid: None,
            tags: serde_json::Map::new(),
            action: None,
            parents: Vec::new(),
            version: None,
            members: Vec::new()
        }
    }

    fn value(&self) -> Value {
        Value::Rel
    }

    fn set_tag(&mut self, k: String, v: String) {
        let v_unescape = unescape(v);

        let value = match serde_json::from_str::<serde_json::Value>(&*v_unescape) {
            Ok(value) => value,
            Err(_) => serde_json::Value::String(v_unescape)
        };
        self.tags.insert(k, value);
    }

    fn has_tags(&self) -> bool {
        !self.tags.is_empty()
    }

    fn to_feat(&self, _tree: &OSMTree) -> Result<geojson::Feature, XMLError> {
        Ok(geojson::Feature {
            bbox: None,
            geometry: None,
            id: None,
            properties: None,
            foreign_members: None
        })
    }

    fn is_valid(&self) -> Result<bool, String> {
        match self.id {
            None => { return Err(String::from("Missing id")); },
            Some(_) => ()
        }
        match self.version {
            None => { return Err(String::from("Missing version")); },
            Some(_) => ()
        }

        Ok(true)
    }
}

impl fmt::Display for Rel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Rel: id={}]", self.id.unwrap())
    }
}

impl RelMem {
    pub fn new(rtype: Option<Value>, rref: Option<i64>, rrole: Option<String>) -> RelMem {
        RelMem {
            rtype,
            rref,
            rrole
        }
    }
}
