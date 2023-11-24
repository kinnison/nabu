// @generated automatically by Diesel CLI.

diesel::table! {
    identity (id) {
        id -> Int4,
        name -> Varchar,
        admin -> Bool,
    }
}

diesel::table! {
    krate (id) {
        id -> Int4,
        name -> Varchar,
        owner -> Int4,
    }
}

diesel::table! {
    kratever (id) {
        id -> Int4,
        krate -> Int4,
        exposed -> Bool,
        ver -> Varchar,
        yanked -> Bool,
        metadata -> Jsonb,
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

diesel::joinable!(krate -> identity (owner));
diesel::joinable!(kratever -> krate (krate));
diesel::joinable!(token -> identity (identity));

diesel::allow_tables_to_appear_in_same_query!(
    identity,
    krate,
    kratever,
    token,
);
