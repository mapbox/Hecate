-- creates, populates, and creates indexes on the geo_history table using the delta.features JSON
DROP TABLE IF EXISTS geo_history;
CREATE TABLE geo_history (
    id          BIGINT NOT NULL,
    delta       BIGINT NOT NULL,
    key         TEXT,
    action      TEXT NOT NULL,
    version     BIGINT NOT NULL,
    geom        GEOMETRY(GEOMETRY, 4326),
    props       JSONB,
    PRIMARY KEY (id, version)
);

INSERT INTO geo_history (id, delta, version, action, props, key, geom)
SELECT
    (feat->>'id')::BIGINT AS id,
    id AS delta,
    -- the version stored in the deltas.features blob is one behind
    -- if it doesn't exist it should be 1, otherwise increment by 1
    CASE
        WHEN feat->>'version' IS NULL THEN 1
        ELSE (feat->>'version')::BIGINT + 1
    END AS version,
    feat->>'action' AS action,
    feat->'properties' AS props,
    feat->>'key' AS key,
    ST_SetSRID(
        ST_MakePoint(
            (feat->'geometry'->'coordinates'->>0)::FLOAT,
            (feat->'geometry'->'coordinates'->>1)::FLOAT
        ),
        4326
    ) AS geom
FROM (
    SELECT
        id,
        JSON_Array_Elements((features -> 'features')::JSON) AS feat
    FROM
        deltas
) t;

CREATE INDEX geo_history_gist ON geo_history USING GIST(geom);
CREATE INDEX geo_history_idx ON geo_history(id);
CREATE INDEX geo_history_deltax ON geo_history(delta);
