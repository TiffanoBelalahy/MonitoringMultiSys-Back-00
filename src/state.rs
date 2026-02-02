use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct AppState {
    pub latest_data: Arc<RwLock<Option<String>>>,
}
