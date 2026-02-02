mod logger;
mod auth;
mod commands;
mod powershell;
// mod server; // ← ya no lo usamos por ahora (era TCP crudo)

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

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/ping", get(ping))
        .route("/help", get(help))
        .route("/cmd", post(cmd));

    let addr = "0.0.0.0:8080";
    println!("HTTP server iniciado en http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("No se pudo bindear el puerto");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Error levantando el servidor HTTP");
}

// ---------- HANDLERS ----------

async fn ping() -> &'static str {
    "PONG"
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

    // 1) AUTH: bloquear al inicio si token inválido
    let role = match auth::rol(&payload.token) {
        Some(r) => r,
        None => {
            // Log de auth fallida
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

    // 2) Parse del mensaje para el log
    let msg = payload.message.trim();
    let mut parts = msg.splitn(2, ' ');
    let command = parts.next().unwrap_or("").to_uppercase();
    let argument = parts.next().unwrap_or("").to_string();

    // 3) Ejecutar comando (ahora sin token embebido)
    let result = catch_unwind(AssertUnwindSafe(|| commands::handle_message(role, msg)));

    let (ok, response) = match result {
        Ok((ok, resp)) => (ok, resp),
        Err(panic) => {
            let msg = if let Some(s) = panic.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic.downcast_ref::<String>() {
                s.clone()
            } else {
                "panic sin mensaje".to_string()
            };

            logger::log_text(&format!("PANIC /cmd: {}", msg));
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

    // 6) Respuesta OK
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
        name: "Clawdbot",
        version: env!("CARGO_PKG_VERSION"),
        endpoints: vec!["GET /ping", "GET /help", "POST /cmd"],
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
