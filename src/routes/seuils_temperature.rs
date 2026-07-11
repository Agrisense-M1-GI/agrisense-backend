use axum::{
    extract::{Extension, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    middlewares::auth::Claims,
    models::{SeuilTemperature, SeuilTemperaturePayload},
    AppState,
};

// GET /api/seuils/temperature
pub async fn get_seuil(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<SeuilTemperature>> {

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized)?;

    let seuil = sqlx::query_as!(
        SeuilTemperature,
        "SELECT * FROM seuils_temperature WHERE utilisateur_id = $1",
        user_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Aucun seuil de température configuré".to_string()))?;

    Ok(Json(seuil))
}

// POST /api/seuils/temperature — crée ou remplace le seuil
pub async fn upsert_seuil(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<SeuilTemperaturePayload>,
) -> AppResult<Json<SeuilTemperature>> {

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized)?;

    if payload.valeur_min >= payload.valeur_max {
        return Err(AppError::BadRequest(
            "valeur_min doit être inférieure à valeur_max".to_string()
        ));
    }

    let seuil = sqlx::query_as!(
        SeuilTemperature,
        r#"
        INSERT INTO seuils_temperature (utilisateur_id, valeur_min, valeur_max)
        VALUES ($1, $2, $3)
        ON CONFLICT (utilisateur_id)
        DO UPDATE SET
            valeur_min = EXCLUDED.valeur_min,
            valeur_max = EXCLUDED.valeur_max
        RETURNING *
        "#,
        user_id,
        payload.valeur_min,
        payload.valeur_max,
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(seuil))
}