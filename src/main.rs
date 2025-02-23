use axum::{extract::State, routing::get, Router};
use std::sync::{Arc, Mutex};
use sysinfo::System;

#[derive(Clone)]
struct AppState {
    system: Arc<Mutex<System>>,
}

async fn root_get() -> &'static str {
    "Hello, world"
}

async fn cpus_get(State(state): State<AppState>) -> String {
    use ::std::fmt::Write;
    let mut sys = state.system.lock().unwrap();
    sys.refresh_cpu_usage();

    let mut s = String::new();
    for (index, cpu) in sys.cpus().iter().enumerate() {
        let index = index + 1;
        let usage = cpu.cpu_usage();
        writeln!(&mut s, "CPU {index} {usage}% ").unwrap();
    }
    s
}

#[tokio::main]
async fn main() {
    let state = AppState {
        system: Arc::new(Mutex::new(System::new())),
    };

    let router = Router::new()
        .route("/", get(root_get))
        .route("/api/cpus", get(cpus_get))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    let addr = listener.local_addr().unwrap();
    println!("Listening on http://{addr}");
    axum::serve(listener, router).await.unwrap();
}
