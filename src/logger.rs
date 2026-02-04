use chrono::Local;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;

const LOG_DIR: &str = "logs";
const LOG_PREFIX: &str = "clawdbot";

fn log_path_for_today() -> String {
    let date = Local::now().format("%Y-%m-%d").to_string();
    format!("{}/{}-{}.log", LOG_DIR, LOG_PREFIX, date)
}

fn write_line(line: &str) {
    // Crear carpeta logs si no existe
    if create_dir_all(LOG_DIR).is_err() {
        return;
    }

    let path = log_path_for_today();

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = file.write_all(line.as_bytes());
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i >= max {
            break;
        }
        out.push(ch);
    }
    out.push_str("…");
    out
}

pub fn log(ip: &str, rol: &str, command: &str, argument: &str, ok: bool) {
    let ts = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Evita argumentos gigantes (por ejemplo un PS largo)
    let arg = truncate(argument, 200);

    let line = format!(
        "{} | {} | {} | {} | {} | {}\n",
        ts,
        ip,
        rol,
        command,
        arg,
        if ok { "OK" } else { "ERROR" }
    );

    write_line(&line);
}

pub fn log_text(text: &str) {
    let ts = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let line = format!("{} | {}\n", ts, text);
    write_line(&line);
}
