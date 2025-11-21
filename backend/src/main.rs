use axum::serve;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use crate::server::state::init_app_state;

mod server;

#[tokio::main]
async fn main() {
    let app_state = init_app_state("data/canvas.json");

    let app = server::routes::create_router()
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Listening on http://{}", addr);

    serve(listener, app).await.unwrap();
}