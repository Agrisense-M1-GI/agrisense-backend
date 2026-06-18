use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::mpsc;
use crate::config::Config;

// ─────────────────────────────────────────
// Structure d'une trame parsée
// ─────────────────────────────────────────
#[derive(Debug)]
struct TrameNoeud {
    node_id:     String,
    temperature: Option<f64>,
    humidite_air: Option<f64>,
    humidite_sol: Option<f64>,   // convertie en % (0-100)
}

// ─────────────────────────────────────────
// Parse une ligne série
// Exemple :
// [RX SPONTANE] DATA:ID:node_01:TS:993:LAT:0.0:LON:0.0:ZONE:ZONE_A:TEMP:29.3:HUM:72.0:SOL:1024
// ─────────────────────────────────────────
fn parse_ligne(ligne: &str) -> Option<TrameNoeud> {
    // On ne traite que les lignes DATA
    if !ligne.contains("DATA:") {
        return None;
    }

    // Extrait la partie après "DATA:"
    let data_part = ligne.split("DATA:").nth(1)?;
    let tokens: Vec<&str> = data_part.split(':').collect();

    let mut node_id      = String::new();
    let mut temperature  = None;
    let mut humidite_air = None;
    let mut humidite_sol = None;

    let mut i = 0;
    while i + 1 < tokens.len() {
        match tokens[i] {
            "ID"   => { node_id = tokens[i + 1].to_string(); i += 2; }
            "TEMP" => {
                temperature = tokens[i + 1].parse::<f64>().ok();
                i += 2;
            }
            "HUM"  => {
                humidite_air = tokens[i + 1].parse::<f64>().ok();
                i += 2;
            }
            "SOL"  => {
                // Conversion : 1024 = sec (0%) / 0 = humide (100%)
                if let Ok(raw) = tokens[i + 1].parse::<f64>() {
                    let pct = ((1024.0 - raw) / 1024.0 * 100.0)
                        .clamp(0.0, 100.0);
                    humidite_sol = Some(pct);
                }
                i += 2;
            }
            _ => { i += 1; }
        }
    }

    if node_id.is_empty() {
        return None;
    }

    Some(TrameNoeud { node_id, temperature, humidite_air, humidite_sol })
}

// Détecte si la ligne est un ACK de capture
fn parse_ack_capture(ligne: &str) -> Option<String> {
    // ACK:ID:noode_first_001:TS:147:CMD:CAPTURE:STATUS:OK
    if ligne.contains("CMD:CAPTURE") && ligne.contains("STATUS:OK") {
        let tokens: Vec<&str> = ligne.split(':').collect();
        for i in 0..tokens.len() {
            if tokens[i] == "ID" && i + 1 < tokens.len() {
                return Some(tokens[i + 1].to_string());
            }
        }
    }
    None
}

// Détecte si le nœud est injoignable
fn parse_echec(ligne: &str) -> bool {
    ligne.contains("NOEUD_INJOIGNABLE")
}


// ─────────────────────────────────────────
// Trouve le port série automatiquement
// ─────────────────────────────────────────
fn detecter_port() -> Option<String> {
    let ports = serialport::available_ports().ok()?;

    tracing::info!("🔍 Ports série disponibles :");
    for p in &ports {
        tracing::info!("   → {}", p.port_name);
    }

    // Prend le premier port USB/série disponible
    for port in &ports {
        let name = &port.port_name;
        // Sur Windows : COM3, COM4...  Sur Linux : /dev/ttyUSB0, /dev/ttyACM0
        if name.contains("COM") || name.contains("ttyUSB") || name.contains("ttyACM") {
            tracing::info!("✅ Port sélectionné automatiquement : {}", name);
            return Some(name.clone());
        }
    }
    None
}

// ─────────────────────────────────────────
// Insère les données en base
// ─────────────────────────────────────────
async fn inserer_donnees(
    db: &PgPool,
    trame: &TrameNoeud,
) -> Result<(), sqlx::Error> {

    // Récupère l'UUID du capteur depuis son nom/node_id
    let capteur = sqlx::query!(
        "SELECT id FROM noeuds_capteurs WHERE nom = $1",
        trame.node_id
    )
    .fetch_optional(db)
    .await?;

    let capteur_id = match capteur {
        Some(c) => c.id,
        None => {
            tracing::warn!(
                "⚠️  Nœud {} inconnu en base — données ignorées. \
                 Crée le capteur via POST /api/capteurs avec nom='{}'",
                trame.node_id, trame.node_id
            );
            return Ok(());
        }
    };

    // Température
    if let Some(temp) = trame.temperature {
        sqlx::query!(
            "INSERT INTO donnees_temperature (noeud_capteur_id, valeur) VALUES ($1, $2)",
            capteur_id, temp
        )
        .execute(db)
        .await?;
        tracing::info!("🌡️  Température : {}°C", temp);
    }

    // Humidité air
    if let Some(hum_air) = trame.humidite_air {
        sqlx::query!(
            "INSERT INTO donnees_humidite (noeud_capteur_id, valeur, type_humidite)
             VALUES ($1, $2, 'air')",
            capteur_id, hum_air
        )
        .execute(db)
        .await?;
        tracing::info!("💧 Humidité air : {}%", hum_air);
    }

    // Humidité sol
    if let Some(hum_sol) = trame.humidite_sol {
        sqlx::query!(
            "INSERT INTO donnees_humidite (noeud_capteur_id, valeur, type_humidite)
             VALUES ($1, $2, 'sol')",
            capteur_id, hum_sol
        )
        .execute(db)
        .await?;
        tracing::info!("🌱 Humidité sol : {:.1}%", hum_sol);
    }

    // Met à jour la dernière connexion du capteur
    sqlx::query!(
        "UPDATE noeuds_capteurs SET derniere_connexion = NOW() WHERE id = $1",
        capteur_id
    )
    .execute(db)
    .await?;

    Ok(())
}


