use err::HecateError;

pub fn capabilities() -> Result<String, HecateError> {
    String::from(r#"
        <?xml version="1.0" encoding="utf-8" ?>
        <ows:ServiceIdentification>
            <ows:Title>WFS</ows:Title>
            <ows:Abstract></ows:Abstract>
            <ows:Keywords><ows:Keyword></ows:Keyword></ows:Keywords>
            <ows:ServiceType>WFS</ows:ServiceType>
            <ows:ServiceTypeVersion>2.0.0</ows:ServiceTypeVersion>
            <ows:Fees></ows:Fees>
            <ows:AccessConstraints></ows:AccessConstraints>
        </ows:ServiceIdentification>
        <ows:ServiceProvider>
            <ows:ProviderName></ows:ProviderName>
            <ows:ServiceContact>
                <ows:IndividualName></ows:IndividualName>
                <ows:PositionName></ows:PositionName>
                <ows:ContactInfo>
                    <ows:Phone>
                        <ows:Voice></ows:Voice>
                        <ows:Facsimile></ows:Facsimile>
                    </ows:Phone>
                    <ows:Address>
                        <ows:DeliveryPoint></ows:DeliveryPoint>
                        <ows:City></ows:City>
                        <ows:AdministrativeArea></ows:AdministrativeArea>
                        <ows:PostalCode></ows:PostalCode>
                        <ows:Country></ows:Country>
                        <ows:ElectronicMailAddress></ows:ElectronicMailAddress>
                    </ows:Address>
                    <ows:OnlineResource xlink:href=""/>
                    <ows:HoursOfService></ows:HoursOfService>
                    <ows:ContactInstructions></ows:ContactInstructions>
                </ows:ContactInfo>
                <ows:role></ows:role>
            </ows:ServiceContact>
        </ows:ServiceProvider>
        <ows:OperationsMetadata>
            <ows:Operation name="GetCapabilities">
                <ows:DCP>
                    <ows:HTTP>
                        <ows:Get xlink:href="https://services.nationalmap.gov/arcgis/services/WFS/govunits/MapServer/WFSServer?"/>
                        <ows:Post xlink:href="https://services.nationalmap.gov/arcgis/services/WFS/govunits/MapServer/WFSServer"/>
                    </ows:HTTP>
                </ows:DCP>
                <ows:Parameter name="AcceptVersions">
                    <ows:AllowedValues><ows:Value>2.0.0</ows:Value></ows:AllowedValues>
                </ows:Parameter>
            </ows:Operation>
            <ows:Operation name="DescribeFeatureType">
                <ows:DCP>
                    <ows:HTTP>
                        <ows:Get xlink:href="https://services.nationalmap.gov/arcgis/services/WFS/govunits/MapServer/WFSServer?"/>
                        <ows:Post xlink:href="https://services.nationalmap.gov/arcgis/services/WFS/govunits/MapServer/WFSServer"/>
                    </ows:HTTP>
                </ows:DCP>
            <ows:Parameter name="outputFormat">
<ows:AllowedValues>
<ows:Value>text/xml; subtype=gml/3.2</ows:Value>
</ows:AllowedValues>
</ows:Parameter>
</ows:Operation>
<ows:Operation name="GetPropertyValue">
<ows:DCP>
<ows:HTTP>
<ows:Get xlink:href="https://services.nationalmap.gov/arcgis/services/WFS/govunits/MapServer/WFSServer?"/>
<ows:Post xlink:href="https://services.nationalmap.gov/arcgis/services/WFS/govunits/MapServer/WFSServer"/>
</ows:HTTP>
</ows:DCP>
</ows:Operation>
<ows:Operation name="GetFeature">
<ows:DCP>
<ows:HTTP>
<ows:Get xlink:href="https://services.nationalmap.gov/arcgis/services/WFS/govunits/MapServer/WFSServer?"/>
<ows:Post xlink:href="https://services.nationalmap.gov/arcgis/services/WFS/govunits/MapServer/WFSServer"/>
</ows:HTTP>
</ows:DCP>
<ows:Parameter name="resultType">
<ows:AllowedValues>
<ows:Value>results</ows:Value>
<ows:Value>hits</ows:Value>
</ows:AllowedValues>
</ows:Parameter>
<ows:Parameter name="outputFormat">
<ows:AllowedValues>
<ows:Value>GML32</ows:Value>
<ows:Value>GML32+ZIP</ows:Value>
<ows:Value>application/gml+xml; version=3.2</ows:Value>
<ows:Value>GML3</ows:Value>
<ows:Value>GML3+ZIP</ows:Value>
<ows:Value>text/xml; subtype=gml/3.1.1</ows:Value>
<ows:Value>GML2</ows:Value>
<ows:Value>GML2+ZIP</ows:Value>
<ows:Value>text/xml; subtype=gml/2.1.2</ows:Value>
<ows:Value>GEOJSON</ows:Value>
<ows:Value>GEOJSON+ZIP</ows:Value>
<ows:Value>ESRIGEOJSON</ows:Value>
<ows:Value>ESRIGEOJSON+ZIP</ows:Value>
<ows:Value>KML</ows:Value>
<ows:Value>application/vnd.google-earth.kml xml</ows:Value>
<ows:Value>application/vnd.google-earth.kml+xml</ows:Value>
<ows:Value>KMZ</ows:Value>
<ows:Value>application/vnd.google-earth.kmz</ows:Value>
<ows:Value>SHAPE+ZIP</ows:Value>
<ows:Value>CSV</ows:Value>
<ows:Value>CSV+ZIP</ows:Value>
</ows:AllowedValues>
</ows:Parameter>
</ows:Operation>
<ows:Operation name="GetGmlObject">
<ows:DCP>
<ows:HTTP>
<ows:Get xlink:href="https://services.nationalmap.gov/arcgis/services/WFS/govunits/MapServer/WFSServer?"/>
<ows:Post xlink:href="https://services.nationalmap.gov/arcgis/services/WFS/govunits/MapServer/WFSServer"/>
</ows:HTTP>
</ows:DCP>
</ows:Operation>
<ows:Operation name="ListStoredQueries">
<ows:DCP>
<ows:HTTP>
<ows:Get xlink:href="https://services.nationalmap.gov/arcgis/services/WFS/govunits/MapServer/WFSServer?"/>
<ows:Post xlink:href="https://services.nationalmap.gov/arcgis/services/WFS/govunits/MapServer/WFSServer"/>
</ows:HTTP>
</ows:DCP>
</ows:Operation>
<ows:Operation name="DescribeStoredQueries">
<ows:DCP>
<ows:HTTP>
<ows:Get xlink:href="https://services.nationalmap.gov/arcgis/services/WFS/govunits/MapServer/WFSServer?"/>
<ows:Post xlink:href="https://services.nationalmap.gov/arcgis/services/WFS/govunits/MapServer/WFSServer"/>
</ows:HTTP>
</ows:DCP>
</ows:Operation>
<ows:Constraint name="ImplementsBasicWFS"><ows:NoValues/><ows:DefaultValue>TRUE</ows:DefaultValue></ows:Constraint>
<ows:Constraint name="ImplementsTransactionalWFS"><ows:NoValues/><ows:DefaultValue>FALSE</ows:DefaultValue></ows:Constraint>
<ows:Constraint name="ImplementsLockingWFS"><ows:NoValues/><ows:DefaultValue>FALSE</ows:DefaultValue></ows:Constraint>
<ows:Constraint name="KVPEncoding"><ows:NoValues/><ows:DefaultValue>TRUE</ows:DefaultValue></ows:Constraint>
<ows:Constraint name="XMLEncoding"><ows:NoValues/><ows:DefaultValue>TRUE</ows:DefaultValue></ows:Constraint>
<ows:Constraint name="SOAPEncoding"><ows:NoValues/><ows:DefaultValue>FALSE</ows:DefaultValue></ows:Constraint>
<ows:Constraint name="ImplementsInheritance"><ows:NoValues/><ows:DefaultValue>FALSE</ows:DefaultValue></ows:Constraint>
<ows:Constraint name="ImplementsRemoteResolve"><ows:NoValues/><ows:DefaultValue>FALSE</ows:DefaultValue></ows:Constraint>
<ows:Constraint name="ImplementsResultPaging"><ows:NoValues/><ows:DefaultValue>TRUE</ows:DefaultValue></ows:Constraint>
<ows:Constraint name="ImplementsStandardJoins"><ows:NoValues/><ows:DefaultValue>FALSE</ows:DefaultValue></ows:Constraint>
<ows:Constraint name="ImplementsSpatialJoins"><ows:NoValues/><ows:DefaultValue>FALSE</ows:DefaultValue></ows:Constraint>
<ows:Constraint name="ImplementsTemporalJoins"><ows:NoValues/><ows:DefaultValue>FALSE</ows:DefaultValue></ows:Constraint>
<ows:Constraint name="ImplementsFeatureVersioning"><ows:NoValues/><ows:DefaultValue>FALSE</ows:DefaultValue></ows:Constraint>
<ows:Constraint name="ManageStoredQueries"><ows:NoValues/><ows:DefaultValue>FALSE</ows:DefaultValue></ows:Constraint>
</ows:OperationsMetadata>
<wfs:FeatureTypeList>
<wfs:FeatureType>
<wfs:Name>WFS_govunits:Reserve</wfs:Name>
<wfs:Title>Reserve</wfs:Title>
<wfs:DefaultCRS>urn:ogc:def:crs:EPSG::3857</wfs:DefaultCRS>
<ows:WGS84BoundingBox>
<ows:LowerCorner>-179.1500822837491 -14.55975325902829</ows:LowerCorner>
<ows:UpperCorner>179.7751119061824 71.21464378528709</ows:UpperCorner>
</ows:WGS84BoundingBox>
</wfs:FeatureType>
<wfs:FeatureType>
<wfs:Name>WFS_govunits:Native_American_Area</wfs:Name>
<wfs:Title>Native_American_Area</wfs:Title>
<wfs:DefaultCRS>urn:ogc:def:crs:EPSG::3857</wfs:DefaultCRS>
<ows:WGS84BoundingBox>
<ows:LowerCorner>-179.2310859995919 18.91069000006619</ows:LowerCorner>
<ows:UpperCorner>179.8596810003538 71.43978599945648</ows:UpperCorner>
</ows:WGS84BoundingBox>
</wfs:FeatureType>
<wfs:FeatureType>
<wfs:Name>WFS_govunits:County_or_Equivalent</wfs:Name>
<wfs:Title>County_or_Equivalent</wfs:Title>
<wfs:DefaultCRS>urn:ogc:def:crs:EPSG::3857</wfs:DefaultCRS>
<ows:WGS84BoundingBox>
<ows:LowerCorner>-179.2310859995919 -14.60181300034508</ows:LowerCorner>
<ows:UpperCorner>179.8596810003538 71.43978599945648</ows:UpperCorner>
</ows:WGS84BoundingBox>
</wfs:FeatureType>
<wfs:FeatureType>
<wfs:Name>WFS_govunits:Incorporated_Place</wfs:Name>
<wfs:Title>Incorporated_Place</wfs:Title>
<wfs:DefaultCRS>urn:ogc:def:crs:EPSG::3857</wfs:DefaultCRS>
<ows:WGS84BoundingBox>
<ows:LowerCorner>-176.6966920000369 -14.41994999964142</ows:LowerCorner>
<ows:UpperCorner>146.0648190000749 71.3401859997405</ows:UpperCorner>
</ows:WGS84BoundingBox>
</wfs:FeatureType>
<wfs:FeatureType>
<wfs:Name>WFS_govunits:Unincorporated_Place</wfs:Name>
<wfs:Title>Unincorporated_Place</wfs:Title>
<wfs:DefaultCRS>urn:ogc:def:crs:EPSG::3857</wfs:DefaultCRS>
<ows:WGS84BoundingBox>
<ows:LowerCorner>-169.1154979999023 13.25030200041252</ows:LowerCorner>
<ows:UpperCorner>173.4299209996617 70.57081599975668</ows:UpperCorner>
</ows:WGS84BoundingBox>
</wfs:FeatureType>
<wfs:FeatureType>
<wfs:Name>WFS_govunits:Minor_Civil_Division</wfs:Name>
<wfs:Title>Minor_Civil_Division</wfs:Title>
<wfs:DefaultCRS>urn:ogc:def:crs:EPSG::3857</wfs:DefaultCRS>
<ows:WGS84BoundingBox>
<ows:LowerCorner>-179.2310859995919 -14.60181300034508</ows:LowerCorner>
<ows:UpperCorner>179.8596810003538 71.43978599945648</ows:UpperCorner>
</ows:WGS84BoundingBox>
</wfs:FeatureType>
<wfs:FeatureType>
<wfs:Name>WFS_govunits:State_or_Territory_Low-res</wfs:Name>
<wfs:Title>State_or_Territory_Low-res</wfs:Title>
<wfs:DefaultCRS>urn:ogc:def:crs:EPSG::3857</wfs:DefaultCRS>
<ows:WGS84BoundingBox>
<ows:LowerCorner>-168.0000252501159 17.62500268255173</ows:LowerCorner>
<ows:UpperCorner>-64.4999977652957 71.25000003477736</ows:UpperCorner>
</ows:WGS84BoundingBox>
</wfs:FeatureType>
<wfs:FeatureType>
<wfs:Name>WFS_govunits:State_or_Territory_High-res</wfs:Name>
<wfs:Title>State_or_Territory_High-res</wfs:Title>
<wfs:DefaultCRS>urn:ogc:def:crs:EPSG::3857</wfs:DefaultCRS>
<ows:WGS84BoundingBox>
<ows:LowerCorner>-179.2310859995919 -14.60181300034508</ows:LowerCorner>
<ows:UpperCorner>179.8596810003538 71.43978599945648</ows:UpperCorner>
</ows:WGS84BoundingBox>
</wfs:FeatureType>
<wfs:FeatureType>
<wfs:Name>WFS_govunits:Congressional_District</wfs:Name>
<wfs:Title>Congressional_District</wfs:Title>
<wfs:DefaultCRS>urn:ogc:def:crs:EPSG::3857</wfs:DefaultCRS>
<ows:WGS84BoundingBox>
<ows:LowerCorner>-179.2310859995919 -14.60181300034508</ows:LowerCorner>
<ows:UpperCorner>179.8596810003538 71.43978599945648</ows:UpperCorner>
</ows:WGS84BoundingBox>
</wfs:FeatureType>
</wfs:FeatureTypeList>
<fes:Filter_Capabilities>
<fes:Conformance>
<fes:Constraint name="ImplementsQuery"><ows:NoValues/><ows:DefaultValue>TRUE</ows:DefaultValue></fes:Constraint>
<fes:Constraint name="ImplementsAdHocQuery"><ows:NoValues/><ows:DefaultValue>TRUE</ows:DefaultValue></fes:Constraint>
<fes:Constraint name="ImplementsFunctions"><ows:NoValues/><ows:DefaultValue>TRUE</ows:DefaultValue></fes:Constraint>
<fes:Constraint name="ImplementsResourceId"><ows:NoValues/><ows:DefaultValue>TRUE</ows:DefaultValue></fes:Constraint>
<fes:Constraint name="ImplementsMinStandardFilter"><ows:NoValues/><ows:DefaultValue>TRUE</ows:DefaultValue></fes:Constraint>
<fes:Constraint name="ImplementsStandardFilter"><ows:NoValues/><ows:DefaultValue>TRUE</ows:DefaultValue></fes:Constraint>
<fes:Constraint name="ImplementsMinSpatialFilter"><ows:NoValues/><ows:DefaultValue>TRUE</ows:DefaultValue></fes:Constraint>
<fes:Constraint name="ImplementsSpatialFilter"><ows:NoValues/><ows:DefaultValue>TRUE</ows:DefaultValue></fes:Constraint>
<fes:Constraint name="ImplementsMinTemporalFilter"><ows:NoValues/><ows:DefaultValue>FALSE</ows:DefaultValue></fes:Constraint>
<fes:Constraint name="ImplementsTemporalFilter"><ows:NoValues/><ows:DefaultValue>FALSE</ows:DefaultValue></fes:Constraint>
<fes:Constraint name="ImplementsVersionNav"><ows:NoValues/><ows:DefaultValue>FALSE</ows:DefaultValue></fes:Constraint>
<fes:Constraint name="ImplementsSorting"><ows:NoValues/><ows:DefaultValue>TRUE</ows:DefaultValue></fes:Constraint>
<fes:Constraint name="ImplementsExtendedOperators"><ows:NoValues/><ows:DefaultValue>FALSE</ows:DefaultValue></fes:Constraint>
<fes:Constraint name="ImplementsMinimumXPath"><ows:NoValues/><ows:DefaultValue>TRUE</ows:DefaultValue></fes:Constraint>
<fes:Constraint name="ImplementsSchemaElementFunc"><ows:NoValues/><ows:DefaultValue>FALSE</ows:DefaultValue></fes:Constraint>
</fes:Conformance>
<fes:Id_Capabilities>
<fes:ResourceIdentifier name="fes:ResourceId"/>
</fes:Id_Capabilities>
<fes:Scalar_Capabilities>
<fes:LogicalOperators/>
<fes:ComparisonOperators>
<fes:ComparisonOperator name="PropertyIsEqualTo"/>
<fes:ComparisonOperator name="PropertyIsNotEqualTo"/>
<fes:ComparisonOperator name="PropertyIsLessThan"/>
<fes:ComparisonOperator name="PropertyIsGreaterThan"/>
<fes:ComparisonOperator name="PropertyIsLessThanOrEqualTo"/>
<fes:ComparisonOperator name="PropertyIsGreaterThanOrEqualTo"/>
<fes:ComparisonOperator name="PropertyIsLike"/>
<fes:ComparisonOperator name="PropertyIsNull"/>
<fes:ComparisonOperator name="PropertyIsNil"/>
<fes:ComparisonOperator name="PropertyIsBetween"/>
</fes:ComparisonOperators>
</fes:Scalar_Capabilities>
<fes:Spatial_Capabilities>
<fes:GeometryOperands xmlns:gml="http://www.opengis.net/gml" xmlns:gml32="http://www.opengis.net/gml">
<fes:GeometryOperand name="gml:Box"/>
<fes:GeometryOperand name="gml:Envelope"/>
<fes:GeometryOperand name="gml:Point"/>
<fes:GeometryOperand name="gml:LineString"/>
<fes:GeometryOperand name="gml:LinearRing"/>
<fes:GeometryOperand name="gml:Polygon"/>
<fes:GeometryOperand name="gml:MultiPoint"/>
<fes:GeometryOperand name="gml:MultiCurve"/>
<fes:GeometryOperand name="gml:MultiSurface"/>
</fes:GeometryOperands>
<fes:SpatialOperators>
<fes:SpatialOperator name="BBOX"/>
<fes:SpatialOperator name="Equals"/>
<fes:SpatialOperator name="Disjoint"/>
<fes:SpatialOperator name="Intersects"/>
<fes:SpatialOperator name="Touches"/>
<fes:SpatialOperator name="Crosses"/>
<fes:SpatialOperator name="Within"/>
<fes:SpatialOperator name="Contains"/>
<fes:SpatialOperator name="Overlaps"/>
<fes:SpatialOperator name="Beyond"/>
<fes:SpatialOperator name="DWithin"/>
</fes:SpatialOperators>
</fes:Spatial_Capabilities>
</fes:Filter_Capabilities>
</wfs:WFS_Capabilities>

    "#)
}
