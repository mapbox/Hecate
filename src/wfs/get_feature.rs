use crate::wfs::Query;
use crate::wfs::ResultType;

use crate::stream::PGStream;
use crate::err::HecateError;

pub fn get_feature(conn: r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, query: &Query) -> Result<PGStream, HecateError> {

    if query.srsname.is_some() && query.srsname != Some(String::from("urn:ogc:def:crs:EPSG::4326")) {
        let mut err = HecateError::new(400, String::from("Only srsname=urn:ogc:def:crs:EPSG::4326 supported"), None);
        err.to_wfsxml();
        return Err(err);
    }

    let geom_filter = match query.typenames {
        None => {
            let mut err = HecateError::new(400, String::from("typenames param required"), None);
            err.to_wfsxml();
            return Err(err);
        },
        Some(ref typenames) => {
            if &*typenames == "HecatePointData" {
                String::from("GeometryType(geom) = 'POINT'")
            } else if &*typenames == "HecateMultiPointData" {
                String::from("GeometryType(geom) = 'MULTIPOINT'")
            } else if &*typenames == "HecateLineStringData" {
                String::from("GeometryType(geom) = 'LINESTRING'")
            } else if &*typenames == "HecateMultiLineStringData" {
                String::from("GeometryType(geom) = 'MULTILINESTRING'")
            } else if &*typenames == "HecatePolygonData" {
                String::from("GeometryType(geom) = 'POLYGON'")
            } else if &*typenames == "HecateMultiPolygonData" {
                String::from("GeometryType(geom) = 'MULTIPOLYGON'")
            } else {
                let mut err = HecateError::new(400, String::from("Unknown typenames layer"), None);
                err.to_wfsxml();
                return Err(err);
            }
        }
    };

    //TODO support custom limits
    let limit = 1000;

    match conn.query(format!("
        SELECT
            ST_AsGML(3, ST_Extent(d.geom), 5, 32) AS extent
        FROM (
            SELECT geom
            FROM geo
            WHERE
                {geom_filter}
        ) d;
    ",
        geom_filter = geom_filter
    ).as_str(), &[ ]) {
        Ok(res) => {
            let gmlenvelope: String = res.get(0).get(0);

            let pre = Some(format!(r#"
                <wfs:FeatureCollection xmlns:{typename}="mapbox.com" xmlns:wfs="http://www.opengis.net/wfs/2.0" xmlns:gml="http://www.opengis.net/gml/3.2" numberReturned="1">
                    <wfs:boundedBy>{gmlenvelope}</wfs:boundedBy>
            "#,
                gmlenvelope = gmlenvelope,
                typename = query.typenames.as_ref().unwrap()
            ).into_bytes());

            let post = Some(String::from("</wfs:FeatureCollection>").into_bytes());

            //TODO handle resulttype = hits

            Ok(PGStream::new(conn, String::from("next_wfsfeature"), format!(r#"
                DECLARE next_wfsfeature CURSOR FOR
                    SELECT
                        xmlelement(name "wfs:member", (
                            xmlelement(name "{typename}", xmlattributes(CONCAT('{typename}.', id) AS "gml:id"),
                                xmlconcat(
                                    xmlelement(name "gml:boundedBy", ST_AsGML(3, geom, 5, 32)::XML),
                                    xmlelement(name "hecate_key", key),
                                    xmlelement(name "hecate_version", version),
                                    xmlelement(name "props", props::TEXT),
                                    xmlelement(name "hecate_geom", ST_AsGML(3, ST_FlipCoordinates(geom), 5, 21)::XML),
                                    xmlconcat((
                                        SELECT
                                            xmlagg(format('<%1$s>%2$s</%1$s>', d.key, d.value)::XML)
                                        FROM
                                            jsonb_each_text(geo.props) AS d
                                    ))
                                )
                            )
                        )::XML)::TEXT
                    FROM
                        geo
                    WHERE
                        {geom_filter}
            "#,
                geom_filter = geom_filter,
                typename = query.typenames.as_ref().unwrap()
            ), &[], pre, post)?)
        },
        Err(err) => Err(HecateError::from_db(err))
    }
}
