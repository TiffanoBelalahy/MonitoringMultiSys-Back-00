mod agent_client;
mod routes;
mod state;

use axum::{Router, routing::get};
use routes::processes::processes;
use tokio::net::TcpListener;
use state::AppState;

#[tokio::main]
async fn main() {
    let state = AppState::default();

    let app = Router::new()
        .route("/api/processes", get(processes))
        .with_state(state.clone());

    let addr = "0.0.0.0:8080";
    println!("Server running on http://{addr}");

    let listener = TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
