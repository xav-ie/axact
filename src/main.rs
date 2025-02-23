// use axum::handler::HandlerWithoutStateExt;
use axum::{routing::get, Router};

async fn handler() -> &'static str {
    "Hello, World!"
}

// Potentially simpler way to write, but I am following faterthanlime as close
// as possible for now
// #[tokio::main]
// async fn main() {
//     let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
//     axum::serve(listener, handler.into_make_service())
//         .await
//         .unwrap();
//
//     println!("Hello, world!");
// }

#[tokio::main]
async fn main() {
    let router = Router::new().route("/", get(|| handler()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    let addr = listener.local_addr().unwrap();
    println!("Listening on {addr}");
    axum::serve(listener, router).await.unwrap();
}
