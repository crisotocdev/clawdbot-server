use std::net::SocketAddr;

use moltbot_server::app;

#[tokio::main]
async fn main() {
    // Carga variables de entorno desde .env si existe
    let _ = dotenvy::dotenv();

    println!("🚀 Iniciando MoltBot Server...");

    let app = app::build_router();

    let addr: SocketAddr = "0.0.0.0:8080"
        .parse()
        .expect("Dirección inválida");

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
