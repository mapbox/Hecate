CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS hstore;

DROP TABLE IF EXISTS users;
CREATE TABLE users (
    id          BIGSERIAL,
    username    TEXT,
    password    TEXT,
    email       TEXT,
    meta        JSONB
);

DROP TABLE IF EXISTS geo;
CREATE TABLE geo (
    id          BIGSERIAL,
    version     BIGINT,
    geom        GEOMETRY(GEOMETRY, 4326),
    props       JSONB,
    deltas      BIGINT[]
);

DROP TABLE IF EXISTS deltas;
CREATE TABLE deltas (
    id          BIGSERIAL,
    created     TIMESTAMP,
    features    JSONB,
    props       JSONB,
    uid         BIGINT
);
