use axum::{
    extract::ConnectInfo,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::{auth, commands, logger};

pub fn build_router() -> Router {
    Router::new()
        .route("/ping", get(ping))
        .route("/help", get(help))
        .route("/status", get(status))
        .route("/login", post(login))
        .route("/cmd", post(cmd))
}

// ---------- HANDLERS ----------

async fn ping() -> &'static str {
    "PONG"
}

#[derive(Serialize)]
struct StatusResponse {
    name: &'static str,
    version: &'static str,
    online: bool,
}

async fn status() -> Json<StatusResponse> {
    Json(StatusResponse {
        name: "Moltbot IA",
        version: env!("CARGO_PKG_VERSION"),
        online: true,
    })
}

#[derive(Deserialize)]
struct LoginRequest {
    user: String,
    pass: String,
}

#[derive(Serialize)]
struct LoginResponse {
    ok: bool,
    role: String,
    token: String,
    response: String,
}

fn env_or(name: &str, fallback: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| fallback.to_string())
}

async fn login(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let ip = addr.ip().to_string();

    // Credenciales (con defaults)
    let admin_user = env_or("MOLTBOT_ADMIN_USER", "admin");
    let admin_pass = env_or("MOLTBOT_ADMIN_PASS", "admin123");
    let user_user = env_or("MOLTBOT_USER_USER", "user");
    let user_pass = env_or("MOLTBOT_USER_PASS", "user123");

    // Tokens reales (deben existir como env)
    let admin_token = std::env::var("MOLTBOT_ADMIN_TOKEN").unwrap_or_default();
    let user_token = std::env::var("MOLTBOT_USER_TOKEN").unwrap_or_default();

    // ADMIN
    if payload.user == admin_user && payload.pass == admin_pass && !admin_token.is_empty() {
        logger::log(&ip, "ADMIN", "LOGIN", "", true);
        return (
            StatusCode::OK,
            Json(LoginResponse {
                ok: true,
                role: "ADMIN".to_string(),
                token: admin_token,
                response: "LOGIN_OK".to_string(),
            }),
        )
            .into_response();
    }

    // USER
    if payload.user == user_user && payload.pass == user_pass && !user_token.is_empty() {
        logger::log(&ip, "USER", "LOGIN", "", true);
        return (
            StatusCode::OK,
            Json(LoginResponse {
                ok: true,
                role: "USER".to_string(),
                token: user_token,
                response: "LOGIN_OK".to_string(),
            }),
        )
            .into_response();
    }

    // FAIL
    logger::log(&ip, "UNKNOWN", "LOGIN", "", false);
    (
        StatusCode::UNAUTHORIZED,
        Json(LoginResponse {
            ok: false,
            role: "UNKNOWN".to_string(),
            token: "".to_string(),
            response: "UNAUTHORIZED".to_string(),
        }),
    )
        .into_response()
}

#[derive(Deserialize)]
struct CmdRequest {
    token: String,
    message: String,
}

#[derive(Serialize)]
struct CmdResponse {
    ok: bool,
    role: String,
    command: String,
    argument: String,
    response: String,
}

async fn cmd(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<CmdRequest>,
) -> impl IntoResponse {
    let ip = addr.ip().to_string();

    // 1) AUTH
    let role = match auth::rol(&payload.token) {
        Some(r) => r,
        None => {
            logger::log(&ip, "UNKNOWN", "AUTH", "", false);

            let body = Json(CmdResponse {
                ok: false,
                role: "UNKNOWN".to_string(),
                command: "AUTH".to_string(),
                argument: "".to_string(),
                response: "UNAUTHORIZED".to_string(),
            });

            return (StatusCode::UNAUTHORIZED, body).into_response();
        }
    };

    // 2) Parse para log
    let msg = payload.message.trim();
    let mut parts = msg.splitn(2, ' ');
    let command = parts.next().unwrap_or("").to_uppercase();
    let argument = parts.next().unwrap_or("").to_string();

    // 3) Ejecutar comando (sin token embebido)
    let result = catch_unwind(AssertUnwindSafe(|| commands::handle_message(role, msg)));

    let (ok, response) = match result {
        Ok((ok, resp)) => (ok, resp),
        Err(_) => {
            logger::log_text("PANIC /cmd");
            (false, "ERROR_INTERNO_CMD".to_string())
        }
    };

    // 4) Rol en texto
    let role_str = match role {
        auth::Rol::Admin => "ADMIN",
        auth::Rol::User => "USER",
    };

    // 5) Log real
    logger::log(&ip, role_str, &command, &argument, ok);

    // 6) Respuesta
    let body = Json(CmdResponse {
        ok,
        role: role_str.to_string(),
        command,
        argument,
        response,
    });

    (StatusCode::OK, body).into_response()
}

#[derive(Serialize)]
struct HelpResponse {
    name: &'static str,
    version: &'static str,
    endpoints: Vec<&'static str>,
    commands: Vec<&'static str>,
    format: &'static str,
}

async fn help() -> Json<HelpResponse> {
    Json(HelpResponse {
        name: "moltbot",
        version: env!("CARGO_PKG_VERSION"),
        endpoints: vec![
            "GET /ping",
            "GET /help",
            "GET /status",
            "POST /login",
            "POST /cmd",
        ],
        commands: vec![
            "PING",
            "NOTA",
            "VSCODE",
            "CHROME",
            "PS <ACCION>",
            "TIME",
            "PROCESOS",
            "WHOAMI",
            "SYSINFO",
        ],
        format: r#"POST /cmd JSON: { "token": "...", "message": "PING" }"#,
    })
}
