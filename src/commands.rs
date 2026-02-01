use std::process::Command;

use crate::auth::{self, Rol};
use crate::powershell;

pub fn handle_message(msg: &str) -> (bool, String) {
    // Soportar 2 formatos:
    // 1) HTTP nuevo: "TOKEN COMANDO ARG..."
    // 2) TCP viejo:  "TOKEN|COMANDO|ARG..."

    let msg = msg.trim();

    // --- Parse ---
    let (token, comando, argumento) = if msg.contains('|') {
        // Formato viejo: TOKEN|COMANDO|ARG
        let mut partes = msg.splitn(3, '|');
        (
            partes.next().unwrap_or("").trim().to_string(),
            partes.next().unwrap_or("").trim().to_uppercase(),
            partes.next().unwrap_or("").trim().to_string(),
        )
    } else {
        // Formato nuevo: TOKEN COMANDO ARG...
        let mut partes = msg.splitn(3, ' ');
        (
            partes.next().unwrap_or("").trim().to_string(),
            partes.next().unwrap_or("").trim().to_uppercase(),
            partes.next().unwrap_or("").trim().to_string(),
        )
    };

    if token.is_empty() || comando.is_empty() {
        return (false, "FORMATO_INVALIDO".to_string());
    }

    let rol = match auth::rol(&token) {
        Some(r) => r,
        None => return (false, "TOKEN_INVALIDO".to_string()),
    };

    // --- Commands ---
    match comando.as_str() {
    // --- USER permitido ---
    "PING" => (true, "PONG".to_string()),
    "TIME" => (true, powershell::ejecutar("GET_TIME")),
    "PROCESOS" => (true, powershell::ejecutar("LIST_PROCESSES")),
    "WHOAMI" => (true, powershell::ejecutar("WHOAMI")),
    "SYSINFO" => (true, powershell::ejecutar("SYSINFO")),

    // --- ADMIN solamente ---
    "NOTA" => {
        if rol != Rol::Admin {
            return (false, "PERMISO_DENEGADO".to_string());
        }
        match Command::new("notepad.exe").spawn() {
            Ok(_) => (true, "NOTEPAD_ABIERTO".to_string()),
            Err(e) => (false, format!("ERROR_NOTEPAD: {}", e)),
        }
    }

    "VSCODE" => {
        if rol != Rol::Admin {
            return (false, "PERMISO_DENEGADO".to_string());
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
        if rol != Rol::Admin {
            return (false, "PERMISO_DENEGADO".to_string());
        }

        let chrome = r"C:\Program Files\Google\Chrome\Application\chrome.exe";
        match Command::new(chrome).spawn() {
            Ok(_) => (true, "CHROME_ABIERTO".to_string()),
            Err(e) => (false, format!("ERROR_CHROME: {}", e)),
        }
    }

    "PS" => {
        if rol != Rol::Admin {
            return (false, "PERMISO_DENEGADO".to_string());
        }
        if argumento.is_empty() {
            return (false, "FALTA_ARGUMENTO_PS".to_string());
        }
        (true, powershell::ejecutar(&argumento))
    }

    _ => (false, "COMANDO_DESCONOCIDO".to_string()),
    }

}
