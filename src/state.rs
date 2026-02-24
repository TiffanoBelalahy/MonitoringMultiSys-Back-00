use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use sqlx::PgPool;
// Payload envoyé par les agents pour mettre à jour leurs données
#[derive(Clone, Serialize, Deserialize)]
pub struct AgentPayload {
    pub processes: serde_json::Value,
    pub system_stats: serde_json::Value,
    pub ip: String,
}
// State de l'application pour stocker les agents et les commandes
// #[derive(Clone, Default)]
// pub struct AppState {
//     pub agents: Arc<RwLock<HashMap<String, AgentPayload>>>,
//     pub commands: Arc<RwLock<HashMap<String, Vec<serde_json::Value>>>>,
// }

// Implémentation de AppState avec une nouvelle fonction pour initialiser la connexion à la base de données
#[derive(Clone)]
pub struct AppState {
    pub agents: Arc<RwLock<HashMap<String, AgentPayload>>>,
    pub commands: Arc<RwLock<HashMap<String, Vec<serde_json::Value>>>>,
    pub db: PgPool,
}