use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

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

    // Estos valores se envían en MiB.
    // El frontend actualmente los divide entre 1024 para mostrarlos en GiB.
    ram_used: f64,
    ram_total: f64,
}

#[derive(Serialize)]
struct HelpResponse {
    user: Vec<&'static str>,
    admin: Vec<&'static str>,
}

fn format_uptime(started_at: Instant) -> String {
    let total_seconds = started_at.elapsed().as_secs();

    let days = total_seconds / 86_400;
    let hours = (total_seconds % 86_400) / 3_600;
    let minutes = (total_seconds % 3_600) / 60;
    let seconds = total_seconds % 60;

    let mut result = String::new();

    if days > 0 {
        result.push_str(&format!("{days}d "));
    }

    result.push_str(&format!(
        "{hours:02}:{minutes:02}:{seconds:02}"
    ));

    result
}


fn serialize_response<T: Serialize>(value: &T) -> (bool, String) {
    match serde_json::to_string(value) {
        Ok(json) => (true, json),
        Err(error) => (false, format!("ERROR_SERIALIZACION: {error}")),
    }
}

fn status_response(started_at: Instant) -> (bool, String) {
    /*
     * System::new_all() realiza una primera lectura del sistema.
     */
    let mut system = System::new_all();

    /*
     * El uso de CPU se calcula comparando mediciones.
     * Esperamos brevemente antes de refrescar nuevamente.
     */
    thread::sleep(Duration::from_millis(250));
    system.refresh_all();

    /*
     * Calculamos el promedio de uso de todos los núcleos.
     *
     * Así evitamos depender de:
     * - global_cpu_info(), utilizado por sysinfo antiguo.
     * - global_cpu_usage(), utilizado por sysinfo nuevo.
     */
    let cpu_usage = {
        let cpus = system.cpus();

        if cpus.is_empty() {
            0.0
        } else {
            let total_usage: f32 = cpus.iter().map(|cpu| cpu.cpu_usage()).sum();

            total_usage / cpus.len() as f32
        }
    };

    let response = StatusResponse {
        status: "online".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: format_uptime(started_at),
        os: std::env::consts::OS.to_string(),
        pid: std::process::id(),
        cpu: cpu_usage,

        /*
         * Convertimos bytes a MiB.
         * El frontend divide posteriormente entre 1024 para mostrar GiB.
         */
        ram_used: system.used_memory() as f64 / 1024.0 / 1024.0,
        ram_total: system.total_memory() as f64 / 1024.0 / 1024.0,
    };

    serialize_response(&response)
}

fn help_response() -> (bool, String) {
    let response = HelpResponse {
        user: vec![
            "PING", "TIME", "PROCESOS", "WHOAMI", "SYSINFO", "STATUS", "HELP", "VERSION",
        ],
        admin: vec!["NOTA", "VSCODE", "CHROME", "PS"],
    };

    serialize_response(&response)
}

pub fn handle_message(role: Rol, message: &str, started_at: Instant) -> (bool, String) {
    let message = message.trim();

    if message.is_empty() {
        return (false, "MENSAJE_VACIO".to_string());
    }

    /*
     * Separamos el nombre del comando y el resto del mensaje.
     *
     * Ejemplo:
     * "PS Get-Process chrome"
     *
     * comando   = "PS"
     * argumento = "Get-Process chrome"
     */
    let command_end = message.find(char::is_whitespace).unwrap_or(message.len());

    let command = message[..command_end].to_ascii_uppercase();
    let argument = message[command_end..].trim();

    if command.is_empty() {
        return (false, "FORMATO_INVALIDO".to_string());
    }

    match command.as_str() {
        // =========================
        // USER + ADMIN
        // =========================
        "PING" => (true, "PONG".to_string()),

        "TIME" => (true, powershell::ejecutar("GET_TIME")),

        "PROCESOS" => (true, powershell::ejecutar("LIST_PROCESSES")),

        "WHOAMI" => (true, powershell::ejecutar("WHOAMI")),

        "SYSINFO" => (true, powershell::ejecutar("SYSINFO")),

        "VERSION" => (
            true,
            format!("MOLTBOT_VERSION {}", env!("CARGO_PKG_VERSION")),
        ),

        "STATUS" => status_response(started_at),

        "HELP" => help_response(),

        // =========================
        // ADMIN ONLY
        // =========================
        "NOTA" => {
            if role != Rol::Admin {
                return (false, "FORBIDDEN".to_string());
            }

            match Command::new("notepad.exe").spawn() {
                Ok(_) => (true, "NOTEPAD_ABIERTO".to_string()),
                Err(error) => (false, format!("ERROR_NOTEPAD: {error}")),
            }
        }

        "VSCODE" => {
            if role != Rol::Admin {
                return (false, "FORBIDDEN".to_string());
            }

            /*
             * Primero intentamos ejecutar el comando `code`
             * disponible en el PATH.
             */
            if Command::new("cmd").args(["/C", "code"]).spawn().is_ok() {
                return (true, "VSCODE_ABIERTO".to_string());
            }

            /*
             * Instalación global de VS Code.
             */
            let program_files_path = r"C:\Program Files\Microsoft VS Code\Code.exe";

            if Command::new(program_files_path).spawn().is_ok() {
                return (true, "VSCODE_ABIERTO".to_string());
            }

            /*
             * Instalación de VS Code solo para el usuario.
             */
            if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
                let user_path = std::path::Path::new(&local_app_data)
                    .join("Programs")
                    .join("Microsoft VS Code")
                    .join("Code.exe");

                if Command::new(user_path).spawn().is_ok() {
                    return (true, "VSCODE_ABIERTO".to_string());
                }
            }

            (false, "ERROR_VSCODE: no se encontró Code.exe".to_string())
        }

        "CHROME" => {
            if role != Rol::Admin {
                return (false, "FORBIDDEN".to_string());
            }

            let paths = [
                r"C:\Program Files\Google\Chrome\Application\chrome.exe",
                r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
            ];

            for path in paths {
                if Command::new(path).spawn().is_ok() {
                    return (true, "CHROME_ABIERTO".to_string());
                }
            }

            if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
                let user_path = std::path::Path::new(&local_app_data)
                    .join("Google")
                    .join("Chrome")
                    .join("Application")
                    .join("chrome.exe");

                if Command::new(user_path).spawn().is_ok() {
                    return (true, "CHROME_ABIERTO".to_string());
                }
            }

            (false, "ERROR_CHROME: no se encontró chrome.exe".to_string())
        }

        "PS" => {
            if role != Rol::Admin {
                return (false, "FORBIDDEN".to_string());
            }

            if argument.is_empty() {
                return (false, "FALTA_ARGUMENTO_PS".to_string());
            }

            (true, powershell::ejecutar(argument))
        }

        // =========================
        // UNKNOWN
        // =========================
        _ => (false, "COMANDO_DESCONOCIDO".to_string()),
    }
}
