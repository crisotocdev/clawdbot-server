use std::process::Command;
use std::time::Instant;

use crate::auth::Rol;
use crate::powershell;
use serde::Serialize;

use sysinfo::System;

#[derive(Serialize)]
struct StatusResponse {
    status: String,
    version: String,
    uptime: String,
    os: String,
    pid: u32,
    cpu: f32,
    ram_used: f64,
    ram_total: f64,
}

fn format_uptime(started_at: Instant) -> String {
    let secs = started_at.elapsed().as_secs();

    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

pub fn handle_message(role: Rol, msg: &str, started_at: Instant) -> (bool, String) {
    let msg = msg.trim();

    if msg.is_empty() {
        return (false, "MENSAJE_VACIO".to_string());
    }

    // Parse: COMANDO ARG...
    let mut partes = msg.splitn(2, ' ');
    let comando = partes.next().unwrap_or("").to_uppercase();
    let argumento = partes.next().unwrap_or("").to_string();

    if comando.is_empty() {
        return (false, "FORMATO_INVALIDO".to_string());
    }

    match comando.as_str() {
        // =========================
        // USER + ADMIN
        // =========================
        "PING" => (true, "PONG".to_string()),

        "TIME" => (true, powershell::ejecutar("GET_TIME")),

        "PROCESOS" => (true, powershell::ejecutar("LIST_PROCESSES")),

        "WHOAMI" => (true, powershell::ejecutar("WHOAMI")),

        "SYSINFO" => (true, powershell::ejecutar("SYSINFO")),

        // =========================
        // INFO / INTERNOS
        // =========================
        "VERSION" => (
            true,
            format!("MOLTBOT_VERSION {}", env!("CARGO_PKG_VERSION")),
        ),

        "STATUS" => {
            let mut sys = System::new_all();
            sys.refresh_all();

            let resp = StatusResponse {
                status: "online".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                uptime: format_uptime(started_at),
                os: std::env::consts::OS.to_string(),
                pid: std::process::id(),
                cpu: sys.global_cpu_info().cpu_usage(),
                ram_used: sys.used_memory() as f64 / 1024.0 / 1024.0,
                ram_total: sys.total_memory() as f64 / 1024.0 / 1024.0,
            };

            (
                true,
                serde_json::to_string_pretty(&resp).unwrap()
            )
        }

                "HELP" => (
                    true,
                    [
                        "PING",
                        "TIME",
                        "PROCESOS",
                        "WHOAMI",
                        "SYSINFO",
                        "STATUS",
                        "HELP",
                        "VERSION",
                        "NOTA",
                        "VSCODE",
                        "CHROME",
                        "PS",
                    ]
                    .join("\n"),
                ),

        // =========================
        // ADMIN ONLY
        // =========================
        "NOTA" => {
            if role != Rol::Admin {
                return (false, "FORBIDDEN".to_string());
            }

            match Command::new("notepad.exe").spawn() {
                Ok(_) => (true, "NOTEPAD_ABIERTO".to_string()),
                Err(e) => (false, format!("ERROR_NOTEPAD: {}", e)),
            }
        }

        "VSCODE" => {
            if role != Rol::Admin {
                return (false, "FORBIDDEN".to_string());
            }

            if Command::new("cmd").args(["/C", "code"]).spawn().is_ok() {
                return (true, "VSCODE_ABIERTO".to_string());
            }

            let ruta = r"C:\Program Files\Microsoft VS Code\Code.exe";
            match Command::new(ruta).spawn() {
                Ok(_) => (true, "VSCODE_ABIERTO".to_string()),
                Err(e) => (false, format!("ERROR_VSCODE: {}", e)),
            }
        }

        "CHROME" => {
            if role != Rol::Admin {
                return (false, "FORBIDDEN".to_string());
            }

            let chrome = r"C:\Program Files\Google\Chrome\Application\chrome.exe";
            match Command::new(chrome).spawn() {
                Ok(_) => (true, "CHROME_ABIERTO".to_string()),
                Err(e) => (false, format!("ERROR_CHROME: {}", e)),
            }
        }

        "PS" => {
            if role != Rol::Admin {
                return (false, "FORBIDDEN".to_string());
            }

            if argumento.is_empty() {
                return (false, "FALTA_ARGUMENTO_PS".to_string());
            }

            (true, powershell::ejecutar(&argumento))
        }

        // =========================
        // UNKNOWN
        // =========================
        _ => (false, "COMANDO_DESCONOCIDO".to_string()),
    }
}