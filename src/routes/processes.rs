// use axum::{
//     extract::{Query, State},
//     response::Json,
// };
// use serde::Deserialize;
// use crate::{agent_client::fetch_processes, state::AppState};

// #[derive(Deserialize)]
// pub struct Params {
//     pub host: String,
// }

// pub async fn processes(
//     Query(params): Query<Params>,
//     State(state): State<AppState>,
// ) -> Json<String> {

//     // Appel agent
//     if let Ok(data) = fetch_processes(&params.host).await {
//         *state.latest_data.write().await = Some(data);
//     }

//     // Retour cache
//     let data = state
//         .latest_data
//         .read()
//         .await
//         .clone()
//         .unwrap_or_else(|| "{}".to_string());

//     Json(data)
// }
use axum::{
    extract::{Query, State, Path, Json},
    routing::{get, post},
};
use serde::Deserialize;
use crate::state::AppState;



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
    Path(agent_id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<crate::state::AgentPayload>,
) {
    let mut agents = state.agents.write().await;
    agents.insert(agent_id, payload);
}
