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

            Ok(format!(r#"
                <WFS_Capabilities version="2.0.0" schemaLocation="http://www.opengis.net/wfs/2.0" updateSequence="4682">
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
                                    <Value>GML3</Value>
                                    <Value>GML2</Value>
                                    <Value>GEOJSON</Value>
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
                            <Name>HecatePointData</Name>
                            <Title>Hecate Point Data</Title>
                            <DefaultCRS>urn:ogc:def:crs:EPSG::4326</DefaultCRS>
                            <WGS84BoundingBox>
                                <LowerCorner>{lower}</LowerCorner>
                                <UpperCorner>{upper}</UpperCorner>
                            </WGS84BoundingBox>
                        </FeatureType>
                        <FeatureType>
                            <Name>HecateMultiPointData</Name>
                            <Title>Hecate MultiPoint Data</Title>
                            <DefaultCRS>urn:ogc:def:crs:EPSG::4326</DefaultCRS>
                            <WGS84BoundingBox>
                                <LowerCorner>{lower}</LowerCorner>
                                <UpperCorner>{upper}</UpperCorner>
                            </WGS84BoundingBox>
                        </FeatureType>
                        <FeatureType>
                            <Name>HecateLineStringData</Name>
                            <Title>Hecate LineString Data</Title>
                            <DefaultCRS>urn:ogc:def:crs:EPSG::4326</DefaultCRS>
                            <WGS84BoundingBox>
                                <LowerCorner>{lower}</LowerCorner>
                                <UpperCorner>{upper}</UpperCorner>
                            </WGS84BoundingBox>
                        </FeatureType>
                        <FeatureType>
                            <Name>HecateMultiLineStringData</Name>
                            <Title>Hecate MultiLineString Data</Title>
                            <DefaultCRS>urn:ogc:def:crs:EPSG::4326</DefaultCRS>
                            <WGS84BoundingBox>
                                <LowerCorner>{lower}</LowerCorner>
                                <UpperCorner>{upper}</UpperCorner>
                            </WGS84BoundingBox>
                        </FeatureType>
                        <FeatureType>
                            <Name>HecatePolygonData</Name>
                            <Title>Hecate Polygon Data</Title>
                            <DefaultCRS>urn:ogc:def:crs:EPSG::4326</DefaultCRS>
                            <WGS84BoundingBox>
                                <LowerCorner>{lower}</LowerCorner>
                                <UpperCorner>{upper}</UpperCorner>
                            </WGS84BoundingBox>
                        </FeatureType>
                        <FeatureType>
                            <Name>HecateMultiPolygonData</Name>
                            <Title>Hecate MultiPolygon Data</Title>
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
