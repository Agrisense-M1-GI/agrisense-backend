use axum::{extract::State, Json};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use std::sync::Arc;

use crate::{
    errors::{AppError, AppResult},
    middlewares::auth::generate_token,
    models::{AuthResponse, LoginPayload, RegisterPayload, Utilisateur},
    AppState,
};

// POST /api/auth/register
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterPayload>,
) -> AppResult<Json<AuthResponse>> {

    // Vérifie que l'email n'est pas déjà utilisé
    let exists = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM utilisateurs WHERE email = $1",
        payload.email
    )
    .fetch_one(&state.db)
    .await?;

    if exists.unwrap_or(0) > 0 {
        return Err(AppError::BadRequest("Email déjà utilisé".to_string()));
    }

    // Hashe le mot de passe
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("Erreur hashage : {}", e)))?
        .to_string();

    // Insère en base
    let utilisateur = sqlx::query_as!(
        Utilisateur,
        r#"
        INSERT INTO utilisateurs (email, password_hash, nom, prenom, profession)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
        payload.email,
        password_hash,
        payload.nom,
        payload.prenom,
        payload.profession,
    )
    .fetch_one(&state.db)
    .await?;

    // Génère le token JWT
    let token = generate_token(&utilisateur.id.to_string(), &state.config.jwt_secret)?;

    Ok(Json(AuthResponse {
        token,
        utilisateur: utilisateur.into(),
    }))
}

// POST /api/auth/login
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginPayload>,
) -> AppResult<Json<AuthResponse>> {

    // Cherche l'utilisateur
    let utilisateur = sqlx::query_as!(
        Utilisateur,
        "SELECT * FROM utilisateurs WHERE email = $1",
        payload.email
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::BadRequest("Email ou mot de passe incorrect".to_string()))?;

    // Vérifie le mot de passe
    let parsed_hash = PasswordHash::new(&utilisateur.password_hash)
        .map_err(|e| AppError::Internal(format!("Erreur hash : {}", e)))?;

    Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::BadRequest("Email ou mot de passe incorrect".to_string()))?;

    // Génère le token
    let token = generate_token(&utilisateur.id.to_string(), &state.config.jwt_secret)?;

    Ok(Json(AuthResponse {
        token,
        utilisateur: utilisateur.into(),
    }))
}