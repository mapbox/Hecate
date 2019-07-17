use crate::err::HecateError;
use crate::stream::PGStream;

pub fn set(conn: &impl postgres::GenericConnection, name: &String, feat: &serde_json::Value) -> Result<bool, HecateError> {
    match conn.execute("
        INSERT INTO bounds (name, geom) VALUES ($1 , ST_Multi(ST_SetSRID(ST_GeomFromGeoJSON($2::JSON->>'geometry'), 4326)))
            ON CONFLICT (name) DO
                UPDATE
                    SET geom = ST_Multi(ST_SetSRID(ST_GeomFromGeoJSON($2::JSON->>'geometry'), 4326))
                    WHERE bounds.name = $1;
    ", &[ &name, &feat ]) {
        Ok(_) => Ok(true),
        Err(err) => Err(HecateError::from_db(err))
    }
}

pub fn delete(conn: &impl postgres::GenericConnection, name: &String) -> Result<bool, HecateError> {
    match conn.execute("
        DELETE FROM bounds WHERE name = $1
    ", &[ &name ]) {
        Ok(_) => Ok(true),
        Err(err) => Err(HecateError::from_db(err))
    }
}

pub fn filter(conn: &impl postgres::GenericConnection, prefix: &String, limit: &Option<i16>) -> Result<Vec<String>, HecateError> {
    let limit: i16 = match limit {
        None => 100,
        Some(limit) => if *limit > 100 { 100 } else { *limit }
    };

    match conn.query("
        SELECT name
            FROM bounds
            WHERE name iLIKE $1||'%'
            ORDER BY name
            LIMIT $2::SmallInt
    ", &[ &prefix, &limit ]) {
        Ok(rows) => {
            let mut names = Vec::<String>::new();

            for row in rows.iter() {
                names.push(row.get(0));
            }

            Ok(names)
        },
        Err(err) => Err(HecateError::from_db(err))
    }
}

pub fn list(conn: &impl postgres::GenericConnection, limit: &Option<i16>) -> Result<Vec<String>, HecateError> {
    match conn.query("
        SELECT name
        FROM bounds
        ORDER BY name
        LIMIT $1::SmallInt
    ", &[ &limit ]) {
        Ok(rows) => {
            let mut names = Vec::<String>::new();

            for row in rows.iter() {
                names.push(row.get(0));
            }

            Ok(names)
        },
        Err(err) => Err(HecateError::from_db(err))
    }
}

pub fn get(conn: r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>, bounds: String) -> Result<PGStream, HecateError> {
    match PGStream::new(conn, String::from("next_bounds"), String::from(r#"
        DECLARE next_bounds CURSOR FOR
            SELECT
                row_to_json(t)::TEXT
            FROM (
                SELECT
                    geo.id AS id,
                    geo.key AS key,
                    'Feature' AS type,
                    geo.version AS version,
                    ST_AsGeoJSON(geo.geom)::JSON AS geometry,
                    geo.props AS properties
                FROM
                    geo,
                    (
                        SELECT
                            ST_Subdivide(bounds.geom) as subgeom
                        FROM
                            bounds
                        WHERE
                            name = $1
                    ) as b
                WHERE
                    ST_Intersects(geo.geom, b.subgeom)
            ) t
    "#), &[&bounds], None, Some(vec![0x04])) {
        Ok(stream) => Ok(stream),
        Err(err) => Err(err)
    }
}

pub fn meta(conn: &impl postgres::GenericConnection, name: String) -> Result<serde_json::Value, HecateError> {
    match conn.query("
        SELECT
            JSON_Build_Object(
                'id', bounds.id,
                'type', 'Feature',
                'properties', COALESCE(props, '{}'::JSONB),
                'geometry', ST_AsGeoJSON(bounds.geom)::JSON
            )::JSON
            FROM
                bounds
            WHERE
                name = $1
    ", &[ &name ]) {
        Ok(rows) => {
            if rows.len() != 1 {
                return Err(HecateError::new(404, String::from("bound not found"), None));
            }

            let bound: serde_json::Value = rows.get(0).get(0);

            Ok(bound)
        },
        Err(err) => Err(HecateError::from_db(err))
    }
}

pub fn stats_json(conn: &impl postgres::GenericConnection, bounds: String) -> Result<serde_json::Value, HecateError> {
    match conn.query("
        SELECT
            row_to_json(t)
        FROM (
            SELECT
                count(*) AS total,
                json_build_array(
                    ST_XMin(ST_Extent(bounds.geom)),
                    ST_YMin(ST_Extent(bounds.geom)),
                    ST_XMax(ST_Extent(bounds.geom)),
                    ST_YMax(ST_Extent(bounds.geom))
                ) AS bbox,
                to_char(now(), 'YYYY-MM-DD HH24:MI:SS') AS last_calc
            FROM
                geo,
                (
                    SELECT
                        ST_Subdivide(bounds.geom) as subgeom
                    FROM
                        bounds
                    WHERE
                        name = $1
                ) as b,
                bounds
            WHERE
                ST_Intersects(geo.geom, b.subgeom)
                AND bounds.name = $1
        ) t
    ", &[ &bounds ]) {
        Ok(rows) => Ok(rows.get(0).get(0)),
        Err(err) => Err(HecateError::from_db(err))
    }
}
