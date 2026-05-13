use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Non authentifié")]
    Unauthorized,

    #[error("Accès interdit")]
    Forbidden,

    #[error("Ressource introuvable : {0}")]
    NotFound(String),

    #[error("Données invalides : {0}")]
    BadRequest(String),

    #[error("Erreur base de données : {0}")]
    Database(#[from] sqlx::Error),

    #[error("Erreur interne : {0}")]
    Internal(String),
}

// Convertit automatiquement AppError en réponse HTTP JSON
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (_status, message) = match &self {
            AppError::Unauthorized      => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::Forbidden         => (StatusCode::FORBIDDEN, self.to_string()),
            AppError::NotFound(msg)     => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg)   => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Database(e)       => {
                tracing::error!("Erreur DB : {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données".to_string())
            },
            AppError::Internal(msg)     => {
                tracing::error!("Erreur interne : {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg.clone())
            },
        };

        Json(json!({ "error": message })).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;