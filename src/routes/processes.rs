use axum::{
    extract::{Query, State, Path, Json, ConnectInfo},
    routing::{get, post},
};
use serde::Deserialize;
use crate::state::AppState;
use std::net::SocketAddr;
use sqlx::Row;


#[derive(Deserialize)]
pub struct Params {
    pub agent_id: String,
}

pub async fn processes(
    Query(params): Query<Params>,
    State(state): State<AppState>,
) -> Json<serde_json::Value> {

    let agents = state.agents.read().await;

    if let Some(agent) = agents.get(&params.agent_id) {
        Json(serde_json::json!({
            "processes": agent.processes,
            "systemStats": agent.system_stats
        }))
    } else {
        Json(serde_json::json!({
            "processes": [],
            "systemStats": null
        }))
    }
   

}


pub async fn push_metrics(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(agent_id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) {
    // 🔥 créer agent automatiquement
    let _ = sqlx::query(
        "INSERT INTO agents (id, name)
         VALUES ($1, $2)
         ON CONFLICT (id) DO NOTHING"
    )
    .bind(&agent_id)
    .bind(&agent_id)
    .execute(&state.db)
    .await;



    let mut agents = state.agents.write().await;

    // Si l'agent n'existe pas encore, on l'ajoute
    let agent_entry = agents.entry(agent_id.clone()).or_insert(crate::state::AgentPayload {
        processes: serde_json::Value::Array(vec![]),
        system_stats: serde_json::Value::Null,
        ip: addr.ip().to_string(),
    });

    // Mettre à jour les metrics
    agent_entry.processes = payload["processes"].clone();
    agent_entry.system_stats = payload["system_stats"].clone();

    // 🔥 Sauvegarde en base des metrics
    if let Some(stats) = payload.get("system_stats") {
        let cpu = stats["cpu_usage"].as_f64().unwrap_or(0.0);
        let memory_used = stats["memory_used"].as_i64().unwrap_or(0);
        let memory_total = stats["memory_total"].as_i64().unwrap_or(0);
        let network_receive = stats["network_receive"].as_i64().unwrap_or(0);
        let network_transmit = stats["network_transmit"].as_i64().unwrap_or(0);

        let _ = sqlx::query(
            "INSERT INTO agent_metrics 
            (agent_id, cpu_usage, memory_used, memory_total, network_receive, network_transmit)
            VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(&agent_id)
        .bind(cpu)
        .bind(memory_used)
        .bind(memory_total)
        .bind(network_receive)
        .bind(network_transmit)
        .execute(&state.db)
        .await;
    }

    agent_entry.ip = addr.ip().to_string();

    println!("Metrics received from agent {}", agent_id);
}




#[derive(Deserialize)]
pub struct KillParams {
    pub agent_id: String,
    pub pid: u32,
}

pub async fn kill_process(
    State(state): State<AppState>,
    Json(payload): Json<KillParams>,
) -> Json<bool> {

    let mut commands = state.commands.write().await;

    commands
        .entry(payload.agent_id.clone())
        .or_insert_with(Vec::new)
        .push(serde_json::json!({
            "type": "kill",
            "pid": payload.pid
        }));

    println!("Kill command queued for agent {}", payload.agent_id);

    Json(true)
}


pub async fn get_commands(
    Path(agent_id): Path<String>,
    State(state): State<AppState>,
) -> Json<Vec<serde_json::Value>> {

    let mut commands = state.commands.write().await;

    let agent_commands = commands
        .remove(&agent_id)
        .unwrap_or_default();

    Json(agent_commands)
}


#[derive(Deserialize)]
pub struct HistoryParams {
    pub range: String,
}

pub async fn metrics_history(
    Path(agent_id): Path<String>,
    Query(params): Query<HistoryParams>,
    State(state): State<AppState>,
) -> Json<Vec<serde_json::Value>> {

    let interval = match params.range.as_str() {
        "1m" => "1 minute",
        "1h" => "1 hour",
        "10h" => "10 hours",
        "1d" => "1 day",
        "1month" => "1 month",
        _ => "1 minute",
    };

    let rows = sqlx::query(
        &format!(
            "SELECT cpu_usage, memory_used, memory_total,
                    network_receive, network_transmit, created_at
             FROM agent_metrics
             WHERE agent_id = $1
             AND created_at >= NOW() - INTERVAL '{}'
             ORDER BY created_at ASC",
            interval
        )
    )
    .bind(&agent_id)
    .fetch_all(&state.db)
    .await
    .unwrap();

    let result: Vec<_> = rows.into_iter().map(|row| {
        serde_json::json!({
            "cpu_usage": row.get::<f64, _>("cpu_usage"),
            "memory_used": row.get::<i64, _>("memory_used"),
            "memory_total": row.get::<i64, _>("memory_total"),
            "network_receive": row.get::<i64, _>("network_receive"),
            "network_transmit": row.get::<i64, _>("network_transmit"),
            "created_at": row.get::<chrono::NaiveDateTime, _>("created_at").to_string(),
        })
    }).collect();

    Json(result)
}