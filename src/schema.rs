// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Uuid,
        first_name -> Varchar,
        last_name -> Varchar,
        #[max_length = 10]
        phone_number -> Bpchar,
        #[max_length = 3]
        country_code -> Bpchar,
        password_hash -> Varchar,
        refresh_token -> Uuid,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
