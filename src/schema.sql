CREATE TABLE IF NOT EXISTS users (
    id          BIGSERIAL,
    username    TEXT,
    password    TEXT,
    email       TEXT,
    meta        JSON
);

CREATE EXTENSION IF NOT EXISTS postgis;

CREATE TABLE IF NOT EXISTS geo (
    id          BIGSERIAL,
    version     BIGINT,
    geom        GEOMETRY(GEOMETRY, 4326),
    props       JSONB,
    deltas      BIGINT[]
);

CREATE TABLE IF NOT EXISTS deltas (
    id          BIGSERIAL,
    created     TIMESTAMP,
    features    JSONB,
    uid         BIGINT
);
