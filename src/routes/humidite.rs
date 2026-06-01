use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    middlewares::auth::Claims,
    models::{DonneeHumidite, DonneeHumiditePayload, PeriodeQuery},
    AppState,
};

// POST /api/humidite
// Appelé par le capteur pour envoyer une mesure
pub async fn recevoir_mesure(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<DonneeHumiditePayload>,
) -> AppResult<Json<DonneeHumidite>> {

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

    // Enregistre la mesure
    let mesure = sqlx::query_as!(
        DonneeHumidite,
        r#"
        INSERT INTO donnees_humidite (noeud_capteur_id, valeur)
        VALUES ($1, $2)
        RETURNING id, noeud_capteur_id, valeur, date_mesure
        "#,
        payload.noeud_capteur_id,
        payload.valeur,
    )
    .fetch_one(&state.db)
    .await?;

    // Vérifie si le seuil est franchi et crée une notification si besoin
    verifier_seuil(&state, payload.noeud_capteur_id, payload.valeur).await?;

    Ok(Json(mesure))
}

// GET /api/humidite/:capteur_id
// Historique des mesures d'un capteur avec filtre optionnel par période
pub async fn get_historique(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(capteur_id): Path<Uuid>,
    Query(periode): Query<PeriodeQuery>,
) -> AppResult<Json<Vec<DonneeHumidite>>> {

    let mesures = sqlx::query_as!(
        DonneeHumidite,
        r#"
        SELECT id, noeud_capteur_id, valeur, date_mesure
        FROM donnees_humidite
        WHERE noeud_capteur_id = $1
          AND ($2::timestamptz IS NULL OR date_mesure >= $2)
          AND ($3::timestamptz IS NULL OR date_mesure <= $3)
        ORDER BY date_mesure DESC
        LIMIT 500
        "#,
        capteur_id,
        periode.debut,
        periode.fin,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(mesures))
}

// GET /api/humidite/:capteur_id/derniere
// Retourne uniquement la dernière mesure d'un capteur
pub async fn get_derniere_mesure(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(capteur_id): Path<Uuid>,
) -> AppResult<Json<DonneeHumidite>> {

    let mesure = sqlx::query_as!(
        DonneeHumidite,
        r#"
        SELECT id, noeud_capteur_id, valeur, date_mesure
        FROM donnees_humidite
        WHERE noeud_capteur_id = $1
        ORDER BY date_mesure DESC
        LIMIT 1
        "#,
        capteur_id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Aucune mesure disponible".to_string()))?;

    Ok(Json(mesure))
}

// Vérifie si la valeur dépasse un seuil configuré et crée une notification
async fn verifier_seuil(
    state: &AppState,
    capteur_id: Uuid,
    valeur: f64,
) -> AppResult<()> {

    // Récupère tous les seuils configurés
    let seuils = sqlx::query!(
        "SELECT utilisateur_id, valeur_min, valeur_max FROM seuils_humidite"
    )
    .fetch_all(&state.db)
    .await?;

    for seuil in seuils {
        let (message, source) = if valeur < seuil.valeur_min {
            (
                format!(
                    "⚠️ Humidité critique ({:.1}%) sous le seuil minimum ({:.1}%) — capteur {}",
                    valeur, seuil.valeur_min, capteur_id
                ),
                "humidite_basse",
            )
        } else if valeur > seuil.valeur_max {
            (
                format!(
                    "⚠️ Humidité excessive ({:.1}%) au-dessus du seuil maximum ({:.1}%) — capteur {}",
                    valeur, seuil.valeur_max, capteur_id
                ),
                "humidite_haute",
            )
        } else {
            continue; // Dans les limites, pas de notification
        };

        sqlx::query!(
            r#"
            INSERT INTO notifications (utilisateur_id, type, message, source)
            VALUES ($1, 'alerte_critique', $2, $3)
            "#,
            seuil.utilisateur_id,
            message,
            source,
        )
        .execute(&state.db)
        .await?;

        tracing::warn!("🚨 Alerte humidité : {}", message);
    }

    Ok(())
}