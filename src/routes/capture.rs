use axum::{
    extract::{Extension, Path, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    middlewares::auth::Claims,
    models::{DemandeCaptureDb, DemandeCapture, DemandeCapturePayload},
    AppState,
};

fn to_response(d: DemandeCaptureDb, base_url: &str) -> DemandeCapture {
    let image_url = d.image_id.map(|id| {
        // L'URL sera construite depuis le chemin stocké en base
        format!("{}/fichiers/{}", base_url, id)
    });

    DemandeCapture {
        id:         d.id,
        node_id:    d.node_id,
        statut:     d.statut,
        image_url,
        created_at: d.created_at,
        updated_at: d.updated_at,
    }
}

// POST /api/capturer
// Le frontend demande une capture
pub async fn demander_capture(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<DemandeCapturePayload>,
) -> AppResult<Json<DemandeCapture>> {

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized)?;

    // Vérifie qu'il n'y a pas déjà une demande en cours pour ce nœud
    let en_cours = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM demandes_capture
        WHERE node_id = $1
          AND statut IN ('en_attente', 'ack_recu')
        "#,
        payload.node_id
    )
    .fetch_one(&state.db)
    .await?;

    if en_cours.unwrap_or(0) > 0 {
        return Err(AppError::BadRequest(
            "Une capture est déjà en cours pour ce nœud".to_string()
        ));
    }

    // Crée la demande en base
    let demande = sqlx::query_as!(
        DemandeCaptureDb,
        r#"
        INSERT INTO demandes_capture (utilisateur_id, node_id)
        VALUES ($1, $2)
        RETURNING *
        "#,
        user_id,
        payload.node_id,
    )
    .fetch_one(&state.db)
    .await?;

    // Envoie la commande au port série
    match &state.serial_tx {
        Some(tx) => {
            tx.send("CMD:CAPTURE".to_string()).await
                .map_err(|e| AppError::Internal(
                    format!("Impossible d'envoyer la commande série : {}", e)
                ))?;
            tracing::info!("📸 Commande CMD:CAPTURE envoyée pour nœud {}", payload.node_id);
        }
        None => {
            // Pas de port série — on marque la demande comme échouée
            sqlx::query!(
                "UPDATE demandes_capture SET statut = 'echouee', message_erreur = $1 WHERE id = $2",
                "Port série non disponible",
                demande.id
            )
            .execute(&state.db)
            .await?;

            return Err(AppError::Internal(
                "Port série non disponible sur ce serveur".to_string()
            ));
        }
    }

    let base_url = format!(
        "http://{}:{}",
        state.config.server_host,
        state.config.server_port
    );

    Ok(Json(to_response(demande, &base_url)))
}

// GET /api/capturer/:job_id
// Le frontend poll pour connaître le statut
pub async fn statut_capture(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(job_id): Path<Uuid>,
) -> AppResult<Json<DemandeCapture>> {

    let demande = sqlx::query_as!(
        DemandeCaptureDb,
        "SELECT * FROM demandes_capture WHERE id = $1",
        job_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Demande introuvable".to_string()))?;

    // Si terminée, récupère le chemin de l'image
    let image_url = if demande.statut == "terminee" {
        if let Some(img_id) = demande.image_id {
            sqlx::query_scalar!(
                "SELECT chemin_stockage FROM images WHERE id = $1",
                img_id
            )
            .fetch_optional(&state.db)
            .await?
            .flatten()
            .map(|chemin| format!(
                "http://{}:{}/fichiers/{}",
                state.config.server_host,
                state.config.server_port,
                chemin.replace("data/nodes/", "")
            ))
        } else {
            None
        }
    } else {
        None
    };

    Ok(Json(DemandeCapture {
        id:         demande.id,
        node_id:    demande.node_id,
        statut:     demande.statut,
        image_url,
        created_at: demande.created_at,
        updated_at: demande.updated_at,
    }))
}

// GET /api/capturer/historique
// Historique des demandes de capture
pub async fn historique_captures(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<Vec<DemandeCaptureDb>>> {

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized)?;

    let demandes = sqlx::query_as!(
        DemandeCaptureDb,
        r#"
        SELECT * FROM demandes_capture
        WHERE utilisateur_id = $1
        ORDER BY created_at DESC
        LIMIT 50
        "#,
        user_id
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(demandes))
}