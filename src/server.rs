use std::io::{Write, BufRead, BufReader};
use std::net::TcpListener;

use crate::auth::{rol, Rol};
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
                // Reader separado para poder leer líneas del cliente
                let mut reader = BufReader::new(stream.try_clone().expect("No se pudo clonar stream"));

                // 1) Pedir token
                if stream.write_all(b"TOKEN:\n").is_err() {
                    continue;
                }
                let _ = stream.flush();

                // 2) Leer token
                let mut token_line = String::new();
                if reader.read_line(&mut token_line).is_err() {
                    let _ = stream.write_all(b"ERR|LECTURA_TOKEN_FALLIDA\n");
                    continue;
                }

                // 3) Validar rol
                let role = match rol(&token_line) {
                    Some(r) => r,
                    None => {
                        let _ = stream.write_all(b"ERR|UNAUTHORIZED\n");
                        let _ = stream.flush();
                        continue;
                    }
                };

                // 4) Confirmar auth OK
                match role {
                    Rol::Admin => {
                        let _ = stream.write_all(b"OK|AUTH_ADMIN\n");
                    }
                    Rol::User => {
                        let _ = stream.write_all(b"OK|AUTH_USER\n");
                    }
                }
                let _ = stream.flush();

                // 5) Leer el mensaje real (una línea) y procesarlo
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

                println!("📱 ({:?}) Mensaje recibido: {}", role, mensaje);

                // 6) Manejo de comandos (en 4E usaremos el role para permisos)
                let (ok, respuesta) = commands::handle_message(mensaje);

                let salida = if ok {
                    format!("OK|{}\n", respuesta)
                } else {
                    format!("ERR|{}\n", respuesta)
                };

                let _ = stream.write_all(salida.as_bytes());
                let _ = stream.flush();
            }

            Err(e) => eprintln!("❌ Error de conexión: {}", e),
        }
    }
}
