use err::HecateError;

pub fn describe_feature_type(conn: &r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>) -> Result<String, HecateError> {
    match conn.query("
        SELECT 1;
    ", &[]) {
        Ok(res) => {
            let default_type = String::from(r#"
                <xsd:element maxOccurs="1" minOccurs="0" name="hecate:version" nillable="false" type="xsd:integer"/>
            "#);

            Ok(format!(r#"
                <xsd:schema xmlns:gml="http://www.opengis.net/gml/3.2" xmlns:publicgis="http://bloomington.in.gov/publicgis" xmlns:wfs="http://www.opengis.net/wfs/2.0" xmlns:xsd="http://www.w3.org/2001/XMLSchema" elementFormDefault="qualified" targetNamespace="http://bloomington.in.gov/publicgis">
                    <xsd:import namespace="http://www.opengis.net/gml/3.2" schemaLocation="https://tarantula.bloomington.in.gov:443/geoserver/schemas/gml/3.2.1/gml.xsd"/>
                    <xsd:complexType name="HecatePointDataType">
                        <xsd:complexContent>
                            <xsd:extension base="gml:AbstractFeatureType">
                                <xsd:sequence>
                                    {default_type}
                                    <xsd:element ref="gml:pointProperty"/>
                                </xsd:sequence>
                            </xsd:extension>
                        </xsd:complexContent>
                    </xsd:complexType>
                    <xsd:element name="HecatePointData" substitutionGroup="gml:AbstractFeature" type="HecatePointDataType"/>

                    <xsd:complexType name="HecateMultiPointDataType">
                        <xsd:complexContent>
                            <xsd:extension base="gml:AbstractFeatureType">
                                <xsd:sequence>
                                    {default_type}
                                    <xsd:element name="geom" type="gml:MultiPoint"/>
                                </xsd:sequence>
                            </xsd:extension>
                        </xsd:complexContent>
                    </xsd:complexType>
                    <xsd:element name="HecateMultiPointData" substitutionGroup="gml:AbstractFeature" type="HecateMultiPointDataType"/>

                    <xsd:complexType name="HecateLineStringDataType">
                        <xsd:complexContent>
                            <xsd:extension base="gml:AbstractFeatureType">
                                <xsd:sequence>
                                    {default_type}
                                    <xsd:element name="geom" type="gml:LineString"/>
                                </xsd:sequence>
                            </xsd:extension>
                        </xsd:complexContent>
                    </xsd:complexType>
                    <xsd:element name="HecateLineStringData" substitutionGroup="gml:AbstractFeature" type="HecateLineStringDataType"/>

                    <xsd:complexType name="HecateMultiLineStringDataType">
                        <xsd:complexContent>
                            <xsd:extension base="gml:AbstractFeatureType">
                                <xsd:sequence>
                                    {default_type}
                                    <xsd:element name="geom" type="gml:MultiLineString"/>
                                </xsd:sequence>
                            </xsd:extension>
                        </xsd:complexContent>
                    </xsd:complexType>
                    <xsd:element name="HecateMultiLineStringData" substitutionGroup="gml:AbstractFeature" type="HecateMultiLineStringDataType"/>

                    <xsd:complexType name="HecatePolygonDataType">
                        <xsd:complexContent>
                            <xsd:extension base="gml:AbstractFeatureType">
                                <xsd:sequence>
                                    {default_type}
                                    <xsd:element name="geom" type="gml:LineString"/>
                                </xsd:sequence>
                            </xsd:extension>
                        </xsd:complexContent>
                    </xsd:complexType>
                    <xsd:element name="HecatePolygonData" substitutionGroup="gml:AbstractFeature" type="HecatePolygonDataType"/>

                    <xsd:complexType name="HecateMultiPolygonDataType">
                        <xsd:complexContent>
                            <xsd:extension base="gml:AbstractFeatureType">
                                <xsd:sequence>
                                    {default_type}
                                    <xsd:element name="geom" type="gml:MultiLineString"/>
                                </xsd:sequence>
                            </xsd:extension>
                        </xsd:complexContent>
                    </xsd:complexType>
                    <xsd:element name="HecateMultiPolygonData" substitutionGroup="gml:AbstractFeature" type="HecateMultiPolygonDataType"/>
                </xsd:schema>
            "#,
                default_type = default_type
            ))
        },
        Err(err) => Err(HecateError::from_db(err))
    }
}
