use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use chrono::Local;

pub fn log(
    ip: &str,
    rol: &str,
    command: &str,
    argument: &str,
    ok: bool,
) {
    // Crear carpeta logs si no existe
    let _ = create_dir_all("logs");

    let ts = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let line = format!(
        "{} | {} | {} | {} | {} | {}\n",
        ts,
        ip,
        rol,
        command,
        argument,
        if ok { "OK" } else { "ERROR" }
    );

    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs/clawdbot.log")
    {
        let _ = file.write_all(line.as_bytes());
    }
}

pub fn log_text(line: &str) {
    let _ = create_dir_all("logs");
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs/clawdbot.log")
    {
        let _ = file.write_all(format!("{}\n", line).as_bytes());
    }
}

