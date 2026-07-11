use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    middlewares::auth::Claims,
    models::{DonneeTemperature, DonneeTemperaturePayload, PeriodeQuery},
    AppState,
};

// POST /api/temperature
// Appelé par le capteur pour envoyer une mesure
pub async fn recevoir_mesure(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<DonneeTemperaturePayload>,
) -> AppResult<Json<DonneeTemperature>> {

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

    let mesure = sqlx::query_as!(
        DonneeTemperature,
        r#"
        INSERT INTO donnees_temperature (noeud_capteur_id, valeur)
        VALUES ($1, $2)
        RETURNING id, noeud_capteur_id, valeur, date_mesure
        "#,
        payload.noeud_capteur_id,
        payload.valeur,
    )
    .fetch_one(&state.db)
    .await?;

    verifier_seuil_temperature(&state, payload.noeud_capteur_id, payload.valeur).await?;

    tracing::info!(
        "🌡️ Température reçue : {:.1}°C — capteur {}",
        payload.valeur,
        payload.noeud_capteur_id
    );

    Ok(Json(mesure))
}

// GET /api/temperature/:capteur_id
// Historique des mesures avec filtre optionnel par période
pub async fn get_historique(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(capteur_id): Path<Uuid>,
    Query(periode): Query<PeriodeQuery>,
) -> AppResult<Json<Vec<DonneeTemperature>>> {

    let mesures = sqlx::query_as!(
        DonneeTemperature,
        r#"
        SELECT id, noeud_capteur_id, valeur, date_mesure
        FROM donnees_temperature
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

// GET /api/temperature/:capteur_id/derniere
// Retourne uniquement la dernière mesure
pub async fn get_derniere_mesure(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(capteur_id): Path<Uuid>,
) -> AppResult<Json<DonneeTemperature>> {

    let mesure = sqlx::query_as!(
        DonneeTemperature,
        r#"
        SELECT id, noeud_capteur_id, valeur, date_mesure
        FROM donnees_temperature
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

// Ajoute cette fonction
async fn verifier_seuil_temperature(
    state: &AppState,
    capteur_id: Uuid,
    valeur: f64,
) -> AppResult<()> {

    let seuils = sqlx::query!(
        "SELECT utilisateur_id, valeur_min, valeur_max FROM seuils_temperature"
    )
    .fetch_all(&state.db)
    .await?;

    for seuil in seuils {
        let message = if valeur < seuil.valeur_min {
            Some(format!(
                "🌡️ Température critique ({:.1}°C) sous le seuil minimum ({:.1}°C) — capteur {}",
                valeur, seuil.valeur_min, capteur_id
            ))
        } else if valeur > seuil.valeur_max {
            Some(format!(
                "🌡️ Température excessive ({:.1}°C) au-dessus du seuil maximum ({:.1}°C) — capteur {}",
                valeur, seuil.valeur_max, capteur_id
            ))
        } else {
            None
        };

        if let Some(msg) = message {
            sqlx::query!(
                r#"
                INSERT INTO notifications (utilisateur_id, type, message, source)
                VALUES ($1, 'alerte_critique', $2, 'temperature')
                "#,
                seuil.utilisateur_id,
                msg,
            )
            .execute(&state.db)
            .await?;

            tracing::warn!("🚨 {}", msg);
        }
    }

    Ok(())
}