use axum::{
    extract::{Multipart, Path, State},
    Json,
};
use std::{sync::Arc, path::PathBuf};
use tokio::{fs, io::AsyncWriteExt};
use chrono::Utc;

use crate::{
    errors::{AppError, AppResult},
    models::{
        Image,
        MetricsPayload, ModeResponse, ModeUpdate, NoeudCapteur,
    },
    AppState,
};

// Dossier de stockage des fichiers reçus
const UPLOAD_DIR: &str = "data/nodes";

// ─────────────────────────────────────────────
// GET /api/node/:node_id/mode
// Le Pi consulte le mode au démarrage
// ─────────────────────────────────────────────
pub async fn get_mode(
    State(state): State<Arc<AppState>>,
    Path(node_id): Path<String>,
) -> AppResult<Json<ModeResponse>> {

    // Vérifie que le capteur existe
    let capteur = sqlx::query_as!(
        NoeudCapteur,
        r#"
        SELECT id, nom, type_capteur, longitude, latitude, batterie,
               etat, surface_couverte, derniere_connexion, created_at, updated_at
        FROM noeuds_capteurs WHERE nom = $1
        "#,
        node_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Nœud {} inconnu", node_id)))?;

    let mode = sqlx::query_scalar!(
        "SELECT valeur FROM config_systeme WHERE cle = 'mode'"
    )
    .fetch_one(&state.db)
    .await?;

    tracing::info!("📡 Nœud {} consulte le mode → {}", capteur.nom, mode);

    Ok(Json(ModeResponse {
        node_id: node_id.clone(),
        mode,
    }))
}

// ─────────────────────────────────────────────
// PUT /api/node/mode
// Changer le mode depuis l'interface admin
// ─────────────────────────────────────────────
pub async fn set_mode(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ModeUpdate>,
) -> AppResult<Json<ModeResponse>> {

    let mode = payload.mode.to_uppercase();

    if mode != "NORMAL" && mode != "MAINTENANCE" {
        return Err(AppError::BadRequest(
            "Mode invalide. Valeurs acceptées : NORMAL, MAINTENANCE".to_string()
        ));
    }

    sqlx::query!(
        "UPDATE config_systeme SET valeur = $1 WHERE cle = 'mode'",
        mode
    )
    .execute(&state.db)
    .await?;

    tracing::info!("🔧 Mode système changé → {}", mode);

    Ok(Json(ModeResponse {
        node_id: "system".to_string(),
        mode,
    }))
}

// ─────────────────────────────────────────────
// POST /api/node/:node_id/upload/image
// Le Pi envoie une image (multipart/form-data)
// ─────────────────────────────────────────────
pub async fn upload_image(
    State(state): State<Arc<AppState>>,
    Path(node_id): Path<String>,
    mut multipart: Multipart,
) -> AppResult<Json<Image>> {

    // Récupère l'UUID du capteur depuis son nom
    let capteur = sqlx::query_as!(
        NoeudCapteur,
        r#"
        SELECT id, nom, type_capteur, longitude, latitude, batterie,
               etat, surface_couverte, derniere_connexion, created_at, updated_at
        FROM noeuds_capteurs WHERE nom = $1
        "#,
        node_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Nœud {} inconnu", node_id)))?;

    // Crée le dossier de destination
    let upload_path = PathBuf::from(UPLOAD_DIR)
        .join(&node_id)
        .join("images");
    fs::create_dir_all(&upload_path).await
        .map_err(|e| AppError::Internal(format!("Erreur création dossier : {}", e)))?;

    // Lit le fichier depuis le multipart
    let mut file_bytes: Vec<u8> = Vec::new();
    let mut original_filename = String::from("image.jpg");

    while let Some(field) = multipart.next_field().await
        .map_err(|e| AppError::BadRequest(format!("Erreur multipart : {}", e)))?
    {
        if field.name() == Some("file") {
            if let Some(fname) = field.file_name() {
                original_filename = fname.to_string();
            }
            file_bytes = field.bytes().await
                .map_err(|e| AppError::BadRequest(format!("Erreur lecture fichier : {}", e)))?
                .to_vec();
        }
    }

    if file_bytes.is_empty() {
        return Err(AppError::BadRequest("Aucun fichier reçu".to_string()));
    }

    // Sauvegarde sur disque
    let timestamp  = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let extension  = PathBuf::from(&original_filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jpg")
        .to_string();
    let filename   = format!("{}_{}.{}", node_id, timestamp, extension);
    let dest       = upload_path.join(&filename);
    let chemin     = format!("{}/{}/images/{}", UPLOAD_DIR, node_id, filename);
    let taille     = file_bytes.len() as i64;

    let mut file = fs::File::create(&dest).await
        .map_err(|e| AppError::Internal(format!("Erreur écriture fichier : {}", e)))?;
    file.write_all(&file_bytes).await
        .map_err(|e| AppError::Internal(format!("Erreur écriture données : {}", e)))?;

    // Enregistre en base
    let image = sqlx::query_as!(
        Image,
        r#"
        INSERT INTO images (noeud_capteur_id, code, chemin_stockage, taille_octets, format)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, noeud_capteur_id, code, longueur, largeur,
                  chemin_stockage, taille_octets, format, date_capture, est_traitee, created_at
        "#,
        capteur.id,
        format!("{}_{}", node_id, timestamp),
        chemin,
        taille,
        extension,
    )
    .fetch_one(&state.db)
    .await?;

    // Après l'insertion de l'image en base, ferme le job de capture en cours
    sqlx::query!(
        r#"
        UPDATE demandes_capture
        SET statut   = 'terminee',
            image_id = $1
        WHERE node_id = $2
        AND statut IN ('en_attente', 'ack_recu')
        "#,
        image.id,
        node_id,
    )
    .execute(&state.db)
    .await?;

    tracing::info!("✅ Job de capture fermé pour nœud {}", node_id);

    tracing::info!(
        "🖼️  Image reçue du nœud {} → {} ({} octets)",
        node_id, filename, taille
    );

    Ok(Json(image))
}

// ─────────────────────────────────────────────
// POST /api/node/:node_id/upload/metrics
// Le Pi envoie un fichier JSON de métriques
// ─────────────────────────────────────────────
pub async fn upload_metrics(
    State(state): State<Arc<AppState>>,
    Path(node_id): Path<String>,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {

    // Récupère le capteur
    let capteur = sqlx::query_as!(
        NoeudCapteur,
        r#"
        SELECT id, nom, type_capteur, longitude, latitude, batterie,
               etat, surface_couverte, derniere_connexion, created_at, updated_at
        FROM noeuds_capteurs WHERE nom = $1
        "#,
        node_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Nœud {} inconnu", node_id)))?;

    // Crée le dossier de destination
    let upload_path = PathBuf::from(UPLOAD_DIR)
        .join(&node_id)
        .join("metrics");
    fs::create_dir_all(&upload_path).await
        .map_err(|e| AppError::Internal(format!("Erreur création dossier : {}", e)))?;

    // Lit le fichier JSON depuis le multipart
    let mut file_bytes: Vec<u8> = Vec::new();

    while let Some(field) = multipart.next_field().await
        .map_err(|e| AppError::BadRequest(format!("Erreur multipart : {}", e)))?
    {
        if field.name() == Some("file") {
            file_bytes = field.bytes().await
                .map_err(|e| AppError::BadRequest(format!("Erreur lecture fichier : {}", e)))?
                .to_vec();
        }
    }

    if file_bytes.is_empty() {
        return Err(AppError::BadRequest("Aucun fichier reçu".to_string()));
    }

    // Sauvegarde locale du fichier brut
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let dest      = upload_path.join(format!("{}.json", timestamp));
    let mut file  = fs::File::create(&dest).await
        .map_err(|e| AppError::Internal(format!("Erreur écriture fichier : {}", e)))?;
    file.write_all(&file_bytes).await
        .map_err(|e| AppError::Internal(format!("Erreur écriture données : {}", e)))?;

    // Parse le JSON
    let metrics: MetricsPayload = serde_json::from_slice(&file_bytes)
        .map_err(|e| AppError::BadRequest(format!("JSON invalide : {}", e)))?;

    let mut resultats = serde_json::json!({
        "status":  "ok",
        "node_id": node_id,
        "enregistre": {}
    });

    // Humidité
    if let Some(humidite) = metrics.humidity {
        sqlx::query!(
            "INSERT INTO donnees_humidite (noeud_capteur_id, valeur) VALUES ($1, $2)",
            capteur.id, humidite
        )
        .execute(&state.db)
        .await?;

        // Vérifie les seuils
        //verifier_seuil_humidite(&state, capteur, humidite).await?;
        verifier_seuil_humidite(&state, humidite).await?;

        resultats["enregistre"]["humidite"] = serde_json::json!(humidite);
        tracing::info!("💧 Humidité reçue : {}% — nœud {}", humidite, node_id);
    }

    // Température
    if let Some(temperature) = metrics.temperature {
        sqlx::query!(
            "INSERT INTO donnees_temperature (noeud_capteur_id, valeur) VALUES ($1, $2)",
            capteur.id, temperature
        )
        .execute(&state.db)
        .await?;

        resultats["enregistre"]["temperature"] = serde_json::json!(temperature);
        tracing::info!("🌡️  Température reçue : {}°C — nœud {}", temperature, node_id);
    }

    // Batterie
    if let Some(batterie) = metrics.battery {
        sqlx::query!(
            "UPDATE noeuds_capteurs SET batterie = $1, derniere_connexion = NOW() WHERE id = $2",
            batterie, capteur.id
        )
        .execute(&state.db)
        .await?;

        resultats["enregistre"]["batterie"] = serde_json::json!(batterie);
    }

    Ok(Json(resultats))
}

// ─────────────────────────────────────────────
// Vérifie les seuils d'humidité et notifie
// ─────────────────────────────────────────────
async fn verifier_seuil_humidite(
    state: &AppState,
    //capteur_id: uuid::Uuid,
    valeur: f64,
) -> AppResult<()> {
    let seuils = sqlx::query!(
        "SELECT utilisateur_id, valeur_min, valeur_max FROM seuils_humidite"
    )
    .fetch_all(&state.db)
    .await?;

    for seuil in seuils {
        let message = if valeur < seuil.valeur_min {
            Some(format!(
                "⚠️ Humidité critique ({:.1}%) sous le seuil minimum ({:.1}%)",
                valeur, seuil.valeur_min
            ))
        } else if valeur > seuil.valeur_max {
            Some(format!(
                "⚠️ Humidité excessive ({:.1}%) au-dessus du seuil maximum ({:.1}%)",
                valeur, seuil.valeur_max
            ))
        } else {
            None
        };

        if let Some(msg) = message {
            sqlx::query!(
                r#"INSERT INTO notifications (utilisateur_id, type, message, source)
                   VALUES ($1, 'alerte_critique', $2, 'humidite')"#,
                seuil.utilisateur_id,
                msg
            )
            .execute(&state.db)
            .await?;

            tracing::warn!("🚨 {}", msg);
        }
    }
    Ok(())
}