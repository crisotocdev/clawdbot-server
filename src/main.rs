mod logger;
mod auth;
mod commands;
mod powershell;
// mod server; // ← ya no lo usamos por ahora (era TCP crudo)

use axum::{
    routing::{get, post},
    Json, Router,
    extract::ConnectInfo,
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
    command: String,
    argument: String,
    response: String,
}

async fn cmd(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<CmdRequest>,
) -> Json<CmdResponse> {

    let ip = addr.ip().to_string();

    let msg = payload.message.trim();
    let mut parts = msg.splitn(2, ' ');
    let command = parts.next().unwrap_or("").to_uppercase();
    let argument = parts.next().unwrap_or("").to_string();

    let full_message = format!("{} {}", payload.token, payload.message);
    let result = catch_unwind(AssertUnwindSafe(|| commands::handle_message(&full_message)));

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

    // Determinar rol para el log
    let rol = match auth::rol(&payload.token) {
        Some(auth::Rol::Admin) => "ADMIN",
        Some(auth::Rol::User) => "USER",
        None => "UNKNOWN",
    };

    // 👉 LOG REAL
    logger::log(
        &ip,
        rol,
        &command,
        &argument,
        ok,
    );

    Json(CmdResponse {
        ok,
        command,
        argument,
        response,
    })
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
