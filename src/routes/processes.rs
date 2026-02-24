use axum::{
    extract::{Query, State, Path, Json, ConnectInfo},
    routing::{get, post},
};
use serde::Deserialize;
use crate::state::AppState;
use std::net::SocketAddr;

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


// pub async fn push_metrics(
//     ConnectInfo(addr): ConnectInfo<SocketAddr>,
//     Path(agent_id): Path<String>,
//     State(state): State<AppState>,
//     Json(payload): Json<serde_json::Value>,
// ) {
//     let agent_payload = crate::state::AgentPayload {
//         processes: payload["processes"].clone(),
//         system_stats: payload["system_stats"].clone(),
//         ip: addr.ip().to_string(),
//     };

//     let mut agents = state.agents.write().await;
//     agents.insert(agent_id, agent_payload);
// }
pub async fn push_metrics(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(agent_id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) {
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
    agent_entry.ip = addr.ip().to_string();

    println!("Metrics received from agent {}", agent_id);
}




#[derive(Deserialize)]
pub struct KillParams {
    pub agent_id: String,
    pub pid: u32,
}

// pub async fn kill_process(
//     State(state): State<AppState>,
//     Json(payload): Json<KillParams>,
// ) -> Json<bool> {

//     let agents = state.agents.read().await;

//     if let Some(_agent) = agents.get(&payload.agent_id) {
//         // ⚠️ ici il faut que ton agent supporte le kill réel
//         println!("Kill request for PID {} on agent {}", payload.pid, payload.agent_id);

//         // Pour l’instant simulation :
//         return Json(true);
//     }

//     Json(false)
// }

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

