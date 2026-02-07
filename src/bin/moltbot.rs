use reqwest::blocking::Client;
use serde_json::json;

use moltbot_server::token_store;

fn help() {
    println!(
        r#"Uso:
  moltbot set-token <TOKEN>     Guarda token (verifica con server)
  moltbot whoami               Muestra rol del token guardado
  moltbot cmd <COMANDO>        Ejecuta comando (ej: PING)
  moltbot logout               Borra token

Ejemplos:
  moltbot set-token admin123
  moltbot whoami
  moltbot cmd PING
"#
    );
}

fn api_base() -> String {
    std::env::var("MOLTBOT_API").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string())
}

// ✅ Ajustado a tu server: endpoint /login
fn verify_token(token: &str) -> Result<String, String> {
    let url = format!("{}/login", api_base());

    // en tus pruebas estabas usando { user, pass }
    // aquí tratamos token como "pass" para validar rápido.
    let body = json!({ "user": "admin", "pass": token });

    let client = Client::new();
    let res = client
        .post(url)
        .json(&body)
        .send()
        .map_err(|e| format!("No se pudo conectar: {e}"))?;

    let text = res.text().unwrap_or_default();

    // tu server responde "LOGIN_OK" si es válido
    if text.contains("LOGIN_OK") {
        // si tu JSON trae role, mejor parsearlo, pero esto sirve rápido
        // (si tienes role="ADMIN" o similar en el texto)
        if text.contains("ADMIN") {
            Ok("ADMIN".into())
        } else {
            Ok("USER".into())
        }
    } else {
        Err(text)
    }
}

fn send_cmd(token: &str, command: &str) -> Result<(), String> {
    let url = format!("{}/cmd", api_base());

    let body = json!({
        "token": token,
        "message": command
    });

    let client = Client::new();
    let res = client
        .post(url)
        .json(&body)
        .send()
        .map_err(|e| format!("No se pudo conectar: {e}"))?;

    let text = res.text().unwrap_or_default();
    println!("{}", text);
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        help();
        return;
    }

    match args[1].as_str() {
        "set-token" => {
            if args.len() < 3 {
                eprintln!("Falta TOKEN");
                return;
            }

            let token = &args[2];

            match verify_token(token) {
                Ok(role) => {
                    token_store::save_token(token).expect("No pude guardar token");
                    println!("✅ Token guardado. Rol: {}", role);
                }
                Err(e) => eprintln!("❌ Token inválido: {}", e),
            }
        }

        "whoami" => {
            let token = match token_store::load_token() {
                Ok(Some(t)) => t,
                Ok(None) => {
                    eprintln!("❌ No hay token guardado. Usa: moltbot set-token <TOKEN>");
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("❌ Error leyendo token: {e}");
                    std::process::exit(1);
                }
            };

            match verify_token(&token) {
                Ok(role) => println!("✅ Rol: {}", role),
                Err(e) => println!("❌ Token guardado pero inválido: {}", e),
            }
        }

        "cmd" => {
            if args.len() < 3 {
                eprintln!("Falta comando");
                return;
            }

            let command = args[2..].join(" ");

            let token = match token_store::load_token() {
                Ok(Some(t)) => t,
                Ok(None) => {
                    eprintln!("❌ No hay token guardado. Usa: moltbot set-token <TOKEN>");
                    return;
                }
                Err(e) => {
                    eprintln!("❌ Error leyendo token: {e}");
                    return;
                }
            };

            if let Err(e) = send_cmd(&token, &command) {
                eprintln!("❌ Error: {}", e);
            }
        }

        "logout" => {
            if let Err(e) = token_store::delete_token() {
                eprintln!("❌ No pude borrar token: {}", e);
            } else {
                println!("✅ Token borrado");
            }
        }

        _ => help(),
    }
}
