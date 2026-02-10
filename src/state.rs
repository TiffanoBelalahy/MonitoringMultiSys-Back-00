use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct AgentPayload {
    pub processes: serde_json::Value,
    pub system_stats: serde_json::Value,
}

#[derive(Clone, Default)]
pub struct AppState {
    pub agents: Arc<RwLock<HashMap<String, AgentPayload>>>,
}
