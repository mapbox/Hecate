CREATE TABLE IF NOT EXISTS users (
    username    TEXT,
    email       TEXT
);

CREATE EXTENSION IF NOT EXISTS postgis;

CREATE TABLE IF NOT EXISTS geo (
    id          BIGSERIAL,
    version     BIGINT,
    geom        GEOMETRY(GEOMETRY, 4326),
    props       JSONB,
    hashes      BIGINT[]
);

CREATE TABLE IF NOT EXISTS deltas (
    hash        BIGSERIAL,
    props       JSONB
);
