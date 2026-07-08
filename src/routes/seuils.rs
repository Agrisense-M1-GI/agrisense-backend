use axum::{
    extract::{Extension, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;
use crate::{
    errors::{AppError, AppResult},
    middlewares::auth::Claims,
    models::{SeuilHumidite, SeuilHumiditePayload},
    AppState,
};

// GET /api/seuils
pub async fn get_seuil(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<Vec<SeuilHumidite>>> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;

    let seuils = sqlx::query_as!(
        SeuilHumidite,
        "SELECT * FROM seuils_humidite WHERE utilisateur_id = $1",
        user_id
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(seuils))
}

// POST /api/seuils  — crée ou remplace le seuil
pub async fn upsert_seuil(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<SeuilHumiditePayload>,
) -> AppResult<Json<SeuilHumidite>> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;

    if payload.valeur_min >= payload.valeur_max {
        return Err(AppError::BadRequest(
            "valeur_min doit être inférieure à valeur_max".to_string()
        ));
    }

    if payload.type_humidite != "sol" && payload.type_humidite != "air" {
        return Err(AppError::BadRequest(
            "type_humidite doit être 'sol' ou 'air'".to_string()
        ));
    }

    let seuil = sqlx::query_as!(
        SeuilHumidite,
        r#"
        INSERT INTO seuils_humidite (utilisateur_id, valeur_min, valeur_max, irrigation_auto, type_humidite)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (utilisateur_id, type_humidite)
        DO UPDATE SET
            valeur_min      = EXCLUDED.valeur_min,
            valeur_max      = EXCLUDED.valeur_max,
            irrigation_auto = EXCLUDED.irrigation_auto
        RETURNING *
        "#,
        user_id,
        payload.valeur_min,
        payload.valeur_max,
        payload.irrigation_auto.unwrap_or(false),
        payload.type_humidite
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(seuil))
}