use std::io::{Write, BufRead, BufReader};
use std::net::TcpListener;

use crate::commands;

pub fn start() {
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

                let (ok, respuesta) = commands::handle_message(mensaje);

                let salida = if ok {
                    format!("OK|{}\n", respuesta)
                } else {
                    format!("ERR|{}\n", respuesta)
                };

                let _ = stream.write_all(salida.as_bytes());
            }

            Err(e) => eprintln!("❌ Error de conexión: {}", e),
        }
    }
}