// Met à jour la demande de capture quand ACK reçu
async fn traiter_ack_capture(db: &PgPool, node_id: &str) {
    let result = sqlx::query!(
        r#"
        UPDATE demandes_capture
        SET statut = 'ack_recu'
        WHERE node_id = $1
          AND statut = 'en_attente'
        "#,
        node_id
    )
    .execute(db)
    .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            tracing::info!("✅ ACK capture reçu pour nœud {}", node_id);
        }
        Ok(_) => {}
        Err(e) => tracing::error!("❌ Erreur MAJ ACK : {}", e),
    }
}


// Met à jour la demande de capture en cas d'échec
async fn traiter_echec_capture(db: &PgPool) {
    let result = sqlx::query!(
        r#"
        UPDATE demandes_capture
        SET statut         = 'echouee',
            message_erreur = 'Nœud injoignable — pas de réponse après 3 tentatives'
        WHERE statut IN ('en_attente', 'ack_recu')
        "#
    )
    .execute(db)
    .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            tracing::warn!("❌ Capture échouée — nœud injoignable");
        }
        Ok(_) => {}
        Err(e) => tracing::error!("❌ Erreur MAJ échec : {}", e),
    }
}

// Expire les demandes trop anciennes (timeout 2 minutes)
async fn expirer_demandes_anciennes(db: &PgPool) {
    let _ = sqlx::query!(
        r#"
        UPDATE demandes_capture
        SET statut         = 'echouee',
            message_erreur = 'Timeout — aucune image reçue après 2 minutes'
        WHERE statut IN ('en_attente', 'ack_recu')
          AND created_at < NOW() - INTERVAL '2 minutes'
        "#
    )
    .execute(db)
    .await;
}


// ─────────────────────────────────────────
// Tâche principale — lancée en background
// ─────────────────────────────────────────
pub async fn lancer_lecteur_serie(
    db:     PgPool,
    config: Arc<Config>,
    mut rx: mpsc::Receiver<String>,   // ← reçoit les commandes à envoyer
) {
    // Détermine le port à utiliser
    let port_name = match &config.serial_port {
        Some(p) => {
            tracing::info!("📡 Port série configuré : {}", p);
            p.clone()
        }
        None => {
            match detecter_port() {
                Some(p) => p,
                None => {
                    tracing::warn!(
                        "⚠️  Aucun port série détecté. \
                         Configure SERIAL_PORT dans .env si nécessaire."
                    );
                    return;
                }
            }
        }
    };

    // Tâche séparée pour expirer les demandes toutes les 30s
    let db_expire = db.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            expirer_demandes_anciennes(&db_expire).await;
        }
    });

    // Boucle de reconnexion — si le port se déconnecte, on réessaie
    loop {
        tracing::info!("🔌 Connexion au port série {} @ {} baud", port_name, config.serial_baud);

        let port = serialport::new(&port_name, config.serial_baud)
            .timeout(std::time::Duration::from_millis(100))
            .open();

        match port {
            Err(e) => {
                tracing::error!("❌ Impossible d'ouvrir {} : {}", port_name, e);
                tracing::info!("⏳ Nouvelle tentative dans 5 secondes...");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
            Ok(port) => {
                tracing::info!("✅ Port série {} ouvert", port_name);

                // Clone du port pour l'écriture
                let mut port_write = port.try_clone()
                    .expect("Impossible de cloner le port série");
                let mut port_read  = port;
                let mut buffer     = String::new();

                // Tâche d'écriture — reçoit les commandes depuis le canal
                tokio::spawn(async move {
                    while let Some(cmd) = rx.recv().await {
                        let line = format!("{}\n", cmd);
                        if let Err(e) = std::io::Write::write_all(&mut port_write, line.as_bytes()) {
                            tracing::error!("❌ Erreur écriture série : {}", e);
                        } else {
                            tracing::info!("📤 Envoyé sur série : {}", cmd);
                        }
                    }
                });

                // Boucle de lecture
                loop {
                    // Lecture octet par octet sur le port série
                    let mut byte = [0u8; 1];
                    match port_read.read(&mut byte) {
                        Ok(1) => {
                            let c = byte[0] as char;
                            if c == '\n' {
                                let ligne = buffer.trim().to_string();
                                buffer.clear();

                                if !ligne.is_empty() {
                                    tracing::debug!("📥 Série : {}", ligne);

                                    // Données télémétriques
                                    if let Some(trame) = parse_ligne(&ligne) {
                                        if let Err(e) = inserer_donnees(&db, &trame).await {
                                            tracing::error!("❌ Erreur insertion : {}", e);
                                        }
                                    }

                                    // ACK capture
                                    if let Some(node_id) = parse_ack_capture(&ligne) {
                                        traiter_ack_capture(&db, &node_id).await;
                                    }

                                    // Échec capture
                                    if parse_echec(&ligne) {
                                        traiter_echec_capture(&db).await;
                                    }
                                }
                            } else {
                                buffer.push(c);
                            }
                        }
                        Ok(_) | Err(_) => {
                            // Timeout ou erreur de lecture — normal, on continue
                        }
                    }

                    // Yield pour ne pas bloquer Tokio
                    tokio::task::yield_now().await;
                }
            }
        }
    }
}