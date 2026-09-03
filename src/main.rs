use connect4::{routes, state};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("Failed to read .env");

    let state = state::setup().await;
    let router = routes::router(state);

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to address");
    axum::serve(listener, router)
        .await
        .expect("Failed to servie app");
}
