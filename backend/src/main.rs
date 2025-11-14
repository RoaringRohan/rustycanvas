use axum::serve;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use crate::server::state::init_shared_canvas;

mod server;

#[tokio::main]
async fn main() {
    let shared_canvas = init_shared_canvas();

    let app = server::routes::create_router()
        .with_state(shared_canvas);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Listening on http://{}", addr);

    serve(listener, app).await.unwrap();
}