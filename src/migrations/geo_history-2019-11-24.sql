-- populates the geo_history table using the delta.features JSON

INSERT INTO geo_history (id, delta, version, action, props, geom)
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
