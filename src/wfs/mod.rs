
use err::HecateError;

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

pub enum RequestType {
    GetCapabilities,
    DescribeFeatureType,
    GetFeature,
    Invalid
}

pub struct Query {
    pub service: String,
    pub version: String,
    pub request: RequestType,
    pub typenames: Option<String>,
    pub startindex: Option<u32>,
    pub count: Option<u32>,
    pub srsname: Option<String>,
    pub bbox: Option<String>,  //TODO this should be a vec of EPSG:4326 coords,
    pub resulttype: Option<String>
}

impl Query {
    pub fn new(req: &Req) -> Self {
        Query {
            service: Query::std_service(&req),
            version: Query::std_version(&req),
            request: Query::std_request(&req),
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

    fn std_resulttype(req: &Req) -> Option<String> {
        if req.resulttype.is_some() {
            return Some(req.resulttype.clone().unwrap());
        } else if req.resultType.is_some() {
            return Some(req.resultType.clone().unwrap());
        } else if req.RESULTTYPE.is_some() {
            return Some(req.RESULTTYPE.clone().unwrap());
        } else {
            return None;
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

    fn std_typenames(req: &Req) -> Option<String> {
        if req.typename.is_some() {
            return Some(req.typename.clone().unwrap());
        } else if req.typeName.is_some() {
            return Some(req.typeName.clone().unwrap());
        } else if req.TYPENAME.is_some() {
            return Some(req.TYPENAME.clone().unwrap());
        } else if req.typenames.is_some() {
            return Some(req.typenames.clone().unwrap());
        } else if req.typeNames.is_some() {
            return Some(req.typeNames.clone().unwrap());
        } else if req.TYPENAMES.is_some() {
            return Some(req.TYPENAMES.clone().unwrap());
        } else {
            return None;
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

pub fn capabilities(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, host: &String) -> Result<String, HecateError> {
    match conn.query("
        SELECT
            ST_XMin(extent.extent)||' '|| ST_YMin(extent.extent) as lower,
            ST_XMax(extent.extent)||' '||ST_YMax(extent.extent) as upper
        FROM (
            SELECT
                ST_EstimatedExtent('geo', 'geom') AS extent
        ) as extent
    ", &[]) {
        Ok(res) => {
            let lower: String = res.get(0).get(0);
            let upper: String = res.get(0).get(1);

            Ok(format!(r#"
                <WFS_Capabilities version="2.0.0" schemaLocation="http://www.opengis.net/wfs/2.0 http://ggcity.org:80/geoserver/schemas/wfs/2.0/wfs.xsd" updateSequence="4682">
                    <ServiceIdentification>
                        <Title>WFS</Title>
                        <Abstract></Abstract>
                        <Keywords><Keyword></Keyword></Keywords>
                        <ServiceType>WFS</ServiceType>
                        <ServiceTypeVersion>2.0.0</ServiceTypeVersion>
                        <Fees></Fees>
                        <AccessConstraints></AccessConstraints>
                    </ServiceIdentification>
                    <ServiceProvider>
                        <ProviderName></ProviderName>
                        <ServiceContact>
                            <IndividualName></IndividualName>
                            <PositionName></PositionName>
                            <ContactInfo>
                                <Phone>
                                    <Voice></Voice>
                                    <Facsimile></Facsimile>
                                </Phone>
                                <Address>
                                    <DeliveryPoint></DeliveryPoint>
                                    <City></City>
                                    <AdministrativeArea></AdministrativeArea>
                                    <PostalCode></PostalCode>
                                    <Country></Country>
                                    <ElectronicMailAddress></ElectronicMailAddress>
                                </Address>
                                <OnlineResource href="/admin"/>
                                <HoursOfService></HoursOfService>
                                <ContactInstructions></ContactInstructions>
                            </ContactInfo>
                            <role></role>
                        </ServiceContact>
                    </ServiceProvider>
                    <OperationsMetadata>
                        <Operation name="GetCapabilities">
                            <DCP><HTTP>
                                    <Get href="{host}/api/wfs?"/>
                                    <Post href="{host}/api/wfs"/>
                            </HTTP></DCP>
                            <Parameter name="AcceptVersions">
                                <AllowedValues><Value>2.0.0</Value></AllowedValues>
                            </Parameter>
                        </Operation>
                        <Operation name="DescribeFeatureType">
                            <DCP><HTTP>
                                <Get href="{host}/api/wfs?"/>
                                <Post href="{host}/api/wfs"/>
                            </HTTP></DCP>
                            <Parameter name="outputFormat">
                                <AllowedValues>
                                    <Value>text/xml; subtype=gml/3.2</Value>
                                </AllowedValues>
                            </Parameter>
                        </Operation>
                        <Operation name="GetPropertyValue">
                            <DCP><HTTP>
                                <Get href="{host}/api/wfs?"/>
                                <Post href="{host}/api/wfs"/>
                            </HTTP></DCP>
                        </Operation>
                        <Operation name="GetFeature">
                            <DCP><HTTP>
                                <Get href="{host}/api/wfs?"/>
                                <Post href="{host}/api/wfs"/>
                            </HTTP></DCP>
                            <Parameter name="resultType">
                                <AllowedValues>
                                    <Value>results</Value>
                                    <Value>hits</Value>
                                </AllowedValues>
                            </Parameter>
                            <Parameter name="outputFormat">
                                <AllowedValues>
                                    <Value>GML32</Value>
                                    <Value>GML32+ZIP</Value>
                                    <Value>application/gml+xml; version=3.2</Value>
                                    <Value>GML3</Value>
                                    <Value>GML3+ZIP</Value>
                                    <Value>text/xml; subtype=gml/3.1.1</Value>
                                    <Value>GML2</Value>
                                    <Value>GML2+ZIP</Value>
                                    <Value>text/xml; subtype=gml/2.1.2</Value>
                                    <Value>GEOJSON</Value>
                                    <Value>GEOJSON+ZIP</Value>
                                    <Value>ESRIGEOJSON</Value>
                                    <Value>ESRIGEOJSON+ZIP</Value>
                                    <Value>KML</Value>
                                    <Value>application/vnd.google-earth.kml xml</Value>
                                    <Value>application/vnd.google-earth.kml+xml</Value>
                                    <Value>KMZ</Value>
                                    <Value>application/vnd.google-earth.kmz</Value>
                                    <Value>SHAPE+ZIP</Value>
                                    <Value>CSV</Value>
                                    <Value>CSV+ZIP</Value>
                                </AllowedValues>
                            </Parameter>
                        </Operation>
                        <Operation name="GetGmlObject">
                            <DCP><HTTP>
                                <Get href="{host}/api/wfs?"/>
                                <Post href="{host}/api/wfs"/>
                            </HTTP></DCP>
                        </Operation>
                        <Operation name="ListStoredQueries">
                            <DCP><HTTP>
                                <Get href="{host}/api/wfs?"/>
                                <Post href="{host}/api/wfs"/>
                            </HTTP></DCP>
                        </Operation>
                        <Operation name="DescribeStoredQueries">
                            <DCP><HTTP>
                                <Get href="{host}/api/wfs?"/>
                                <Post href="{host}/api/wfs"/>
                            </HTTP></DCP>
                        </Operation>
                        <Constraint name="ImplementsBasicWFS"><NoValues/><DefaultValue>TRUE</DefaultValue></Constraint>
                        <Constraint name="ImplementsTransactionalWFS"><NoValues/><DefaultValue>FALSE</DefaultValue></Constraint>
                        <Constraint name="KVPEncoding"><NoValues/><DefaultValue>TRUE</DefaultValue></Constraint>
                        <Constraint name="XMLEncoding"><NoValues/><DefaultValue>TRUE</DefaultValue></Constraint>
                        <Constraint name="SOAPEncoding"><NoValues/><DefaultValue>FALSE</DefaultValue></Constraint>
                        <Constraint name="ImplementsInheritance"><NoValues/><DefaultValue>FALSE</DefaultValue></Constraint>
                        <Constraint name="ImplementsRemoteResolve"><NoValues/><DefaultValue>FALSE</DefaultValue></Constraint>
                        <Constraint name="ImplementsResultPaging"><NoValues/><DefaultValue>TRUE</DefaultValue></Constraint>
                        <Constraint name="ImplementsStandardJoins"><NoValues/><DefaultValue>FALSE</DefaultValue></Constraint>
                        <Constraint name="ImplementsSpatialJoins"><NoValues/><DefaultValue>FALSE</DefaultValue></Constraint>
                        <Constraint name="ImplementsTemporalJoins"><NoValues/><DefaultValue>FALSE</DefaultValue></Constraint>
                        <Constraint name="ImplementsFeatureVersioning"><NoValues/><DefaultValue>FALSE</DefaultValue></Constraint>
                        <Constraint name="ManageStoredQueries"><NoValues/><DefaultValue>FALSE</DefaultValue></Constraint>
                    </OperationsMetadata>
                    <FeatureTypeList>
                        <FeatureType>
                            <Name>HecateData</Name>
                            <Title>Hecate Data</Title>
                            <DefaultCRS>urn:ogc:def:crs:EPSG::4326</DefaultCRS>
                            <WGS84BoundingBox>
                                <LowerCorner>{lower}</LowerCorner>
                                <UpperCorner>{upper}</UpperCorner>
                            </WGS84BoundingBox>
                        </FeatureType>
                    </FeatureTypeList>
                    <Filter_Capabilities>
                        <Conformance>
                            <Constraint name="ImplementsQuery"><NoValues/><DefaultValue>TRUE</DefaultValue></Constraint>
                            <Constraint name="ImplementsAdHocQuery"><NoValues/><DefaultValue>TRUE</DefaultValue></Constraint>
                            <Constraint name="ImplementsFunctions"><NoValues/><DefaultValue>TRUE</DefaultValue></Constraint>
                            <Constraint name="ImplementsResourceId"><NoValues/><DefaultValue>TRUE</DefaultValue></Constraint>
                            <Constraint name="ImplementsMinStandardFilter"><NoValues/><DefaultValue>TRUE</DefaultValue></Constraint>
                            <Constraint name="ImplementsStandardFilter"><NoValues/><DefaultValue>TRUE</DefaultValue></Constraint>
                            <Constraint name="ImplementsMinSpatialFilter"><NoValues/><DefaultValue>TRUE</DefaultValue></Constraint>
                            <Constraint name="ImplementsSpatialFilter"><NoValues/><DefaultValue>TRUE</DefaultValue></Constraint>
                            <Constraint name="ImplementsMinTemporalFilter"><NoValues/><DefaultValue>FALSE</DefaultValue></Constraint>
                            <Constraint name="ImplementsTemporalFilter"><NoValues/><DefaultValue>FALSE</DefaultValue></Constraint>
                            <Constraint name="ImplementsVersionNav"><NoValues/><DefaultValue>FALSE</DefaultValue></Constraint>
                            <Constraint name="ImplementsSorting"><NoValues/><DefaultValue>TRUE</DefaultValue></Constraint>
                            <Constraint name="ImplementsExtendedOperators"><NoValues/><DefaultValue>FALSE</DefaultValue></Constraint>
                            <Constraint name="ImplementsMinimumXPath"><NoValues/><DefaultValue>TRUE</DefaultValue></Constraint>
                            <Constraint name="ImplementsSchemaElementFunc"><NoValues/><DefaultValue>FALSE</DefaultValue></Constraint>
                        </Conformance>
                        <Id_Capabilities>
                            <ResourceIdentifier name="ResourceId"/>
                        </Id_Capabilities>
                        <Scalar_Capabilities>
                            <LogicalOperators/>
                            <ComparisonOperators>
                                <ComparisonOperator name="PropertyIsEqualTo"/>
                                <ComparisonOperator name="PropertyIsNotEqualTo"/>
                                <ComparisonOperator name="PropertyIsLessThan"/>
                                <ComparisonOperator name="PropertyIsGreaterThan"/>
                                <ComparisonOperator name="PropertyIsLessThanOrEqualTo"/>
                                <ComparisonOperator name="PropertyIsGreaterThanOrEqualTo"/>
                                <ComparisonOperator name="PropertyIsLike"/>
                                <ComparisonOperator name="PropertyIsNull"/>
                                <ComparisonOperator name="PropertyIsNil"/>
                                <ComparisonOperator name="PropertyIsBetween"/>
                            </ComparisonOperators>
                        </Scalar_Capabilities>
                        <Spatial_Capabilities>
                            <GeometryOperands xmlns:gml="http://www.opengis.net/gml" xmlns:gml32="http://www.opengis.net/gml">
                                <GeometryOperand name="gml:Box"/>
                                <GeometryOperand name="gml:Envelope"/>
                                <GeometryOperand name="gml:Point"/>
                                <GeometryOperand name="gml:LineString"/>
                                <GeometryOperand name="gml:LinearRing"/>
                                <GeometryOperand name="gml:Polygon"/>
                                <GeometryOperand name="gml:MultiPoint"/>
                                <GeometryOperand name="gml:MultiCurve"/>
                                <GeometryOperand name="gml:MultiSurface"/>
                            </GeometryOperands>
                            <SpatialOperators>
                                <SpatialOperator name="BBOX"/>
                                <SpatialOperator name="Equals"/>
                                <SpatialOperator name="Disjoint"/>
                                <SpatialOperator name="Intersects"/>
                                <SpatialOperator name="Touches"/>
                                <SpatialOperator name="Crosses"/>
                                <SpatialOperator name="Within"/>
                                <SpatialOperator name="Contains"/>
                                <SpatialOperator name="Overlaps"/>
                                <SpatialOperator name="Beyond"/>
                                <SpatialOperator name="DWithin"/>
                            </SpatialOperators>
                        </Spatial_Capabilities>
                    </Filter_Capabilities>
                </WFS_Capabilities>
            "#,
                lower = lower,
                upper = upper,
                host = host
            ))
        },
        Err(err) => Err(HecateError::from_db(err))
    }
}

pub fn describe_feature_type(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>) -> Result<String, HecateError> {
    match conn.query("
        SELECT 1;
    ", &[]) {
        Ok(res) => {
            Ok(format!(r#"
                <xsd:schema xmlns:gml="http://www.opengis.net/gml/3.2" xmlns:publicgis="http://bloomington.in.gov/publicgis" xmlns:wfs="http://www.opengis.net/wfs/2.0" xmlns:xsd="http://www.w3.org/2001/XMLSchema" elementFormDefault="qualified" targetNamespace="http://bloomington.in.gov/publicgis">
                    <xsd:import namespace="http://www.opengis.net/gml/3.2" schemaLocation="https://tarantula.bloomington.in.gov:443/geoserver/schemas/gml/3.2.1/gml.xsd"/>
                    <xsd:complexType name="HecateDataType">
                        <xsd:complexContent>
                            <xsd:extension base="gml:AbstractFeatureType">
                                <xsd:sequence>
                                    <xsd:element maxOccurs="1" minOccurs="0" name="the_geom" nillable="true" type="gml:PointPropertyType"/>
                                    <xsd:element maxOccurs="1" minOccurs="0" name="TAG" nillable="true" type="xsd:string"/>
                            </xsd:sequence>
                        </xsd:extension>
                        </xsd:complexContent>
                    </xsd:complexType>
                    <xsd:element name="HecateData" substitutionGroup="gml:AbstractFeature" type="HecateDataType"/>
                </xsd:schema>
            "#))
        },
        Err(err) => Err(HecateError::from_db(err))
    }
}

pub fn get_feature(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, query: &Query) -> Result<String, HecateError> {
    if query.srsname.is_some() && query.srsname != Some(String::from("urn:ogc:def:crs:EPSG::4326")) {
        let mut err = HecateError::new(400, String::from("Only srsname=urn:ogc:def:crs:EPSG::4326 supported"), None);
        err.to_wfsxml();
        return Err(err);
    }

    if query.typenames.is_some() && query.typenames != Some(String::from("HecateData")) {
        let mut err = HecateError::new(400, String::from("Only typenames=HecateData supported"), None);
        err.to_wfsxml();
        return Err(err);
    }

    match conn.query("
        SELECT 1;
    ", &[]) {
        Ok(res) => {
            Ok(format!(r#""#))
        },
        Err(err) => Err(HecateError::from_db(err))
    }
}
