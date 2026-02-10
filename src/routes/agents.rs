use axum::{extract::State, response::Json};
use crate::state::AppState;

pub async fn list_agents(
    State(state): State<AppState>,
) -> Json<Vec<serde_json::Value>> {
    let agents = state.agents.read().await;

    let result = agents
        .keys()
        .map(|id| {
            serde_json::json!({
                "id": id,
                "online": true
            })
        })
        .collect();

    Json(result)
}
