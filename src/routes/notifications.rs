use axum::{
    extract::{Extension, Path, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    middlewares::auth::Claims,
    models::Notification,
    AppState,
};

// GET /api/notifications
// Récupère l'historique des notifications de l'utilisateur
pub async fn get_notifications(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<Vec<Notification>>> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;

    let notifications = sqlx::query_as!(
        Notification,
        r#"
        SELECT * FROM notifications 
        WHERE utilisateur_id = $1 
        ORDER BY date DESC
        "#,
        user_id
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(notifications))
}

// PATCH /api/notifications/:id/lue
// Marque une notification comme lue
pub async fn mark_as_lue(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(notification_id): Path<Uuid>,
) -> AppResult<Json<Notification>> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;

    let notification = sqlx::query_as!(
        Notification,
        r#"
        UPDATE notifications
        SET statut = 'lue'
        WHERE id = $1 AND utilisateur_id = $2
        RETURNING *
        "#,
        notification_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Notification introuvable".to_string()))?;

    Ok(Json(notification))
}
