use crate::err::HecateError;

mod capabilities;
mod get_feature;
mod describe_feature_type;

pub use self::capabilities::capabilities;
pub use self::get_feature::get_feature;
pub use self::describe_feature_type::describe_feature_type;

///
/// Req is used by Rocket to handle the user's request on the GET wfs? endpoint
///
/// It must be converted to a WFSQuery before it is used by any of the functions in this mod
///
#[derive(FromForm, Debug)]
pub struct Req {
    pub service: Option<String>,
    pub version: Option<String>,
    pub request: Option<String>,
    pub SERVICE: Option<String>,
    pub VERSION: Option<String>,
    pub REQUEST: Option<String>,

    //Optional Params
    pub TYPENAME: Option<String>,
    pub TYPENAMES: Option<String>,
    pub typename: Option<String>,
    pub typeName: Option<String>,
    pub typenames: Option<String>,
    pub typeNames: Option<String>,

    pub namespaces: Option<String>,
    pub NAMESPACES: Option<String>,

    pub RESULTTYPE: Option<String>,
    pub resultType: Option<String>,
    pub resulttype: Option<String>,

    pub STARTINDEX: Option<u32>,
    pub startIndex: Option<u32>,
    pub startindex: Option<u32>,

    pub COUNT: Option<u32>,
    pub count: Option<u32>,

    pub SRSNAME: Option<String>,
    pub SRSName: Option<String>,
    pub srsname: Option<String>,

    pub BBOX: Option<String>,
    pub bbox: Option<String>
}

#[derive(PartialEq, Debug)]
pub enum RequestType {
    GetCapabilities,
    DescribeFeatureType,
    GetFeature,
    Invalid
}

#[derive(PartialEq, Debug)]
pub enum ResultType {
    Hits,
    Results,
    Invalid
}

#[derive(PartialEq, Debug)]
pub struct Query {
    pub service: String,
    pub version: String,
    pub request: RequestType,
    pub namespaces: Option<String>,
    pub typenames: Option<String>,
    pub startindex: Option<u32>,
    pub count: Option<u32>,
    pub srsname: Option<String>,
    pub bbox: Option<String>,  //TODO this should be a vec of EPSG:4326 coords,
    pub resulttype: ResultType
}

impl Query {
    pub fn new(req: &Req) -> Self {
        Query {
            service: Query::std_service(&req),
            version: Query::std_version(&req),
            request: Query::std_request(&req),
            namespaces: Query::std_namespaces(&req),
            typenames: Query::std_typenames(&req),
            startindex: Query::std_startindex(&req),
            count: Query::std_count(&req),
            srsname: Query::std_srsname(&req),
            bbox: Query::std_bbox(&req),
            resulttype: Query::std_resulttype(&req)
        }
    }

    fn std_bbox(req: &Req) -> Option<String> {
        if req.bbox.is_some() {
            return Some(req.bbox.clone().unwrap());
        } else if req.BBOX.is_some() {
            return Some(req.BBOX.clone().unwrap());
        } else {
            return None;
        }
    }

    fn std_count(req: &Req) -> Option<u32> {
        if req.count.is_some() {
            return Some(req.count.clone().unwrap());
        } else if req.COUNT.is_some() {
            return Some(req.COUNT.clone().unwrap());
        } else {
            return None;
        }
    }

    fn std_resulttype(req: &Req) -> ResultType {
        let res = String::from("results");
        let restype: &String = match req.resulttype {
            Some(ref resulttype) => resulttype,
            None => match req.RESULTTYPE {
                Some(ref resulttype) => resulttype,
                None => match req.resultType {
                    Some(ref resulttype) => resulttype,
                    None => &res
                }
            }
        };

        if restype == "results" {
            ResultType::Results
        } else if restype == "hits" {
            ResultType::Hits
        } else {
            ResultType::Invalid
        }
    }

    fn std_srsname(req: &Req) -> Option<String> {
        if req.srsname.is_some() {
            return Some(req.srsname.clone().unwrap());
        } else if req.SRSName.is_some() {
            return Some(req.SRSName.clone().unwrap());
        } else if req.SRSNAME.is_some() {
            return Some(req.SRSNAME.clone().unwrap());
        } else {
            return None;
        }
    }

    fn std_startindex(req: &Req) -> Option<u32> {
        if req.startindex.is_some() {
            return Some(req.startindex.clone().unwrap());
        } else if req.startIndex.is_some() {
            return Some(req.startIndex.clone().unwrap());
        } else if req.STARTINDEX.is_some() {
            return Some(req.STARTINDEX.clone().unwrap());
        } else {
            return None;
        }
    }

    fn std_namespaces(req: &Req) -> Option<String> {
        if req.namespaces.is_some() {
            return Some(req.namespaces.clone().unwrap());
        } else if req.NAMESPACES.is_some() {
            return Some(req.NAMESPACES.clone().unwrap());
        } else {
            return None;
        }
    }

    fn std_typenames(req: &Req) -> Option<String> {
        let typename: Option<String>;

        if req.typename.is_some() {
            typename = Some(req.typename.clone().unwrap());
        } else if req.typeName.is_some() {
            typename = Some(req.typeName.clone().unwrap());
        } else if req.TYPENAME.is_some() {
            typename = Some(req.TYPENAME.clone().unwrap());
        } else if req.typenames.is_some() {
            typename = Some(req.typenames.clone().unwrap());
        } else if req.typeNames.is_some() {
            typename = Some(req.typeNames.clone().unwrap());
        } else if req.TYPENAMES.is_some() {
            typename =  Some(req.TYPENAMES.clone().unwrap());
        } else {
            typename = None;
        }

        match typename {
            Some(typename) => Some(typename.replace("wfs:", "")),
            None => None
        }
    }

    fn std_version(req: &Req) -> String {
        if req.version.is_some() {
            return req.version.clone().unwrap();
        } else if  req.VERSION.is_some() {
            return req.VERSION.clone().unwrap();
        } else {
            return String::from("2.0.0");
        }
    }

    fn std_service(req: &Req) -> String {
        if req.service.is_some() {
            return req.service.clone().unwrap();
        } else if  req.SERVICE.is_some() {
            return req.SERVICE.clone().unwrap();
        } else {
            return String::from("WFS");
        }
    }

    fn std_request(req: &Req) -> RequestType {
        let request: String = match req.request {
            Some(ref request) => request.clone(),
            None => match req.REQUEST {
                Some(ref request) => request.clone(),
                None => String::from("")
            }
        };

        if request == "GetCapabilities" {
            RequestType::GetCapabilities
        } else if request == "DescribeFeatureType" {
            RequestType::DescribeFeatureType
        } else if request == "GetFeature" {
            RequestType::GetFeature
        } else {
            RequestType::Invalid
        }
    }
}
