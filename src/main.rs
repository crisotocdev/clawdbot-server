use std::io::{Write, BufRead, BufReader};
use std::net::TcpListener;
use std::process::Command;

const TOKEN: &str = "CLAWDBOT_9F3A_2026_X7KQ_LMN82_SECURE";

fn main() {
    let direccion = "0.0.0.0:8080";
    let listener = TcpListener::bind(direccion)
        .expect("No se pudo iniciar el servidor");

    println!("🚀 Clawdbot Server iniciado en {}", direccion);
    println!("📡 Esperando conexión...");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut reader = BufReader::new(&mut stream);
                let mut mensaje = String::new();

                if reader.read_line(&mut mensaje).is_err() {
                    let _ = stream.write_all(b"ERR|LECTURA_FALLIDA\n");
                    continue;
                }

                let mensaje = mensaje.trim();
                if mensaje.is_empty() {
                    let _ = stream.write_all(b"ERR|MENSAJE_VACIO\n");
                    continue;
                }

                println!("📱 Mensaje recibido: {}", mensaje);

                let (ok, respuesta) = handle_message(mensaje);

                let salida = if ok {
                    format!("OK|{}\n", respuesta)
                } else {
                    format!("ERR|{}\n", respuesta)
                };

                let _ = stream.write_all(salida.as_bytes());
            }

            Err(e) => {
                eprintln!("❌ Error de conexión: {}", e);
            }
        }
    }
}

fn handle_message(msg: &str) -> (bool, String) {
    // Formato esperado: TOKEN|COMANDO|ARG
    let mut partes = msg.splitn(3, '|');

    let token = partes.next().unwrap_or("");
    let comando = partes.next().unwrap_or("").to_uppercase();
    let argumento = partes.next().unwrap_or("").to_string();

    if token != TOKEN {
        return (false, "TOKEN_INVALIDO".to_string());
    }

    match comando.as_str() {
        "PING" => (true, "PONG".to_string()),

        "NOTA" => match Command::new("notepad.exe").spawn() {
            Ok(_) => (true, "NOTEPAD_ABIERTO".to_string()),
            Err(e) => (false, format!("ERROR_NOTEPAD: {}", e)),
        },

        "VSCODE" => {
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
            let chrome = r"C:\Program Files\Google\Chrome\Application\chrome.exe";
            match Command::new(chrome).spawn() {
                Ok(_) => (true, "CHROME_ABIERTO".to_string()),
                Err(e) => (false, format!("ERROR_CHROME: {}", e)),
            }
        }

        "PS" => match ejecutar_powershell(&argumento) {
            Ok(out) => (true, out),
            Err(e) => (false, format!("ERROR_PS: {}", e)),
        },

        _ => (false, "COMANDO_DESCONOCIDO".to_string()),
    }
}

fn ejecutar_powershell(accion: &str) -> std::io::Result<String> {
    let script = match accion.to_uppercase().as_str() {
        "GET_TIME" => "Get-Date | Out-String",
        "LIST_PROCESSES" => "Get-Process | Select-Object -First 10 | Out-String",
        _ => return Ok("ACCION_NO_PERMITIDA".to_string()),
    };

    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy", "Bypass",
            "-Command", script
        ])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if !output.status.success() {
        return Ok(format!("STDERR: {}", stderr));
    }

    Ok(if stdout.is_empty() {
        "OK".to_string()
    } else {
        stdout
    })
}
