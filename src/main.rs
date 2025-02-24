use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    http::Response,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use core::time;
use sysinfo::System;
use tokio::sync::broadcast;

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<Snapshot>,
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

type Snapshot = Vec<f32>;

#[tokio::main]
async fn main() {
    let (tx, _) = broadcast::channel::<Snapshot>(1);

    let app_state = AppState { tx: tx.clone() };

    let router = Router::new()
        .route("/", get(root_get))
        .route("/index.html", get(root_get))
        .route("/index.mjs", get(mjs_get))
        .route("/api/realtime_cpus", get(realtime_cpus_get))
        .with_state(app_state.clone());

    tokio::task::spawn_blocking(move || {
        let mut sys = System::new();
        loop {
            sys.refresh_cpu_usage();
            let v: Vec<_> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
            let _ = tx.send(v);
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
    let mut rx = app_state.tx.subscribe();
    while let Ok(msg) = rx.recv().await {
        let payload = serde_json::to_string(&msg).unwrap();
        if let Err(e) = ws.send(Message::Text(payload.into())).await {
            let mut current_error: Option<&(dyn std::error::Error)> = Some(&e);
            let mut should_break = false;

            // errors form a chain, and we must traverse the chain...
            while let Some(err) = current_error {
                if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
                    should_break = matches!(
                        io_err.kind(),
                        std::io::ErrorKind::BrokenPipe | std::io::ErrorKind::ConnectionReset
                    );
                    break;
                }
                current_error = err.source();
            }

            if should_break {
                break;
            }

            eprintln!("WebSocket send error: {:?}", e);
            break;
        }
    }
}
