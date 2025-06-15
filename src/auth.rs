use chrono::Duration;
use chrono::Utc;
use diesel::PgConnection;
use diesel::prelude::*;
use dotenvy::dotenv;
use jsonwebtoken::encode;
use jsonwebtoken::EncodingKey;
use jsonwebtoken::Header;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::User;
use crate::Claims;

#[derive(Debug, Serialize, Deserialize)]
pub struct JWTResponse {
    pub access_token: String,
    pub refresh_token: String
}

pub fn refresh_jwt(conn: &mut PgConnection, refresh_token_var: String) -> Result<JWTResponse, &'static str> {
    use crate::schema::users::dsl::*;
    let refresh_token_var = match Uuid::parse_str(refresh_token_var.as_str()) {
        Ok(token) => token,
        Err(_) => return Err("Invalid refresh token")
    };

    let user = users
            .filter(refresh_token.eq(refresh_token_var)).first::<User>(conn);
    
    let user = match user {
        Ok(user) => user,
        Err(_) => return Err("Invalid refresh token")
    };

    let access_token = generate_jwt(&user);

    // Invalidate the refresh token and make a new one
    let new_refresh_token = Uuid::new_v4();
    diesel::update(users.filter(id.eq(user.id)))
        .set(refresh_token.eq(new_refresh_token))
        .execute(conn)
        .map_err(|_| "Failed to update refresh token")?;

    Ok(JWTResponse {
        access_token,
        refresh_token: new_refresh_token.to_string()
    })
}

pub fn generate_jwt(user: &User) -> String {
    dotenv().ok();
    use std::env;

    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let claims = Claims {
        sub: user.id.to_string(),
        exp: (Utc::now() + Duration::minutes(5)).timestamp() // 5 minutes
    };

    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .unwrap();

    token
}