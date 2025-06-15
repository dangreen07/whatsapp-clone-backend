// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Uuid,
        first_name -> Varchar,
        last_name -> Varchar,
        phone_number -> Varchar,
        password_hash -> Varchar,
        refresh_token -> Uuid,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
