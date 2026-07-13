use axum::{
    extract::{Path, State, Extension},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    middlewares::auth::Claims,
    models::{AnalyseJournaliere, CallbackImagePayload, CallbackMetriquesPayload, Recommandation},
    AppState,
};

// ─────────────────────────────────────────────
// WEBHOOKS — appelés par le service Python
// ─────────────────────────────────────────────

// POST /api/ia/callback/image
// Le service Python envoie ici le résultat d'analyse d'image
pub async fn callback_image(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CallbackImagePayload>,
) -> AppResult<Json<Recommandation>> {

    // Cherche notre image via le model_image_id retourné par le service Python
    let image = sqlx::query!(
        r#"
        SELECT id FROM images
        WHERE model_image_id = $1
        "#,
        payload.image_id   // ← c'est l'UUID du service Python
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| {
        tracing::warn!(
            "⚠️ Callback reçu pour model_image_id {} mais aucune image trouvée en base",
            payload.image_id
        );
        AppError::NotFound(format!(
            "Aucune image trouvée pour model_image_id {}",
            payload.image_id
        ))
    })?;

    // Enregistre la recommandation avec notre UUID
    let recommandation = sqlx::query_as!(
        Recommandation,
        r#"
        INSERT INTO recommandations
            (image_id, sensor_id, etat, actions_texte, priorite)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING
            id, image_id, sensor_id, etat, actions_texte,
            actions, priorite, est_lue, created_at
        "#,
        image.id,
        payload.sensor_id,
        payload.etat,
        payload.actions,
        payload.priorite,
    )
    .fetch_one(&state.db)
    .await?;

    // Marque l'image comme traitée
    sqlx::query!(
        "UPDATE images SET est_traitee = TRUE WHERE id = $1",
        image.id
    )
    .execute(&state.db)
    .await?;

    tracing::info!(
        "✅ Recommandation enregistrée — model_image_id: {} → notre image: {} — état: {} priorité: {}",
        payload.image_id, image.id, payload.etat, payload.priorite
    );

    Ok(Json(recommandation))
}

// POST /api/ia/callback/metriques
// Le service Python envoie ici le résultat d'analyse journalière
pub async fn callback_metriques(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CallbackMetriquesPayload>,
) -> AppResult<Json<AnalyseJournaliere>> {

    let aujourd_hui = chrono::Utc::now().date_naive();

    // Upsert — si une analyse existe déjà aujourd'hui, on la met à jour
    let analyse = sqlx::query_as!(
        AnalyseJournaliere,
        r#"
        INSERT INTO analyses_journalieres (date_jour, etat, contenu, priorite)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (date_jour)
        DO UPDATE SET
            etat     = EXCLUDED.etat,
            contenu  = EXCLUDED.contenu,
            priorite = EXCLUDED.priorite
        RETURNING *
        "#,
        aujourd_hui,
        payload.etat,
        payload.actions,
        payload.priorite,
    )
    .fetch_one(&state.db)
    .await?;

    tracing::info!(
        "✅ Analyse journalière enregistrée — {} état: {} priorité: {}",
        aujourd_hui, payload.etat, payload.priorite
    );

    Ok(Json(analyse))
}

