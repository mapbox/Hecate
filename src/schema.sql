table! {
    use diesel_geography::sql_types::*;

    bounds (id) {
        id -> Int8,
        geom -> Nullable<Geometry>,
        name -> Nullable<Text>,
        props -> Nullable<Jsonb>,
    }
}

table! {
    use diesel_geography::sql_types::*;

    deltas (id) {
        id -> Int8,
        created -> Nullable<Timestamp>,
        features -> Nullable<Jsonb>,
        affected -> Nullable<Array<Int8>>,
        props -> Nullable<Jsonb>,
        uid -> Nullable<Int8>,
        finalized -> Nullable<Bool>,
    }
}

table! {
    use diesel_geography::sql_types::*;

    geo (id) {
        id -> Int8,
        key -> Nullable<Text>,
        version -> Nullable<Int8>,
        geom -> Nullable<Geometry>,
        props -> Nullable<Jsonb>,
        deltas -> Nullable<Array<Int8>>,
    }
}

table! {
    use diesel_geography::sql_types::*;

    meta (key) {
        key -> Text,
        value -> Nullable<Jsonb>,
    }
}

table! {
    use diesel_geography::sql_types::*;

    spatial_ref_sys (srid) {
        srid -> Int4,
        auth_name -> Nullable<Varchar>,
        auth_srid -> Nullable<Int4>,
        srtext -> Nullable<Varchar>,
        proj4text -> Nullable<Varchar>,
    }
}

table! {
    use diesel_geography::sql_types::*;

    styles (id) {
        id -> Int8,
        name -> Nullable<Text>,
        style -> Nullable<Jsonb>,
        uid -> Nullable<Int8>,
        public -> Nullable<Bool>,
    }
}

table! {
    use diesel_geography::sql_types::*;

    tiles (reference) {
        created -> Nullable<Timestamp>,
        reference -> Text,
        tile -> Nullable<Bytea>,
    }
}

table! {
    use diesel_geography::sql_types::*;

    users (id) {
        id -> Int8,
        access -> Nullable<Text>,
        username -> Nullable<Text>,
        password -> Nullable<Text>,
        email -> Nullable<Text>,
        meta -> Nullable<Jsonb>,
    }
}

table! {
    use diesel_geography::sql_types::*;

    users_tokens (token) {
        name -> Nullable<Text>,
        uid -> Nullable<Int8>,
        token -> Text,
        expiry -> Nullable<Timestamp>,
    }
}

table! {
    use diesel_geography::sql_types::*;

    webhooks (id) {
        id -> Int8,
        name -> Nullable<Text>,
        actions -> Nullable<Array<Text>>,
        url -> Nullable<Text>,
    }
}

allow_tables_to_appear_in_same_query!(
    bounds,
    deltas,
    geo,
    meta,
    spatial_ref_sys,
    styles,
    tiles,
    users,
    users_tokens,
    webhooks,
);
