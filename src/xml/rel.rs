use std::fmt;
use xml::*;

pub struct Rel {
    pub id: Option<i64>,
    pub user: Option<String>,
    pub uid: Option<i32>,
    pub modified: bool,
    pub tags: HashMap<String, String>,
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
            tags: HashMap::new(),
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
        self.tags.insert(k, v);
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

        return Ok(true);
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
            rtype: rtype,
            rref: rref,
            rrole: rrole
        }
    }
}
