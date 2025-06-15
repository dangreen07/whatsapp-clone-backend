use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;
use serde::{Deserialize, Serialize};
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
   pub sub: String,
   pub exp: i64
}

pub mod schema;
pub mod models;
pub mod auth;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn get_connection_pool() -> Pool<ConnectionManager<PgConnection>> {
    dotenv().ok();
    let url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(url);
    // Refer to the `r2d2` documentation for more methods to use
    // when building a connection pool
    Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool")
}