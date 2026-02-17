use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct AgentPayload {
    pub processes: serde_json::Value,
    pub system_stats: serde_json::Value,
    pub ip: String,
}

#[derive(Clone, Default)]
pub struct AppState {
    pub agents: Arc<RwLock<HashMap<String, AgentPayload>>>,
    pub commands: Arc<RwLock<HashMap<String, Vec<serde_json::Value>>>>,
}
