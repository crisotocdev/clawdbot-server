use axum::{
    body::{to_bytes, Body},
    extract::ConnectInfo,
    http::{Request, StatusCode},
};
use serde_json::json;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tower::ServiceExt; // oneshot

fn set_env(key: &str, value: &str) {
    unsafe {
        std::env::set_var(key, value);
    }
}

#[tokio::test]
async fn ping_works() {
    let app = clawdbot_server::app::build_router();

    let req = Request::builder()
        .method("GET")
        .uri("/ping")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&body[..], b"PONG");
}

#[tokio::test]
async fn cmd_admin_ping_ok() {
    set_env("CLAWDBOT_ADMIN_TOKEN", "admin123");
    set_env("CLAWDBOT_USER_TOKEN", "user123");

    let app = clawdbot_server::app::build_router();

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 55555);

    let mut req = Request::builder()
        .method("POST")
        .uri("/cmd")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"token":"admin123","message":"PING"}).to_string(),
        ))
        .unwrap();

    // IMPORTANT: ConnectInfo es requerido por tu handler
    req.extensions_mut().insert(ConnectInfo(addr));

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(v["ok"], true);
    assert_eq!(v["role"], "ADMIN");
    assert_eq!(v["command"], "PING");
    assert_eq!(v["response"], "PONG");
}

#[tokio::test]
async fn cmd_user_cannot_open_vscode() {
    set_env("CLAWDBOT_ADMIN_TOKEN", "admin123");
    set_env("CLAWDBOT_USER_TOKEN", "user123");

    let app = clawdbot_server::app::build_router();
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 55555);

    let mut req = Request::builder()
        .method("POST")
        .uri("/cmd")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"token":"user123","message":"VSCODE"}).to_string(),
        ))
        .unwrap();

    req.extensions_mut().insert(ConnectInfo(addr));

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(v["ok"], false);
    assert_eq!(v["role"], "USER");
    assert_eq!(v["command"], "VSCODE");
    assert_eq!(v["response"], "FORBIDDEN");
}

#[tokio::test]
async fn cmd_invalid_token_returns_401() {
    set_env("CLAWDBOT_ADMIN_TOKEN", "admin123");
    set_env("CLAWDBOT_USER_TOKEN", "user123");

    let app = clawdbot_server::app::build_router();
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 55555);

    let mut req = Request::builder()
        .method("POST")
        .uri("/cmd")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"token":"xxx","message":"PING"}).to_string(),
        ))
        .unwrap();

    req.extensions_mut().insert(ConnectInfo(addr));

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(v["ok"], false);
    assert_eq!(v["response"], "UNAUTHORIZED");
}
