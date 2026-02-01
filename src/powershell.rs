use std::process::Command;

/// Ejecuta acciones permitidas en PowerShell
/// Devuelve siempre String para integrarse fácil con HTTP / JSON
pub fn ejecutar(accion: &str) -> String {
    let accion = accion.trim().to_uppercase();

    let script = match accion.as_str() {
        "GET_TIME" => "Get-Date | Out-String",
        "LIST_PROCESSES" => "Get-Process | Select-Object -First 10 | Out-String",
        "WHOAMI" => "whoami | Out-String",
        "SYSINFO" => "systeminfo | Select-Object -First 20 | Out-String",
        _ => return "ACCION_NO_PERMITIDA".to_string(),
    };

    let output = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .output();

    let output = match output {
        Ok(o) => o,
        Err(e) => return format!("ERROR_EJECUCION: {}", e),
    };

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if !output.status.success() {
        if stderr.is_empty() {
            return "ERROR_DESCONOCIDO".to_string();
        } else {
            return format!("STDERR: {}", stderr);
        }
    }

    if stdout.is_empty() {
        "OK".to_string()
    } else {
        stdout
    }
}