// GET /api/ia/metrics-source
// Le service Python vient lire ici les métriques du jour à analyser
pub async fn metrics_source(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<serde_json::Value>>> {

    // Récupère les mesures des dernières 24h par tranche horaire
    let mesures = sqlx::query!(
        r#"
        SELECT
            h_air.valeur        AS humidite_air,
            h_sol.valeur        AS humidite_sol,
            t.valeur            AS temperature,
            h_air.date_mesure   AS horodatage
        FROM donnees_humidite h_air
        LEFT JOIN donnees_humidite h_sol
            ON h_sol.type_humidite = 'sol'
            AND h_sol.date_mesure BETWEEN h_air.date_mesure - INTERVAL '2 minutes'
                                      AND h_air.date_mesure + INTERVAL '2 minutes'
        LEFT JOIN donnees_temperature t
            ON t.date_mesure BETWEEN h_air.date_mesure - INTERVAL '2 minutes'
                                 AND h_air.date_mesure + INTERVAL '2 minutes'
        WHERE h_air.type_humidite = 'air'
          AND h_air.date_mesure  >= NOW() - INTERVAL '24 hours'
        ORDER BY h_air.date_mesure DESC
        LIMIT 30
        "#
    )
    .fetch_all(&state.db)
    .await?;

    let metriques: Vec<serde_json::Value> = mesures
        .into_iter()
        .map(|row| serde_json::json!({
            "humidity":     row.humidite_air,
            "soil_humidity": row.humidite_sol,   // ← humidité sol
            "air_temp":     row.temperature,      // ← température
            "timestamp":    row.horodatage.to_rfc3339(),
        }))
        .collect();

    Ok(Json(metriques))
}

// ─────────────────────────────────────────────
// RECOMMANDATIONS — lecture par le frontend
// ─────────────────────────────────────────────

// GET /api/recommandations
pub async fn get_recommandations(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
) -> AppResult<Json<Vec<Recommandation>>> {

    let recommandations = sqlx::query_as!(
        Recommandation,
        r#"
        SELECT id, image_id, sensor_id, etat, actions_texte,
               actions, priorite, est_lue, created_at
        FROM recommandations
        ORDER BY created_at DESC
        LIMIT 50
        "#
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(recommandations))
}

// GET /api/recommandations/non-lues
pub async fn get_recommandations_non_lues(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
) -> AppResult<Json<Vec<Recommandation>>> {

    let recommandations = sqlx::query_as!(
        Recommandation,
        r#"
        SELECT id, image_id, sensor_id, etat, actions_texte,
               actions, priorite, est_lue, created_at
        FROM recommandations
        WHERE est_lue = FALSE
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(recommandations))
}

// GET /api/recommandations/:id
pub async fn get_recommandation(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Recommandation>> {

    let recommandation = sqlx::query_as!(
        Recommandation,
        r#"
        SELECT id, image_id, sensor_id, etat, actions_texte,
               actions, priorite, est_lue, created_at
        FROM recommandations WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Recommandation introuvable".to_string()))?;

    Ok(Json(recommandation))
}

// PATCH /api/recommandations/:id/lue
pub async fn marquer_lue(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Recommandation>> {

    let recommandation = sqlx::query_as!(
        Recommandation,
        r#"
        UPDATE recommandations SET est_lue = TRUE WHERE id = $1
        RETURNING id, image_id, sensor_id, etat, actions_texte,
                  actions, priorite, est_lue, created_at
        "#,
        id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Recommandation introuvable".to_string()))?;

    Ok(Json(recommandation))
}

// PATCH /api/recommandations/tout-lire
pub async fn tout_marquer_lue(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
) -> AppResult<Json<serde_json::Value>> {

    let result = sqlx::query!(
        "UPDATE recommandations SET est_lue = TRUE WHERE est_lue = FALSE"
    )
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({
        "message": "Recommandations marquées comme lues",
        "nombre":  result.rows_affected()
    })))
}

// DELETE /api/recommandations/:id
pub async fn delete_recommandation(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {

    let result = sqlx::query!(
        "DELETE FROM recommandations WHERE id = $1", id
    )
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Recommandation introuvable".to_string()));
    }

    Ok(Json(serde_json::json!({ "message": "Recommandation supprimée" })))
}

// ─────────────────────────────────────────────
// ANALYSES JOURNALIERES — lecture par le frontend
// ─────────────────────────────────────────────

// GET /api/analyses
pub async fn get_analyses(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
) -> AppResult<Json<Vec<AnalyseJournaliere>>> {

    let analyses = sqlx::query_as!(
        AnalyseJournaliere,
        "SELECT * FROM analyses_journalieres ORDER BY date_jour DESC LIMIT 30"
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(analyses))
}

// GET /api/analyses/aujourd-hui
pub async fn get_analyse_aujourd_hui(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
) -> AppResult<Json<AnalyseJournaliere>> {

    let aujourd_hui = chrono::Utc::now().date_naive();

    let analyse = sqlx::query_as!(
        AnalyseJournaliere,
        "SELECT * FROM analyses_journalieres WHERE date_jour = $1",
        aujourd_hui
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound(
        "Aucune analyse disponible pour aujourd'hui".to_string()
    ))?;

    Ok(Json(analyse))
}

// GET /api/analyses/:id
pub async fn get_analyse(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<AnalyseJournaliere>> {

    let analyse = sqlx::query_as!(
        AnalyseJournaliere,
        "SELECT * FROM analyses_journalieres WHERE id = $1",
        id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Analyse introuvable".to_string()))?;

    Ok(Json(analyse))
}

// PATCH /api/analyses/:id/lue
pub async fn marquer_analyse_lue(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<AnalyseJournaliere>> {

    let analyse = sqlx::query_as!(
        AnalyseJournaliere,
        r#"
        UPDATE analyses_journalieres SET est_lue = TRUE WHERE id = $1
        RETURNING *
        "#,
        id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Analyse introuvable".to_string()))?;

    Ok(Json(analyse))
}