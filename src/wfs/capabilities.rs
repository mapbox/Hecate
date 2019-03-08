use err::HecateError;

pub fn capabilities(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, host: &String) -> Result<String, HecateError> {
    //TODO If ANALYZE hasn't been run or there is no geo in the table, this can return null
    //Handle this with 0,0 BBOX
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

            Ok(format!(r#"<?xml version="1.0" encoding="UTF-8"?>
                <wfs:WFS_Capabilities
                    version="2.0.0"
                    xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
                    xmlns="http://www.opengis.net/wfs/2.0"
                    xmlns:wfs="http://www.opengis.net/wfs/2.0"
                    xmlns:ows="http://www.opengis.net/ows/1.1"
                    xmlns:gml="http://www.opengis.net/gml/3.2"
                    xmlns:fes="http://www.opengis.net/fes/2.0"
                    xmlns:xlink="http://www.w3.org/1999/xlink"
                    xmlns:xs="http://www.w3.org/2001/XMLSchema"
                    xmlns:xml="http://www.w3.org/XML/1998/namespace"
                    xmlns:sf="http://openplans.org/featuretype"
                >
                <ows:ServiceIdentification>
                    <ows:Title>Hecate Data</ows:Title>
                    <ows:Abstract>Hecate Data</ows:Abstract>
                    <ows:Keywords>
                        <ows:Keyword>WFS</ows:Keyword>
                        <ows:Keyword>GEOSERVER</ows:Keyword>
                        <ows:Keyword>Hecate</ows:Keyword>
                    </ows:Keywords>
                    <ows:ServiceType>WFS</ows:ServiceType>
                    <ows:ServiceTypeVersion>2.0.0</ows:ServiceTypeVersion>
                    <ows:Fees>NONE</ows:Fees>
                    <ows:AccessConstraints>NONE</ows:AccessConstraints>
                </ows:ServiceIdentification>
                <ows:ServiceProvider>
                    <ows:ProviderName>Hecate Datastore</ows:ProviderName>
                    <ows:ServiceContact>
                        <ows:IndividualName></ows:IndividualName>
                        <ows:PositionName></ows:PositionName>
                        <ows:ContactInfo>
                            <ows:Phone>
                            <ows:Voice></ows:Voice>
                            <ows:Facsimile/>
                            </ows:Phone>
                            <ows:Address>
                                <ows:DeliveryPoint/>
                                <ows:City></ows:City>
                                <ows:AdministrativeArea></ows:AdministrativeArea>
                                <ows:PostalCode></ows:PostalCode>
                                <ows:Country></ows:Country>
                                <ows:ElectronicMailAddress></ows:ElectronicMailAddress>
                            </ows:Address>
                        </ows:ContactInfo>
                    </ows:ServiceContact>
                </ows:ServiceProvider>
                <ows:OperationsMetadata>
                    <ows:Operation name="GetCapabilities">
                        <ows:DCP><ows:HTTP>
                            <ows:Get xlink:href="{host}/api/wfs?"/>
                        </ows:HTTP></ows:DCP>
                        <ows:Parameter name="AcceptVersions">
                            <ows:AllowedValues><ows:Value>2.0.0</ows:Value></ows:AllowedValues>
                        </ows:Parameter>
                    </ows:Operation>
                    <ows:Operation name="DescribeFeatureType">
                        <ows:DCP><ows:HTTP>
                            <ows:Get xlink:href="{host}/api/wfs?"/>
                        </ows:HTTP></ows:DCP>
                        <ows:Parameter name="outputFormat">
                            <ows:AllowedValues>
                                <ows:Value>text/xml; subtype=gml/3.2</ows:Value>
                            </ows:AllowedValues>
                        </ows:Parameter>
                    </ows:Operation>
                    <ows:Operation name="GetFeature">
                        <ows:DCP><ows:HTTP>
                            <ows:Get xlink:href="{host}/api/wfs?"/>
                        </ows:HTTP></ows:DCP>
                        <ows:Parameter name="resultType">
                            <ows:AllowedValues>
                                <ows:Value>results</ows:Value>
                                <ows:Value>hits</ows:Value>
                            </ows:AllowedValues>
                        </ows:Parameter>
                        <ows:Parameter name="outputFormat">
                            <ows:AllowedValues>
                                <ows:Value>GML32</ows:Value>
                                <ows:Value>GML3</ows:Value>
                                <ows:Value>GML2</ows:Value>
                                <ows:Value>GEOJSON</ows:Value>
                            </ows:AllowedValues>
                        </ows:Parameter>
                    </ows:Operation>
                    <ows:Constraint name="ImplementsBasicWFS">
                        <ows:NoValues/>
                        <ows:DefaultValue>TRUE</ows:DefaultValue>
                    </ows:Constraint>
                    <ows:Constraint name="ImplementsTransactionalWFS">
                        <ows:NoValues/>
                        <ows:DefaultValue>FALSE</ows:DefaultValue>
                    </ows:Constraint>
                    <ows:Constraint name="ImplementsLockingWFS">
                        <ows:NoValues/>
                        <ows:DefaultValue>FALSE</ows:DefaultValue>
                    </ows:Constraint>
                    <ows:Constraint name="KVPEncoding">
                        <ows:NoValues/>
                        <ows:DefaultValue>TRUE</ows:DefaultValue>
                    </ows:Constraint>
                    <ows:Constraint name="XMLEncoding">
                        <ows:NoValues/>
                        <ows:DefaultValue>TRUE</ows:DefaultValue>
                    </ows:Constraint>
                    <ows:Constraint name="SOAPEncoding">
                        <ows:NoValues/>
                        <ows:DefaultValue>TRUE</ows:DefaultValue>
                    </ows:Constraint>
                    <ows:Constraint name="ImplementsInheritance">
                        <ows:NoValues/>
                        <ows:DefaultValue>FALSE</ows:DefaultValue>
                    </ows:Constraint>
                    <ows:Constraint name="ImplementsRemoteResolve">
                        <ows:NoValues/>
                        <ows:DefaultValue>FALSE</ows:DefaultValue>
                    </ows:Constraint>
                    <ows:Constraint name="ImplementsResultPaging">
                        <ows:NoValues/>
                        <ows:DefaultValue>TRUE</ows:DefaultValue>
                    </ows:Constraint>
                    <ows:Constraint name="ImplementsStandardJoins">
                        <ows:NoValues/>
                        <ows:DefaultValue>TRUE</ows:DefaultValue>
                    </ows:Constraint>
                    <ows:Constraint name="ImplementsSpatialJoins">
                        <ows:NoValues/>
                        <ows:DefaultValue>FALSE</ows:DefaultValue>
                    </ows:Constraint>
                    <ows:Constraint name="ImplementsTemporalJoins">
                        <ows:NoValues/>
                        <ows:DefaultValue>FALSE</ows:DefaultValue>
                    </ows:Constraint>
                    <ows:Constraint name="ImplementsFeatureVersioning">
                        <ows:NoValues/>
                        <ows:DefaultValue>FALSE</ows:DefaultValue>
                    </ows:Constraint>
                    <ows:Constraint name="ManageStoredQueries">
                        <ows:NoValues/>
                        <ows:DefaultValue>FALSE</ows:DefaultValue>
                    </ows:Constraint>
                    <ows:Constraint name="PagingIsTransactionSafe">
                        <ows:NoValues/>
                        <ows:DefaultValue>FALSE</ows:DefaultValue>
                    </ows:Constraint>
                    <ows:Constraint name="QueryExpressions">
                        <ows:AllowedValues>
                            <ows:Value>wfs:Query</ows:Value>
                            <ows:Value>wfs:StoredQuery</ows:Value>
                        </ows:AllowedValues>
                    </ows:Constraint>
                    </ows:OperationsMetadata>
                    <FeatureTypeList>
                        <FeatureType>
                            <Name>HecatePointData</Name>
                            <Title>Hecate Point Data</Title>
                            <DefaultCRS>urn:ogc:def:crs:EPSG::4326</DefaultCRS>
                            <ows:WGS84BoundingBox>
                                <ows:LowerCorner>{lower}</ows:LowerCorner>
                                <ows:UpperCorner>{upper}</ows:UpperCorner>
                            </ows:WGS84BoundingBox>
                        </FeatureType>
                        <FeatureType>
                            <Name>HecateMultiPointData</Name>
                            <Title>Hecate MultiPoint Data</Title>
                            <DefaultCRS>urn:ogc:def:crs:EPSG::4326</DefaultCRS>
                            <ows:WGS84BoundingBox>
                                <ows:LowerCorner>{lower}</ows:LowerCorner>
                                <ows:UpperCorner>{upper}</ows:UpperCorner>
                            </ows:WGS84BoundingBox>
                        </FeatureType>
                        <FeatureType>
                            <Name>HecateLineStringData</Name>
                            <Title>Hecate LineString Data</Title>
                            <DefaultCRS>urn:ogc:def:crs:EPSG::4326</DefaultCRS>
                            <ows:WGS84BoundingBox>
                                <ows:LowerCorner>{lower}</ows:LowerCorner>
                                <ows:UpperCorner>{upper}</ows:UpperCorner>
                            </ows:WGS84BoundingBox>
                        </FeatureType>
                        <FeatureType>
                            <Name>HecateMultiLineStringData</Name>
                            <Title>Hecate MultiLineString Data</Title>
                            <DefaultCRS>urn:ogc:def:crs:EPSG::4326</DefaultCRS>
                            <ows:WGS84BoundingBox>
                                <ows:LowerCorner>{lower}</ows:LowerCorner>
                                <ows:UpperCorner>{upper}</ows:UpperCorner>
                            </ows:WGS84BoundingBox>
                        </FeatureType>
                        <FeatureType>
                            <Name>HecatePolygonData</Name>
                            <Title>Hecate Polygon Data</Title>
                            <DefaultCRS>urn:ogc:def:crs:EPSG::4326</DefaultCRS>
                            <ows:WGS84BoundingBox>
                                <ows:LowerCorner>{lower}</ows:LowerCorner>
                                <ows:UpperCorner>{upper}</ows:UpperCorner>
                            </ows:WGS84BoundingBox>
                        </FeatureType>
                        <FeatureType>
                            <Name>HecateMultiPolygonData</Name>
                            <Title>Hecate MultiPolygon Data</Title>
                            <DefaultCRS>urn:ogc:def:crs:EPSG::4326</DefaultCRS>
                            <ows:WGS84BoundingBox>
                                <ows:LowerCorner>{lower}</ows:LowerCorner>
                                <ows:UpperCorner>{upper}</ows:UpperCorner>
                            </ows:WGS84BoundingBox>
                        </FeatureType>
                    </FeatureTypeList>
                    <fes:Filter_Capabilities>
                        <fes:Conformance>
                            <fes:Constraint name="ImplementsQuery">
                                <ows:NoValues/>
                                <ows:DefaultValue>TRUE</ows:DefaultValue>
                            </fes:Constraint>
                            <fes:Constraint name="ImplementsAdHocQuery">
                                <ows:NoValues/>
                                <ows:DefaultValue>TRUE</ows:DefaultValue>
                            </fes:Constraint>
                            <fes:Constraint name="ImplementsFunctions">
                                <ows:NoValues/>
                                <ows:DefaultValue>TRUE</ows:DefaultValue>
                            </fes:Constraint>
                            <fes:Constraint name="ImplementsResourceId">
                                <ows:NoValues/>
                                <ows:DefaultValue>TRUE</ows:DefaultValue>
                            </fes:Constraint>
                            <fes:Constraint name="ImplementsMinStandardFilter">
                                <ows:NoValues/>
                                <ows:DefaultValue>TRUE</ows:DefaultValue>
                            </fes:Constraint>
                            <fes:Constraint name="ImplementsStandardFilter">
                                <ows:NoValues/>
                                <ows:DefaultValue>TRUE</ows:DefaultValue>
                            </fes:Constraint>
                            <fes:Constraint name="ImplementsMinSpatialFilter">
                                <ows:NoValues/>
                                <ows:DefaultValue>TRUE</ows:DefaultValue>
                            </fes:Constraint>
                            <fes:Constraint name="ImplementsSpatialFilter">
                                <ows:NoValues/>
                                <ows:DefaultValue>TRUE</ows:DefaultValue>
                            </fes:Constraint>
                            <fes:Constraint name="ImplementsMinTemporalFilter">
                                <ows:NoValues/>
                                <ows:DefaultValue>TRUE</ows:DefaultValue>
                            </fes:Constraint>
                            <fes:Constraint name="ImplementsTemporalFilter">
                                <ows:NoValues/>
                                <ows:DefaultValue>TRUE</ows:DefaultValue>
                            </fes:Constraint>
                            <fes:Constraint name="ImplementsVersionNav">
                                <ows:NoValues/>
                                <ows:DefaultValue>FALSE</ows:DefaultValue>
                            </fes:Constraint>
                            <fes:Constraint name="ImplementsSorting">
                                <ows:NoValues/>
                                <ows:DefaultValue>TRUE</ows:DefaultValue>
                            </fes:Constraint>
                            <fes:Constraint name="ImplementsExtendedOperators">
                                <ows:NoValues/>
                                <ows:DefaultValue>FALSE</ows:DefaultValue>
                            </fes:Constraint>
                            <fes:Constraint name="ImplementsMinimumXPath">
                                <ows:NoValues/>
                                <ows:DefaultValue>TRUE</ows:DefaultValue>
                            </fes:Constraint>
                        </fes:Conformance>
                        <fes:Id_Capabilities>
                            <fes:ResourceIdentifier name="fes:ResourceId"/>
                        </fes:Id_Capabilities>
                        <fes:Scalar_Capabilities>
                            <fes:LogicalOperators/>
                            <fes:ComparisonOperators>
                                <fes:ComparisonOperator name="PropertyIsLessThan"/>
                                <fes:ComparisonOperator name="PropertyIsGreaterThan"/>
                                <fes:ComparisonOperator name="PropertyIsLessThanOrEqualTo"/>
                                <fes:ComparisonOperator name="PropertyIsGreaterThanOrEqualTo"/>
                                <fes:ComparisonOperator name="PropertyIsEqualTo"/>
                                <fes:ComparisonOperator name="PropertyIsNotEqualTo"/>
                                <fes:ComparisonOperator name="PropertyIsLike"/>
                                <fes:ComparisonOperator name="PropertyIsBetween"/>
                                <fes:ComparisonOperator name="PropertyIsNull"/>
                                <fes:ComparisonOperator name="PropertyIsNil"/>
                            </fes:ComparisonOperators>
                        </fes:Scalar_Capabilities>
                        <fes:Spatial_Capabilities>
                            <fes:GeometryOperands>
                                <fes:GeometryOperand name="gml:Envelope"/>
                                <fes:GeometryOperand name="gml:Point"/>
                                <fes:GeometryOperand name="gml:MultiPoint"/>
                                <fes:GeometryOperand name="gml:LineString"/>
                                <fes:GeometryOperand name="gml:MultiLineString"/>
                                <fes:GeometryOperand name="gml:Polygon"/>
                                <fes:GeometryOperand name="gml:MultiPolygon"/>
                                <fes:GeometryOperand name="gml:MultiGeometry"/>
                            </fes:GeometryOperands>
                            <fes:SpatialOperators>
                                <fes:SpatialOperator name="Disjoint"/>
                                <fes:SpatialOperator name="Equals"/>
                                <fes:SpatialOperator name="DWithin"/>
                                <fes:SpatialOperator name="Beyond"/>
                                <fes:SpatialOperator name="Intersects"/>
                                <fes:SpatialOperator name="Touches"/>
                                <fes:SpatialOperator name="Crosses"/>
                                <fes:SpatialOperator name="Within"/>
                                <fes:SpatialOperator name="Contains"/>
                                <fes:SpatialOperator name="Overlaps"/>
                                <fes:SpatialOperator name="BBOX"/>
                            </fes:SpatialOperators>
                        </fes:Spatial_Capabilities>
                    </fes:Filter_Capabilities>
                </wfs:WFS_Capabilities>
            "#,
                lower = lower,
                upper = upper,
                host = host
            ))
        },
        Err(err) => Err(HecateError::from_db(err))
    }
}
