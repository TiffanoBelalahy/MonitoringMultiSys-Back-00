use axum::{extract::State, response::Json};
use crate::state::AppState;
use axum::extract::Path;
use serde::Deserialize;
use sqlx::Row;

pub async fn list_agents(
    State(state): State<AppState>,
) -> Json<Vec<serde_json::Value>> {

    let rows = sqlx::query(
        r#"
        SELECT a.id, a.name, a.company_id,
               c.name as company_name
        FROM agents a
        LEFT JOIN companies c ON a.company_id = c.id
        "#
    )
    .fetch_all(&state.db)
    .await
    .unwrap();

    let result: Vec<_> = rows.into_iter().map(|row| {
        serde_json::json!({
            "id": row.get::<String,_>("id"),
            "name": row.get::<String,_>("name"),
            "company_id": row.get::<Option<uuid::Uuid>,_>("company_id")
                .map(|id| id.to_string()),
            "company_name": row.get::<Option<String>,_>("company_name"),
        })
    }).collect();

    Json(result)
}


#[derive(Deserialize)]
pub struct RenameAgentInput {
    pub name: String,
}

pub async fn rename_agent(
    Path(agent_id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<RenameAgentInput>,
) -> Json<String> {

    sqlx::query(
        "UPDATE agents SET name = $1 WHERE id = $2"
    )
    .bind(payload.name)
    .bind(&agent_id)
    .execute(&state.db)
    .await
    .unwrap();

    Json("Agent renamed".into())
}


#[derive(Deserialize)]
pub struct CreateCompanyInput {
    pub name: String,
}

pub async fn create_company(
    State(state): State<AppState>,
    Json(payload): Json<CreateCompanyInput>,
) -> Json<String> {

    sqlx::query(
        "INSERT INTO companies (name) VALUES ($1)"
    )
    .bind(payload.name)
    .execute(&state.db)
    .await
    .unwrap();

    Json("Company created".into())
}

pub async fn list_companies(
    State(state): State<AppState>,
) -> Json<Vec<serde_json::Value>> {

    let rows = sqlx::query(
        "SELECT id, name FROM companies"
    )
    .fetch_all(&state.db)
    .await
    .unwrap();

    let result: Vec<_> = rows.into_iter().map(|row| {
        serde_json::json!({
            "id": row.get::<uuid::Uuid,_>("id").to_string(),
            "name": row.get::<String,_>("name"),
        })
    }).collect();

    Json(result)
}


#[derive(Deserialize)]
pub struct CreateAgentInput {
    pub name: String,
    pub company_id: uuid::Uuid,
}

pub async fn create_agent(
    State(state): State<AppState>,
    Json(payload): Json<CreateAgentInput>,
) -> Json<String> {

    let id = uuid::Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO agents (id, name, company_id) VALUES ($1,$2,$3)"
    )
    .bind(&id)
    .bind(&payload.name)
    .bind(payload.company_id)
    .execute(&state.db)
    .await
    .unwrap();

    Json("Agent created".into())
}

#[derive(Deserialize)]
pub struct AssignAgentInput {
    pub company_id: uuid::Uuid,
}

pub async fn assign_agent_to_company(
    Path(agent_id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<AssignAgentInput>,
) -> Json<String> {

    // 🔍 Vérifier si l'agent a déjà une company
    let row = sqlx::query(
        "SELECT company_id FROM agents WHERE id = $1"
    )
    .bind(&agent_id)
    .fetch_one(&state.db)
    .await;

    // ⚠️ Si agent n'existe pas
    if row.is_err() {
        return Json("Agent not found".into());
    }

    let row = row.unwrap();

    let current_company: Option<uuid::Uuid> = row.get("company_id");

    // ❌ déjà assigné
    if current_company.is_some() {
        return Json("Agent already assigned".into());
    }

    // ✅ assignation
    sqlx::query(
        "UPDATE agents SET company_id = $1 WHERE id = $2"
    )
    .bind(payload.company_id)
    .bind(&agent_id)
    .execute(&state.db)
    .await
    .unwrap();

    Json("Agent assigned".into())
}