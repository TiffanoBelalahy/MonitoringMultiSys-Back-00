mod routes;
mod state;

use axum::{Router, routing::{get, post}};
use routes::processes::{processes, push_metrics, kill_process, get_commands};
use tokio::net::TcpListener;
use state::AppState;
use routes::agents::list_agents;

use tower_http::cors::{CorsLayer, Any};
use axum::http::Method;



#[tokio::main]
async fn main() {
    let state = AppState::default();

    let cors = CorsLayer::new()
        .allow_origin(Any) // DEV ONLY
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/processes", get(processes))
        .route("/api/processes/kill", post(kill_process))
        .route("/api/agents", get(list_agents))
        .route("/api/agents/:agent_id/commands", get(get_commands))
        .route(
            "/api/agents/:agent_id/metrics",
            post(push_metrics),
        )
        .with_state(state.clone())
        .layer(cors); // 👈 ICI LE FIX


    let addr = "0.0.0.0:8081";
    println!("Server running on http://{addr}");

    let listener = TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>())
    .await
    .unwrap();

}
