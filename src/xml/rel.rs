use std::fmt;
use std::collections::HashMap;
use xml::*;

pub struct Rel {
    pub id: Option<i64>,
    pub user: Option<String>,
    pub uid: Option<i32>,
    pub modified: bool,
    pub tags: HashMap<String, String>,
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
    pub fn set_member(&mut self, rtype: Option<Value>, rref: Option<i64>, rrole: Option<String>) {

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

    fn is_valid(&self) -> bool {
        match self.id {
            None => { return false; },
            Some(_) => ()
        }
        match self.version {
            None => { return false; },
            Some(_) => ()
        }

        return true;
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
