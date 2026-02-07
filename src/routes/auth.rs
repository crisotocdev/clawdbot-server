use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::auth::{self, Rol};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub token: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub role: String,
}

pub fn router() -> Router {
    Router::new().route("/login", post(login))
}

async fn login(Json(payload): Json<LoginRequest>) -> impl IntoResponse {
    match auth::rol(&payload.token) {
        Some(Rol::Admin) => (
            StatusCode::OK,
            Json(LoginResponse { role: "admin".into() }),
        )
            .into_response(),
        Some(Rol::User) => (
            StatusCode::OK,
            Json(LoginResponse { role: "user".into() }),
        )
            .into_response(),
        None => (StatusCode::UNAUTHORIZED, "token invalido").into_response(),
    }
}
