BEGIN;

CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS hstore;

DROP TABLE IF EXISTS meta;
CREATE TABLE meta (
    key         TEXT UNIQUE,
    value       JSONB
);

DROP TABLE IF EXISTS tiles;
CREATE TABLE tiles (
    created     TIMESTAMP,
    ref         TEXT UNIQUE,
    tile        BYTEA
);

DROP TABLE IF EXISTS bounds;
CREATE TABLE bounds (
    id          BIGSERIAL,
    geom        GEOMETRY(MULTIPOLYGON, 4326),
    name        TEXT UNIQUE,
    props       JSONB
);
CREATE INDEX bounds_gist ON bounds USING GIST(geom);
CREATE INDEX bounds_idx ON bounds(name);

DROP TABLE IF EXISTS users;
CREATE TABLE users (
    id          BIGSERIAL,
    access      TEXT,
    username    TEXT UNIQUE,
    password    TEXT,
    email       TEXT UNIQUE,
    meta        JSONB
);

DROP TABLE IF EXISTS users_tokens;
CREATE TABLE users_tokens (
    name        TEXT,
    uid         BIGINT,
    token       TEXT,
    expiry      TIMESTAMP
);

DROP TABLE IF EXISTS geo;
DROP INDEX IF EXISTS geo_gist;
DROP INDEX IF EXISTS geo_idx;
CREATE TABLE geo (
    id          BIGSERIAL UNIQUE,
    key         TEXT UNIQUE,
    version     BIGINT,
    geom        GEOMETRY(GEOMETRY, 4326),
    props       JSONB,
    deltas      BIGINT[]
);
CREATE INDEX geo_gist ON geo USING GIST(geom);
CREATE INDEX geo_idx ON geo(id);

DROP TABLE IF EXISTS styles;
CREATE TABLE styles (
    id          BIGSERIAL,
    name        TEXT,
    style       JSONB,
    uid         BIGINT,
    public      BOOLEAN
);

DROP TABLE IF EXISTS deltas;
CREATE TABLE deltas (
    id          BIGSERIAL,
    created     TIMESTAMP,
    features    JSONB,
    affected    BIGINT[],
    props       JSONB,
    uid         BIGINT,
    finalized   BOOLEAN DEFAULT FALSE
);
CREATE INDEX deltas_idx ON deltas(id);
CREATE INDEX deltas_affeted_idx on deltas USING GIN (affected);

-- delete_geo( id, version )
CREATE OR REPLACE FUNCTION delete_geo(BIGINT, BIGINT)
    RETURNS boolean AS $$
    BEGIN
        DELETE FROM geo
            WHERE
                id = $1
                AND version = $2;

        IF NOT FOUND THEN
            RAISE EXCEPTION 'DELETE: ID or VERSION Mismatch';
        END IF;

        RETURN true;
    END;
    $$ LANGUAGE plpgsql;

-- modify_geo( geom_str, props_str, delta, id, version, key )
CREATE OR REPLACE FUNCTION modify_geo(TEXT, TEXT, BIGINT, BIGINT, BIGINT, TEXT)
    RETURNS boolean AS $$
    BEGIN
        UPDATE geo
            SET
                version = version + 1,
                geom = ST_SetSRID(ST_GeomFromGeoJSON($1), 4326),
                props = $2::TEXT::JSON,
                deltas = array_append(deltas, $3::BIGINT),
                key = $6
            WHERE
                id = $4
                AND version = $5;

        IF NOT FOUND THEN
            RAISE EXCEPTION 'MODIFY: ID or VERSION Mismatch';
        END IF;

        RETURN true;
    END;
    $$ LANGUAGE plpgsql;

COMMIT;

-- ------- Hecate Read User ----------
DO $$DECLARE count int;
    BEGIN
        SELECT count(*) INTO count FROM pg_roles WHERE rolname = 'hecate_read';

        IF count > 0 THEN
            EXECUTE 'REVOKE ALL PRIVILEGES ON "geo" FROM hecate_read;';
            EXECUTE 'REVOKE ALL PRIVILEGES ON "deltas" FROM hecate_read;';
            EXECUTE 'DROP ROLE IF EXISTS hecate_read;';
        END IF;
    END$$;

CREATE ROLE hecate_read WITH LOGIN NOINHERIT;
GRANT SELECT ON geo TO hecate_read;
GRANT SELECT ON deltas TO hecate_read;
-- -----------------------------------

-- ------- Hecate User ---------------
DO $$DECLARE
    owner TEXT;
    count INT;
    BEGIN
        SELECT count(*) INTO count FROM pg_roles WHERE rolname = 'hecate';

        IF count = 0 THEN
            EXECUTE 'CREATE ROLE hecate WITH LOGIN NOINHERIT;';
        END IF;

        EXECUTE 'GRANT ALL PRIVILEGES ON DATABASE hecate TO hecate;';
        EXECUTE 'GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO hecate;';
        EXECUTE 'GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO hecate;';

        SELECT u.usename INTO owner
            FROM pg_database d JOIN pg_user u ON (d.datdba = u.usesysid)
            WHERE d.datname = (SELECT current_database());

        IF owner != 'hecate' THEN
            EXECUTE 'ALTER DATABASE hecate OWNER TO "hecate";';
        END IF;
    END$$;
-- -----------------------------------
