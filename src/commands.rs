use std::process::Command;

use crate::auth::Rol;
use crate::powershell;

pub fn handle_message(role: Rol, msg: &str) -> (bool, String) {
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
