use axum::{
    extract::{Extension, State},
    Json,
};
use std::sync::Arc;
use crate::{
    errors::{AppError, AppResult},
    middlewares::auth::Claims,
    models::{Utilisateur, UtilisateurResponse},
    AppState,
};

// GET /api/utilisateurs/me
pub async fn get_me(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<UtilisateurResponse>> {
    let utilisateur = sqlx::query_as!(
        Utilisateur,
        "SELECT * FROM utilisateurs WHERE id = $1",
        uuid::Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Utilisateur introuvable".to_string()))?;

    Ok(Json(utilisateur.into()))
}

// PUT /api/utilisateurs/me
pub async fn update_me(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<UpdateMePayload>,
) -> AppResult<Json<UtilisateurResponse>> {
    let utilisateur = sqlx::query_as!(
        Utilisateur,
        r#"
        UPDATE utilisateurs
        SET nom        = COALESCE($1, nom),
            prenom     = COALESCE($2, prenom),
            profession = COALESCE($3, profession),
            email = COALESCE($4, email)
        WHERE id = $5
        RETURNING *
        "#,
        payload.nom,
        payload.prenom,
        payload.profession,
        payload.email,
        uuid::Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(utilisateur.into()))
}

#[derive(serde::Deserialize)]
pub struct UpdateMePayload {
    pub nom:        Option<String>,
    pub prenom:     Option<String>,
    pub profession: Option<String>,
    pub email: String,
}