use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;
use crate::{
    errors::{AppError, AppResult},
    models::{Culture, CulturePayload},
    AppState,
};

// GET /api/champs/:champ_id/cultures
pub async fn get_cultures(
    State(state): State<Arc<AppState>>,
    Path(champ_id): Path<Uuid>,
) -> AppResult<Json<Vec<Culture>>> {
    let cultures = sqlx::query_as!(
        Culture,
        r#"
        SELECT id, champ_id, nom, type_culture, stade_croissance,
               date_semence, date_recolte_prevue, notes, created_at, updated_at
        FROM cultures WHERE champ_id = $1 ORDER BY created_at DESC
        "#,
        champ_id
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(cultures))
}

// POST /api/champs/:champ_id/cultures
pub async fn create_culture(
    State(state): State<Arc<AppState>>,
    Path(champ_id): Path<Uuid>,
    Json(payload): Json<CulturePayload>,
) -> AppResult<Json<Culture>> {
    let culture = sqlx::query_as!(
        Culture,
        r#"
        INSERT INTO cultures (champ_id, nom, type_culture, stade_croissance, date_semence, date_recolte_prevue, notes)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#,
        champ_id,
        payload.nom,
        payload.type_culture,
        payload.stade_croissance,
        payload.date_semence,
        payload.date_recolte_prevue,
        payload.notes,
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(culture))
}

// PUT /api/champs/:champ_id/cultures/:id
pub async fn update_culture(
    State(state): State<Arc<AppState>>,
    Path((champ_id, id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<CulturePayload>,
) -> AppResult<Json<Culture>> {
    let culture = sqlx::query_as!(
        Culture,
        r#"
        UPDATE cultures
        SET nom                 = COALESCE($1, nom),
            type_culture        = COALESCE($2, type_culture),
            stade_croissance    = COALESCE($3, stade_croissance),
            date_semence        = COALESCE($4, date_semence),
            date_recolte_prevue = COALESCE($5, date_recolte_prevue),
            notes               = COALESCE($6, notes)
        WHERE id = $7 AND champ_id = $8
        RETURNING *
        "#,
        payload.nom,
        payload.type_culture,
        payload.stade_croissance,
        payload.date_semence,
        payload.date_recolte_prevue,
        payload.notes,
        id,
        champ_id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Culture introuvable".to_string()))?;

    Ok(Json(culture))
}

// DELETE /api/champs/:champ_id/cultures/:id
pub async fn delete_culture(
    State(state): State<Arc<AppState>>,
    Path((champ_id, id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    let result = sqlx::query!(
        "DELETE FROM cultures WHERE id = $1 AND champ_id = $2",
        id, champ_id
    )
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Culture introuvable".to_string()));
    }

    Ok(Json(serde_json::json!({ "message": "Culture supprimée" })))
}