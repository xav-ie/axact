use axum::{
    extract::{
        ws::{Message, Utf8Bytes, WebSocket},
        State, WebSocketUpgrade,
    },
    http::Response,
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use core::time;
use std::sync::{Arc, Mutex};
use sysinfo::System;

#[derive(Clone, Default)]
struct AppState {
    cpus: Arc<Mutex<Vec<f32>>>,
}

#[axum::debug_handler]
async fn mjs_get() -> impl IntoResponse {
    let file_contents = tokio::fs::read_to_string("src/index.mjs").await.unwrap();
    Response::builder()
        .status(200)
        .header("Content-Type", "application/javascript;charset=utf-8")
        .body(file_contents)
        .unwrap()
}

#[axum::debug_handler]
async fn root_get() -> impl IntoResponse {
    let file_contents = tokio::fs::read_to_string("src/index.html").await.unwrap();
    Html(file_contents)
}

#[axum::debug_handler]
async fn cpus_get(State(state): State<AppState>) -> Json<Vec<f32>> {
    let v = state.cpus.lock().unwrap().clone();
    Json(v)
}

#[tokio::main]
async fn main() {
    let app_state = AppState::default();

    let router = Router::new()
        .route("/", get(root_get))
        .route("/index.html", get(root_get))
        .route("/index.mjs", get(mjs_get))
        .route("/api/cpus", get(cpus_get))
        .route("/api/realtime_cpus", get(realtime_cpus_get))
        .with_state(app_state.clone());

    tokio::task::spawn_blocking(move || {
        let mut sys = System::new();
        loop {
            sys.refresh_cpu_usage();
            let v: Vec<_> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
            {
                let mut cpus = app_state.cpus.lock().unwrap();
                *cpus = v;
            }
            std::thread::sleep(time::Duration::from_secs(1));
        }
    });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    let addr = listener.local_addr().unwrap();
    println!("Listening on http://{addr}");
    axum::serve(listener, router).await.unwrap();
}

#[axum::debug_handler]
async fn realtime_cpus_get(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|ws: WebSocket| async { realtime_cpus_stream(state, ws).await })
}

async fn realtime_cpus_stream(app_state: AppState, mut ws: WebSocket) {
    let mut interval = tokio::time::interval(time::Duration::from_secs(1));
    loop {
        interval.tick().await;
        let payload = {
            let cpus = app_state.cpus.lock().unwrap();
            serde_json::to_string(&*cpus).unwrap()
        };
        ws.send(Message::Text(Utf8Bytes::from(payload)))
            .await
            .unwrap();
    }
}
