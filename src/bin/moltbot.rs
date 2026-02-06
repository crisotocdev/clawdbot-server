use std::env;

#[derive(serde::Deserialize)]
struct CmdResponse {
    ok: bool,
    role: String,
    command: String,
    argument: String,
    response: String,
}

fn main() {
    // Uso: moltbot <COMANDO> [ARG...]
    // Ej: moltbot PING
    // Ej: moltbot TIME
    // Ej: moltbot PS dir

    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("Uso: moltbot <COMANDO> [ARG...]");
        eprintln!(r#"Ej: moltbot PING"#);
        eprintln!(r#"Ej: moltbot TIME"#);
        eprintln!(r#"Ej: moltbot PS dir"#);
        std::process::exit(2);
    }

    let token = env::var("MOLTBOT_TOKEN")
        .or_else(|_| env::var("CLAWDBOT_ADMIN_TOKEN")) // fallback por si aún lo usas
        .unwrap_or_else(|_| {
            eprintln!("Falta variable de entorno MOLTBOT_TOKEN.");
            eprintln!(r#"PowerShell: $env:MOLTBOT_TOKEN="admin123""#);
            std::process::exit(2);
        });

    let base_url = env::var("MOLTBOT_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());

    let message = args.join(" ");
    let url = format!("{}/cmd", base_url);

    let payload = serde_json::json!({
        "token": token,
        "message": message
    });

    // Cliente HTTP simple (bloqueante) usando reqwest::blocking
    // (lo agregaremos como dependencia)
    let client = reqwest::blocking::Client::new();
    let res = client
        .post(url)
        .header("Content-Type", "application/json")
        .body(payload.to_string())
        .send();

    match res {
    Ok(r) => {
        let status = r.status();
        let text = r.text().unwrap_or_else(|_| "<sin body>".to_string());

        // Intentar parsear JSON
        if let Ok(v) = serde_json::from_str::<CmdResponse>(&text) {
            if status.is_success() && v.ok {
                println!("✅ OK ({})", v.role);
            } else {
                println!("❌ ERROR ({})", v.role);
            }

            if v.argument.trim().is_empty() {
                println!("{} -> {}", v.command, v.response);
            } else {
                println!("{} {} -> {}", v.command, v.argument, v.response);
            }
        } else {
            // Si no es JSON, mostrar crudo
            println!("HTTP {}", status);
            println!("{}", text);
        }
    }
    Err(e) => {
        eprintln!("Error enviando request: {}", e);
        std::process::exit(1);
    }
  }
}
