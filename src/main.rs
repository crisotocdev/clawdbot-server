use std::net::SocketAddr;

use clawdbot_server::app;

#[tokio::main]
async fn main() {
    let app = app::build_router();

    let addr = "0.0.0.0:8080";
    println!("HTTP server iniciado en http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("No se pudo bindear el puerto");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Error levantando el servidor HTTP");
}
