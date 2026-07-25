use std::net::SocketAddr;

use moltbot_server::app;

#[tokio::main]
async fn main() {
    // Cargar variables desde .env
    match dotenvy::dotenv() {
        Ok(_) => println!("✅ .env cargado"),
        Err(_) => println!("⚠️ No se encontró archivo .env (usando variables del sistema)"),
    }

    // Chequeo rápido de tokens
    if std::env::var("MOLTBOT_ADMIN_TOKEN").is_err() {
        println!("⚠️ Falta MOLTBOT_ADMIN_TOKEN");
    }
    if std::env::var("MOLTBOT_USER_TOKEN").is_err() {
        println!("⚠️ Falta MOLTBOT_USER_TOKEN");
    }

    println!("🚀 Iniciando MoltBot Server...");

    let app = app::build_router();

    let addr: SocketAddr = "0.0.0.0:8080".parse().expect("Dirección inválida");

    println!("🌐 HTTP server escuchando en http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("❌ No se pudo bindear el puerto 8080");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("❌ Error levantando el servidor HTTP");
}
