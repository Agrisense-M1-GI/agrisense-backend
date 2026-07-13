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

fn build_capture_timestamp() -> String {
    Utc::now().format("%Y%m%d_%H%M%S").to_string()
}

fn build_image_filename(node_id: &str, timestamp: &str, extension: &str) -> String {
    format!("{}_{}.{}", node_id, timestamp, extension)
}

fn build_metrics_filename(timestamp: &str) -> String {
    format!("{}.json", timestamp)
}

async fn resolve_capture_timestamp(state: &AppState, node_id: &str) -> AppResult<String> {
    if let Some(code) = sqlx::query_scalar!(
        r#"
        SELECT code
        FROM images
        WHERE noeud_capteur_id = (
            SELECT id FROM noeuds_capteurs WHERE nom = $1
        )
        ORDER BY created_at DESC, id DESC
        LIMIT 1
        "#,
        node_id
    )
    .fetch_optional(&state.db)
    .await?
    {
        if let Some(code) = code {
            let normalized = code
                .strip_prefix(&format!("{}_", node_id))
                .unwrap_or(&code);
            return Ok(normalized.to_string());
        }
    }

    let metrics_dir = PathBuf::from(UPLOAD_DIR).join(node_id).join("metrics");
    let mut metrics_files = Vec::new();

    if let Ok(mut entries) = fs::read_dir(&metrics_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                if ext.eq_ignore_ascii_case("json") {
                    if let Some(file_name) = entry.file_name().to_str() {
                        if let Some(stem) = file_name.strip_suffix(".json") {
                            metrics_files.push(stem.to_string());
                        }
                    }
                }
            }
        }
    }

    if let Some(latest) = metrics_files.iter().max() {
        return Ok(latest.clone());
    }

    Ok(build_capture_timestamp())
}

pub async fn appeler_analyse_visuelle(
    http_client: reqwest::Client,
    ai_url:      String,
    server_host: String,
    server_port: u16,
    image_id:    String,
    node_id:     String,
    chemin:      String,
    db:          sqlx::PgPool,
) {
    // Lit le fichier image depuis le disque
    let image_path = std::path::Path::new(&chemin);
    let file_bytes = match tokio::fs::read(image_path).await {
        Ok(b)  => b,
        Err(e) => {
            tracing::error!("❌ Impossible de lire l'image {} : {}", chemin, e);
            return;
        }
    };

    // URL du callback que le service Python appellera avec le résultat
    let callback_url = format!(
        "http://{}:{}/api/ia/callback/image",
        server_host, server_port
    );

    // Construit le multipart — format attendu par POST /analyze-image
    let filename  = image_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("image.jpg")
        .to_string();

    let part = reqwest::multipart::Part::bytes(file_bytes)
        .file_name(filename)
        .mime_str("image/jpeg")
        .unwrap();

    let form = reqwest::multipart::Form::new()
        .part("image", part)
        .text("sensor_id", node_id.clone())
        // On passe le callback_url en champ texte supplémentaire
        // Le service Python doit le lire dans sa config (BACKEND_IMAGE_RESULT_URL)
        // mais on le passe aussi ici pour référence
        .text("callback_url", callback_url);

    match http_client
        .post(format!("{}/analyze-image", ai_url))
        .multipart(form)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
    {
        Ok(res) if res.status().is_success() => {
            match res.json::<serde_json::Value>().await {
                Ok(body) => {
                    // Récupère l'image_id généré par le service Python
                    if let Some(model_image_id) = body
                        .get("image_id")
                        .and_then(|v| v.as_str())
                    {
                        tracing::info!(
                            "📤 Analyse IA lancée — model_image_id: {}",
                            model_image_id
                        );

                        // Sauvegarde le model_image_id dans notre table images
                        // pour faire le lien lors du callback
                        let our_id = match uuid::Uuid::parse_str(&image_id) {
                            Ok(id) => id,
                            Err(_) => {
                                tracing::error!("❌ UUID image invalide : {}", image_id);
                                return;
                            }
                        };

                        if let Err(e) = sqlx::query!(
                            "UPDATE images SET model_image_id = $1 WHERE id = $2",
                            model_image_id,
                            our_id,
                        )
                        .execute(&db)
                        .await
                        {
                            tracing::error!("❌ Erreur sauvegarde model_image_id : {}", e);
                        } else {
                            tracing::info!(
                                "✅ model_image_id {} lié à notre image {}",
                                model_image_id, image_id
                            );
                        }
                    }
                }
                Err(e) => tracing::error!("❌ Erreur parsing réponse IA : {}", e),
            }
        }
        Ok(res) => tracing::error!("❌ Erreur service IA : {}", res.status()),
        Err(e)  => tracing::error!("❌ Service IA injoignable : {}", e),
    }
}

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
    let capture_timestamp = resolve_capture_timestamp(&state, &node_id).await?;
    let extension  = PathBuf::from(&original_filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jpg")
        .to_string();
    let filename   = build_image_filename(&node_id, &capture_timestamp, &extension);
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
              chemin_stockage, taille_octets, format, date_capture, est_traitee, model_image_id, created_at
        "#,
        capteur.id,
        capture_timestamp,
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

    // Lance l'analyse IA en arrière-plan
    let db_clone     = state.db.clone();
    let http_client  = state.http_client.clone();
    let ai_url       = state.config.python_ai_url.clone();
    let server_host  = state.config.server_host.clone();
    let server_port  = state.config.server_port;
    let image_id_str = image.id.to_string();
    let node_id_str = node_id.clone();

    tokio::spawn(async move {
        appeler_analyse_visuelle(
            http_client,
            ai_url,
            server_host,
            server_port,
            image_id_str,
            node_id_str,
            chemin.clone(),
            db_clone,
        )
        .await;
    });

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
    let capture_timestamp = resolve_capture_timestamp(&state, &node_id).await?;
    let dest      = upload_path.join(build_metrics_filename(&capture_timestamp));
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