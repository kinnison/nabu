// @generated automatically by Diesel CLI.

diesel::table! {
    identity (id) {
        id -> Int4,
        name -> Varchar,
        admin -> Bool,
    }
}

diesel::table! {
    token (id) {
        id -> Int4,
        identity -> Int4,
        title -> Varchar,
        content -> Varchar,
    }
}

diesel::joinable!(token -> identity (identity));

diesel::allow_tables_to_appear_in_same_query!(
    identity,
    token,
);
