use axum::{
    extract::{Query, State},
    response::Json,
};
use serde::Deserialize;
use crate::{agent_client::fetch_processes, state::AppState};

#[derive(Deserialize)]
pub struct Params {
    pub host: String,
}

pub async fn processes(
    Query(params): Query<Params>,
    State(state): State<AppState>,
) -> Json<String> {

    // Appel agent
    if let Ok(data) = fetch_processes(&params.host).await {
        *state.latest_data.write().await = Some(data);
    }

    // Retour cache
    let data = state
        .latest_data
        .read()
        .await
        .clone()
        .unwrap_or_else(|| "{}".to_string());

    Json(data)
}
