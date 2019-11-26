-- creates, populates, and creates indexes on the geo_history table using the delta.features JSON
DROP TABLE IF EXISTS geo_history;
CREATE TABLE geo_history (
    id          BIGINT NOT NULL,
    delta       BIGINT NOT NULL,
    key         TEXT,
    action      TEXT NOT NULL,
    version     BIGINT NOT NULL,
    geom        GEOMETRY(GEOMETRY, 4326),
    props       JSONB
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

-- Modifies the action of features that were erroneously created twice -- affects 26 features
UPDATE geo_history
SET
    action = 'modify'
WHERE
    delta = 7729
    AND action = 'create'
    AND id IN (27810597, 27805167, 27816051, 27924080, 27927837, 27923999, 27932150, 27930883, 27932151, 27926520, 27930630, 27931128, 27919336, 27927921, 27921858, 27931856, 27918601, 27925779, 27930666, 27934313, 27934226, 27934331, 27666332, 27666029, 27671491, 27671127);

-- Modifies the version of features that did not have their versions incremented -- affects 26 features
UPDATE geo_history
SET
    version = 3
WHERE
    delta = 7730
    AND version = 2
    AND id IN (27810597, 27805167, 27816051, 27924080, 27927837, 27923999, 27932150, 27930883, 27932151, 27926520, 27930630, 27931128, 27919336, 27927921, 27921858, 27931856, 27918601, 27925779, 27930666, 27934313, 27934226, 27934331, 27666332, 27666029, 27671491, 27671127);

-- Once modification is complete, add primary key constraint
ALTER TABLE geo_history ADD PRIMARY KEY (id, version);

CREATE INDEX geo_history_gist ON geo_history USING GIST(geom);
CREATE INDEX geo_history_idx ON geo_history(id);
CREATE INDEX geo_history_deltax ON geo_history(delta);
