table! {
    use diesel_geography::sql_types::*;

    bounds (id) {
        id -> Int8,
        geom -> Geometry,
        name -> Text,
        props -> Jsonb,
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
        version -> Int8,
        geom -> Geometry,
        props -> Jsonb,
        deltas -> Nullable<Array<Int8>>,
    }
}

table! {
    use diesel_geography::sql_types::*;

    meta (key) {
        key -> Text,
        value -> Jsonb,
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
        name -> Text,
        style -> Jsonb,
        uid -> Int8,
        public -> Bool,
    }
}

table! {
    use diesel_geography::sql_types::*;

    tiles (ref_) {
        created -> Timestamp,
        #[sql_name = "ref"]
        ref_ -> Text,
        tile -> Bytea,
    }
}

table! {
    use diesel_geography::sql_types::*;

    users (id) {
        id -> Int8,
        access -> Nullable<Text>,
        username -> Text,
        password -> Text,
        email -> Text,
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
        name -> Text,
        actions -> Nullable<Array<Text>>,
        url -> Text,
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
