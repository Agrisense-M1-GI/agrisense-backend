use axum::{
    extract::{Extension, Path, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    middlewares::auth::Claims,
    models::{Image, ImagePayload},
    AppState,
};

// POST /api/images
// Appelé par le capteur pour enregistrer une nouvelle image capturée
pub async fn recevoir_image(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ImagePayload>,
) -> AppResult<Json<Image>> {

    // Vérifie que le capteur existe
    let capteur_existe = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM noeuds_capteurs WHERE id = $1",
        payload.noeud_capteur_id
    )
    .fetch_one(&state.db)
    .await?;

    if capteur_existe.unwrap_or(0) == 0 {
        return Err(AppError::NotFound("Capteur introuvable".to_string()));
    }

    let image = sqlx::query_as!(
        Image,
        r#"
        INSERT INTO images (noeud_capteur_id, code, longueur, largeur, chemin_stockage, taille_octets, format)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, noeud_capteur_id, code, longueur, largeur,
                  chemin_stockage, taille_octets, format, date_capture, est_traitee, created_at
        "#,
        payload.noeud_capteur_id,
        payload.code,
        payload.longueur,
        payload.largeur,
        payload.chemin_stockage,
        payload.taille_octets,
        payload.format,
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(image))
}

// GET /api/images/:capteur_id
// Historique des images d'un capteur
pub async fn get_historique_images(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(capteur_id): Path<Uuid>,
) -> AppResult<Json<Vec<Image>>> {

    let images = sqlx::query_as!(
        Image,
        r#"
        SELECT id, noeud_capteur_id, code, longueur, largeur,
               chemin_stockage, taille_octets, format, date_capture, est_traitee, created_at
        FROM images
        WHERE noeud_capteur_id = $1
        ORDER BY date_capture DESC
        "#,
        capteur_id,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(images))
}

// GET /api/images/:capteur_id/non-traitees
// Images pas encore analysées par le modèle IA
pub async fn get_images_non_traitees(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(capteur_id): Path<Uuid>,
) -> AppResult<Json<Vec<Image>>> {

    let images = sqlx::query_as!(
        Image,
        r#"
        SELECT id, noeud_capteur_id, code, longueur, largeur,
               chemin_stockage, taille_octets, format, date_capture, est_traitee, created_at
        FROM images
        WHERE noeud_capteur_id = $1 AND est_traitee = FALSE
        ORDER BY date_capture ASC
        "#,
        capteur_id,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(images))
}

// GET /api/images/detail/:id
// Détail d'une image spécifique
pub async fn get_image(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Image>> {

    let image = sqlx::query_as!(
        Image,
        r#"
        SELECT id, noeud_capteur_id, code, longueur, largeur,
               chemin_stockage, taille_octets, format, date_capture, est_traitee, created_at
        FROM images WHERE id = $1
        "#,
        id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Image introuvable".to_string()))?;

    Ok(Json(image))
}