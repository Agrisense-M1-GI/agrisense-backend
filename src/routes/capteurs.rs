use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;
use crate::{
    errors::{AppError, AppResult},
    models::{NoeudCapteur, NoeudCapteurPayload, UpdateEtatCapteur},
    AppState,
};

// GET /api/capteurs
pub async fn get_capteurs(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<NoeudCapteur>>> {
    let capteurs = sqlx::query_as!(
        NoeudCapteur,
        "SELECT id, nom, type_capteur, longitude, latitude, batterie,
               etat, surface_couverte, derniere_connexion, created_at, updated_at 
        FROM noeuds_capteurs ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(capteurs))
}

// GET /api/capteurs/:id
pub async fn get_capteur(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<NoeudCapteur>> {
    let capteur = sqlx::query_as!(
        NoeudCapteur,
        r#"SELECT id, nom, type_capteur, longitude, latitude, batterie,
               etat, surface_couverte, derniere_connexion, created_at, updated_at 
        FROM noeuds_capteurs WHERE id = $1"#,
        id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Capteur introuvable".to_string()))?;

    Ok(Json(capteur))
}

// POST /api/capteurs
pub async fn create_capteur(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<NoeudCapteurPayload>,
) -> AppResult<Json<NoeudCapteur>> {
    let capteur = sqlx::query_as!(
        NoeudCapteur,
        r#"
        INSERT INTO noeuds_capteurs (nom, type_capteur, longitude, latitude, batterie, surface_couverte)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
        payload.nom,
        payload.type_capteur,
        payload.longitude,
        payload.latitude,
        payload.batterie,
        payload.surface_couverte,
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(capteur))
}

// PATCH /api/capteurs/:id/etat  — mise à jour état/batterie par le capteur lui-même
pub async fn update_etat_capteur(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateEtatCapteur>,
) -> AppResult<Json<NoeudCapteur>> {
    let capteur = sqlx::query_as!(
        NoeudCapteur,
        r#"
        UPDATE noeuds_capteurs
        SET etat               = $1,
            batterie           = COALESCE($2, batterie),
            derniere_connexion = COALESCE($3, NOW())
        WHERE id = $4
        RETURNING *
        "#,
        payload.etat,
        payload.batterie,
        payload.derniere_connexion,
        id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Capteur introuvable".to_string()))?;

    Ok(Json(capteur))
}

// PUT /api/capteurs/:id — mise à jour complète du capteur
pub async fn update_capteur(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<NoeudCapteurPayload>,
) -> AppResult<Json<NoeudCapteur>> {
    let capteur = sqlx::query_as!(
        NoeudCapteur,
        r#"
        UPDATE noeuds_capteurs
        SET nom                = $1,
            type_capteur       = $2,
            longitude          = $3,
            latitude           = $4,
            batterie           = $5,
            surface_couverte   = $6
        WHERE id = $7
        RETURNING *
        "#,
        payload.nom,
        payload.type_capteur,
        payload.longitude,
        payload.latitude,
        payload.batterie,
        payload.surface_couverte,
        id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Capteur introuvable".to_string()))?;

    tracing::info!("✏️  Capteur {} modifié", id);

    Ok(Json(capteur))
}

// DELETE /api/capteurs/:id — supprime un capteur
pub async fn delete_capteur(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let result = sqlx::query!(
        "DELETE FROM noeuds_capteurs WHERE id = $1",
        id,
    )
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Capteur introuvable".to_string()));
    }

    tracing::info!("🗑️  Capteur {} supprimé", id);

    Ok(Json(serde_json::json!({
        "message": "Capteur supprimé avec succès",
        "id": id.to_string()
    })))
}