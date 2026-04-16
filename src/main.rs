mod routes;
mod state;

use axum::{Router, routing::{get, post}};
use routes::processes::{processes, push_metrics, kill_process, get_commands};
use routes::agents::{list_agents, rename_agent, create_company, list_companies, create_agent, assign_agent_to_company};
use routes::auth::{login, register, delete_user};
use routes::processes::metrics_history;
use axum::routing::delete;

use tokio::net::TcpListener;
use state::AppState;

use tower_http::cors::{CorsLayer, Any};
use axum::http::Method;

use dotenvy::dotenv;
use std::{env, collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use sqlx::PgPool;


#[tokio::main]
async fn main() {

    // ✅ Charger .env AVANT tout
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL not set");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to DB");

    let state = AppState {
        agents: Arc::new(RwLock::new(HashMap::new())),
        commands: Arc::new(RwLock::new(HashMap::new())),
        db: pool.clone(),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS, Method::PUT])
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/processes", get(processes))
        .route("/api/processes/kill", post(kill_process))
        .route("/api/agents", get(list_agents))
        .route("/api/agents/:agent_id/commands", get(get_commands))
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/users/:user_id", delete(delete_user))
        .route("/api/agents/:agent_id/metrics", post(push_metrics))
        .route("/api/agents/:agent_id/history", get(metrics_history))
        .route("/api/agents/:agent_id", axum::routing::put(rename_agent))
        .route("/api/companies", get(list_companies))
        .route("/api/companies", post(create_company))
        .route("/api/agents", post(create_agent))
        .route("/api/agents/:agent_id/assign", axum::routing::put(assign_agent_to_company))
        .with_state(state)
        .layer(cors);

    let addr = "0.0.0.0:8081";
    println!("Server running on http://{addr}"); 

    let listener = TcpListener::bind(addr).await.unwrap();

    axum::serve(
    listener,
    app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
        .await
        .unwrap();
}