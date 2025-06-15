use actix_web::{http::Error, post, web, App, HttpResponse, HttpServer};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    }, Argon2, PasswordHash, PasswordVerifier
};
use diesel::PgConnection;
use serde::Deserialize;
use serde_json::json;
use whatsapp_clone_backend::{auth::{generate_jwt, refresh_jwt, JWTResponse}, get_connection_pool, models::{NewUser, User}, schema::users};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;

type DbPool = Pool<ConnectionManager<PgConnection>>;

struct AppState {
    db: DbPool
}

#[derive(Deserialize)]
struct SignUpRequest {
    first_name: String,
    last_name: String,
    phone_number: String,
    country_code: String,
    password: String,
}

#[derive(Deserialize)]
struct SignInRequest {
    phone_number: String,
    country_code: String,
    password: String
}

#[derive(Deserialize)]
struct RefreshTokenRequest {
    refresh_token: String
}

#[post("/sign-up")]
async fn sign_up(data: web::Json<SignUpRequest>, state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(data.password.as_bytes(), &salt).unwrap().to_string();

    let new_user = NewUser {
        first_name: data.first_name.clone(),
        last_name: data.last_name.clone(),
        phone_number: data.phone_number.clone(),
        country_code: data.country_code.clone(),
        password_hash: password_hash
    };

    let mut conn = state.db.get().unwrap();

    let user = match diesel::insert_into(users::table)
        .values(&new_user)
        .returning(User::as_returning())
        .get_result(&mut *conn) {
            Ok(user) => user,
            Err(_) => return Ok(HttpResponse::BadRequest().body("Failed to save new user"))
        };

    let access_token = generate_jwt(&user);

    let response = JWTResponse {
        access_token,
        refresh_token: user.refresh_token.to_string()
    };

    Ok(HttpResponse::Ok().json(json!(response)))
}

#[post("/sign-in")]
async fn sign_in(data: web::Json<SignInRequest>, state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    use whatsapp_clone_backend::schema::users::dsl::*;

    let mut conn = state.db.get().unwrap();

    let user = users.filter(phone_number.eq(data.phone_number.clone()).and(country_code.eq(data.country_code.clone())))
        .first::<User>(&mut *conn);

    let user = match user {
        Ok(user) => user,
        Err(_) => return Ok(HttpResponse::BadRequest().body("Invalid phone number or password"))
    };

    let parsed_hash = PasswordHash::new(&user.password_hash).unwrap();

    let is_valid = Argon2::default().verify_password(data.password.as_bytes(), &parsed_hash).is_ok();

    if !is_valid {
        return Ok(HttpResponse::BadRequest().body("Invalid phone number or password"))
    }

    let access_token = generate_jwt(&user);

    let response = JWTResponse {
        access_token,
        refresh_token: user.refresh_token.to_string()
    };

    Ok(HttpResponse::Ok().json(json!(response)))
}

#[post("/refresh-token")]
async fn refresh_token(data: web::Json<RefreshTokenRequest>, state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let mut conn = state.db.get().unwrap();

    let response = match refresh_jwt(&mut conn, data.refresh_token.clone()) {
        Ok(response) => response,
        Err(_) => return Ok(HttpResponse::BadRequest().body("Invalid refresh token"))
    };

    Ok(HttpResponse::Ok().json(json!(response)))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = get_connection_pool();

    let state = web::Data::new(AppState {
        db: pool
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(sign_up)
            .service(sign_in)
            .service(refresh_token)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}