use axum::{
    extract::{Extension, Path, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;
use crate::{
    errors::{AppError, AppResult},
    middlewares::auth::Claims,
    models::{Champ, ChampPayload},
    AppState,
};

// GET /api/champs  — tous les champs de l'utilisateur connecté
pub async fn get_champs(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<Vec<Champ>>> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;

    let champs = sqlx::query_as!(
        Champ,
        "SELECT * FROM champs WHERE utilisateur_id = $1 ORDER BY created_at DESC",
        user_id
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(champs))
}

// GET /api/champs/:id
pub async fn get_champ(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Champ>> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;

    let champ = sqlx::query_as!(
        Champ,
        "SELECT * FROM champs WHERE id = $1 AND utilisateur_id = $2",
        id, user_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Champ introuvable".to_string()))?;

    Ok(Json(champ))
}

// POST /api/champs
pub async fn create_champ(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<ChampPayload>,
) -> AppResult<Json<Champ>> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;

    let champ = sqlx::query_as!(
        Champ,
        r#"
        INSERT INTO champs (utilisateur_id, nom, description, localisation, superficie, latitude, longitude)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#,
        user_id,
        payload.nom,
        payload.description,
        payload.localisation,
        payload.superficie,
        payload.latitude,
        payload.longitude,
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(champ))
}

// PUT /api/champs/:id
pub async fn update_champ(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ChampPayload>,
) -> AppResult<Json<Champ>> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;

    let champ = sqlx::query_as!(
        Champ,
        r#"
        UPDATE champs
        SET nom          = COALESCE($1, nom),
            description  = COALESCE($2, description),
            localisation = COALESCE($3, localisation),
            superficie   = COALESCE($4, superficie),
            latitude     = COALESCE($5, latitude),
            longitude    = COALESCE($6, longitude)
        WHERE id = $7 AND utilisateur_id = $8
        RETURNING *
        "#,
        payload.nom,
        payload.description,
        payload.localisation,
        payload.superficie,
        payload.latitude,
        payload.longitude,
        id,
        user_id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Champ introuvable".to_string()))?;

    Ok(Json(champ))
}

// DELETE /api/champs/:id
pub async fn delete_champ(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;

    let result = sqlx::query!(
        "DELETE FROM champs WHERE id = $1 AND utilisateur_id = $2",
        id, user_id
    )
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Champ introuvable".to_string()));
    }

    Ok(Json(serde_json::json!({ "message": "Champ supprimé" })))
}