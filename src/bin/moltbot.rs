use std::fs;
use std::path::PathBuf;
use reqwest::blocking::Client;
use serde_json::json;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Uso: moltbot <COMANDO>");
        std::process::exit(1);
    }

    let command = args[1..].join(" ");

    // ---------------------------
    // Token path: %USERPROFILE%\.moltbot\token.txt
    // ---------------------------
    let mut token_path = PathBuf::from(std::env::var("USERPROFILE").unwrap());
    token_path.push(".moltbot");
    token_path.push("token.txt");

    if !token_path.exists() {
        eprintln!("❌ No existe token: {:?}", token_path);
        eprintln!("Ejecuta primero: moltbot login");
        std::process::exit(1);
    }

    let token = fs::read_to_string(&token_path)
        .expect("No se pudo leer token")
        .trim()
        .to_string();

    let url = "http://127.0.0.1:8080/cmd";

    println!("🔌 URL: {}", url);
    println!("🔑 Token: {}...", &token[0..std::cmp::min(4, token.len())]);
    println!("➡ Enviando comando: {}", command);

    let body = json!({
        "token": token,
        "message": command
    });

    let client = Client::new();
    let res = client
        .post(url)
        .json(&body)
        .send();

    match res {
        Ok(r) => {
            let status = r.status();
            let text = r.text().unwrap_or_default();
            println!("HTTP {}", status);
            println!("{}", text);
        }
        Err(e) => {
            eprintln!("❌ No se pudo conectar al server");
            eprintln!("{}", e);
            eprintln!("¿Está corriendo moltbot_server?");
        }
    }
}
