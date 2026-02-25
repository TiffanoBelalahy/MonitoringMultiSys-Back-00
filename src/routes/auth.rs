use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{SaltString, PasswordHash, rand_core::OsRng};
use jsonwebtoken::{encode, Header, EncodingKey};
use std::env;
use sqlx::Row;
use axum::http::StatusCode;
use axum::extract::Path;
use axum::{
    response::IntoResponse,
};

use crate::state::AppState;

#[derive(Deserialize)]
pub struct RegisterInput {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
}

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

// ---------------- REGISTER ----------------
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterInput>,
) -> Json<String> {

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO users (id, email, password) VALUES ($1, $2, $3)"
    )
    .bind(id)
    .bind(&payload.email)
    .bind(&password_hash)
    .execute(&state.db)
    .await
    .unwrap();

    Json("User created".into())
}

// ---------------- LOGIN ----------------
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginInput>,
) -> Json<AuthResponse> {

    let row = sqlx::query(
        "SELECT id, password FROM users WHERE email = $1"
    )
    .bind(&payload.email)
    .fetch_one(&state.db)
    .await
    .unwrap();

    let id: Uuid = row.get("id");
    let password: String = row.get("password");

    let parsed_hash = PasswordHash::new(&password).unwrap();
    let argon2 = Argon2::default();

    argon2
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .unwrap();

    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .unwrap()
        .timestamp() as usize;

    let claims = Claims {
        sub: id.to_string(),
        exp: expiration,
    };

    let secret = env::var("JWT_SECRET").unwrap();

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .unwrap();

    Json(AuthResponse { token })
}

// ---------------- DELETE USER ----------------
pub async fn delete_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await;

    match result {
        Ok(res) if res.rows_affected() > 0 => Ok(StatusCode::NO_CONTENT),
        Ok(_) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}