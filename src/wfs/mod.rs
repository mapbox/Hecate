
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
            Ok(format!(r#"
                <wfs:FeatureCollection xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:wfs="http://www.opengis.net/wfs/2.0" xmlns:publicgis="http://bloomington.in.gov/publicgis" xmlns:gml="http://www.opengis.net/gml/3.2" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" next="https://tarantula.bloomington.in.gov:443/geoserver/publicgis/wfs?REQUEST=GetFeature&amp;SRSNAME=urn%3Aogc%3Adef%3Acrs%3AEPSG%3A%3A4326&amp;VERSION=2.0.0&amp;TYPENAMES=PoliceDistricts&amp;SERVICE=WFS&amp;COUNT=1&amp;STARTINDEX=1" numberMatched="32" numberReturned="1" timeStamp="2018-12-29T18:43:32.160Z" xsi:schemaLocation="http://www.opengis.net/wfs/2.0 https://tarantula.bloomington.in.gov:443/geoserver/schemas/wfs/2.0/wfs.xsd http://www.opengis.net/gml/3.2 https://tarantula.bloomington.in.gov:443/geoserver/schemas/gml/3.2.1/gml.xsd http://bloomington.in.gov/publicgis https://tarantula.bloomington.in.gov:443/geoserver/publicgis/wfs?service=WFS&amp;version=2.0.0&amp;request=DescribeFeatureType&amp;typeName=publicgis%3APoliceDistricts">
                    <wfs:boundedBy>
                        <gml:Envelope>
                            <gml:lowerCorner>39.14403294075481 -86.56771624997428</gml:lowerCorner>
                            <gml:upperCorner>39.156019065043765 -86.55030991360255</gml:upperCorner>
                        </gml:Envelope>
                    </wfs:boundedBy>
                    <wfs:member>
                        <publicgis:PoliceDistricts gml:id="PoliceDistricts.1">
                            <gml:boundedBy><gml:Envelope srsDimension="2" srsName="urn:ogc:def:crs:EPSG::4326">
                                <gml:lowerCorner>39.14403294075481 -86.56771624997428</gml:lowerCorner>
                                <gml:upperCorner>39.156019065043765 -86.55030991360255</gml:upperCorner>
                            </gml:Envelope></gml:boundedBy>
                            <publicgis:the_geom>
                                <gml:MultiSurface srsDimension="2" srsName="urn:ogc:def:crs:EPSG::4326">
                                    <gml:surfaceMember>
                                        <gml:Polygon srsDimension="2">
                                            <gml:exterior>
                                                <gml:LinearRing><gml:posList>39.155767467243535 -86.55050022429107 39.15576360358754 -86.55050128313967 39.15575970291843 -86.55050205310287 39.155755773472634 -86.5505025341186 39.15575182623202 -86.55050272610413 39.15574788039873 -86.5505026253882 39.15558474188524 -86.55049242680535 39.155562679236674 -86.55049137660781 39.155540843102884 -86.55049062797143 39.15551900278399 -86.55049016500408 39.15549716104157 -86.55048999121104 39.15547531785953 -86.55049010306574 39.15545347872878 -86.55049050052656 39.15543164091997 -86.55049118714018 39.15540981265339 -86.55049215931793 39.15531331737389 -86.55037308740587 39.15529116921309 -86.55037447179578 39.15526903307377 -86.55037614349331 39.155246911150655 -86.55037810212892 39.155224805640124 -86.55038034768589 39.155202719009836 -86.55038287943997 39.15518065345777 -86.55038569772697 39.15515861117703 -86.55038880182484 39.15513659436399 -86.55039219171668 39.15511460548631 -86.55039586667843 39.155092646465796 -86.55039982669524 39.15507071977171 -86.55040407139566 39.15504882732264 -86.55040860005967 39.1550269718624 -86.55041341231362 39.153798288015174 -86.55033637679499 39.15365272456949 -86.55032725046327 39.15339395172743 -86.55030991360255 39.15340174880212 -86.55081572471627 39.15094794805074 -86.5506512247775 39.15095133757696 -86.55076708138813 39.150996787954895 -86.55232101701695 39.15107227066235 -86.55490333052305 39.1510746036882 -86.55495458805653 39.15122387047008 -86.55823531543099 39.15122933354697 -86.55835542767248 39.151277715975354 -86.55941928546332 39.15128315498553 -86.5595388959439 39.151303083133016 -86.5599771574394 39.151338752406 -86.56076169306299 39.150821568424135 -86.56110962801152 39.15068712976712 -86.56083541070826 39.15042408721962 -86.56029888503762 39.14877343761605 -86.56169417701383 39.14902770827193 -86.56250599244873 39.14885310748568 -86.56270440365857 39.148333772148206 -86.56307530480177 39.148328040743635 -86.56383490308241 39.14832712744787 -86.56395586553708 39.148324221127794 -86.56404394551743 39.148324333693395 -86.56413213236227 39.145892929482564 -86.5640461660507 39.14418848986796 -86.56400054615509 39.14404088926588 -86.56395275065194 39.14403294075481 -86.56406500039625 39.1441101142895 -86.56640817501558 39.144166737399324 -86.56640969266348 39.144493154055056 -86.56641844013883 39.14482803519018 -86.56642741488857 39.14516291630439 -86.56643638935317 39.14549779712474 -86.56644536388725 39.14583267820028 -86.56645433848685 39.14600011396822 -86.56645911391364 39.146068810080266 -86.56646154539024 39.146167528472006 -86.56646503896361 39.14650235526537 -86.5664768884455 39.146776803292916 -86.56648660108594 39.147784521347475 -86.56649998302494 39.147808354915405 -86.56760760579904 39.14925476790823 -86.56762291732299 39.14957552799258 -86.56762631290657 39.14969618015267 -86.56762758996581 39.15053420111965 -86.56763646119032 39.150867196499014 -86.56763998650767 39.15095241034937 -86.56771624997428 39.151314246698256 -86.56754831719043 39.15163829237959 -86.56735461135338 39.15163568639879 -86.56729721340658 39.15167968620241 -86.56733905228447 39.15175651605114 -86.56728519695626 39.15174993943593 -86.56726972090637 39.15204812411957 -86.56706070030731 39.15222397928623 -86.56693051115812 39.152238240921456 -86.56696064250926 39.15237689262143 -86.56684777403912 39.15264013265113 -86.56654513642593 39.152851729555955 -86.56631152499975 39.15301234947385 -86.5661148731474 39.153024289851004 -86.56610025401622 39.1530860960101 -86.56601815020088 39.15317725484026 -86.56589019855355 39.15327508387515 -86.56573912495332 39.153458777413775 -86.56544170408128 39.15346330685486 -86.56543437041658 39.15389805165807 -86.56473045788611 39.153902096112226 -86.56472390947181 39.154125723508166 -86.56436181887965 39.15414340174454 -86.56430290074199 39.154613483486024 -86.56356751484653 39.15493706087379 -86.56306130755725 39.15505681133022 -86.5628796523764 39.15534243793853 -86.56244636794459 39.15537096202842 -86.56240309739005 39.15540649592842 -86.56233184935782 39.15561219956274 -86.56202823606819 39.15568429316647 -86.56192182694959 39.15582879947262 -86.56170853704468 39.156019065043765 -86.5614207326925 39.15601872200378 -86.5614121230291 39.15600332864366 -86.56102588974375 39.15597943712648 -86.56042650338233 39.15597340036782 -86.56024789604041 39.155960375125844 -86.55980575464773 39.155958724892585 -86.55974973933058 39.155957189028975 -86.55968924251646 39.15595441068488 -86.55957979005956 39.15594546210676 -86.55922729023088 39.155935915825275 -86.55885129602612 39.155934387774934 -86.55879111375017 39.15593290521979 -86.5587349459513 39.15591300497599 -86.55798120461019 39.155888578382815 -86.55705626623785 39.15588299290239 -86.55684480727955 39.15585589096223 -86.55581896163814 39.15585336577262 -86.55568939302356 39.155851647759484 -86.55560126205981 39.155849360586444 -86.55548392582467 39.15584382729491 -86.55515998408265 39.155912532483434 -86.55516294321605 39.15591116698357 -86.5550830130295 39.15589823801097 -86.55444572472447 39.155898227743116 -86.55444521664668 39.155888040333615 -86.55394318858876 39.15588744004532 -86.55391360939325 39.15587573842033 -86.55333708810046 39.155873792793734 -86.55324125827777 39.15586560428125 -86.55283791953912 39.15586448386191 -86.55278274295101 39.155853914007494 -86.55226222512611 39.15584760495518 -86.55195159741388 39.15584666287038 -86.55190521541343 39.15584845647982 -86.5512751728208 39.15585050513396 -86.55055409568003 39.155767467243535 -86.55050022429107</gml:posList></gml:LinearRing>
                                            </gml:exterior>
                                        </gml:Polygon>
                                    </gml:surfaceMember>
                                </gml:MultiSurface>
                            </publicgis:the_geom>
                            <publicgis:id>12</publicgis:id>
                            <publicgis:zone_id>LMS</publicgis:zone_id>
                            <publicgis:zonedesc>MCSO SOUTH DISTRICT</publicgis:zonedesc>
                            <publicgis:agencyid>MCSO</publicgis:agencyid>
                            <publicgis:agency>MONROE COUNTY SHERIFFS OFFICE</publicgis:agency>
                            <publicgis:distnum>S</publicgis:distnum>
                            <publicgis:distname>South District</publicgis:distname>
                            <publicgis:service>Police</publicgis:service>
                            <publicgis:center_x>3102509</publicgis:center_x>
                            <publicgis:center_y>1422871</publicgis:center_y>
                            <publicgis:label/>
                            <publicgis:fulldesc>Monroe County Sheriffs Office South District</publicgis:fulldesc>
                        </publicgis:PoliceDistricts>
                    </wfs:member>
                </wfs:FeatureCollection>            
            "#))
        },
        Err(err) => Err(HecateError::from_db(err))
    }
}
