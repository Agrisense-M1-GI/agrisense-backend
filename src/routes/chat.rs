use axum::{
    extract::{Extension, Path, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    middlewares::auth::Claims,
    models::{
        Conversation, IaMessage, IaRequest, IaResponse,
        MessageChat, MessageResponse, NouveauMessagePayload,
    },
    AppState,
};

// ─────────────────────────────────────────────
// POST /api/chat
// L'utilisateur envoie un message
// ─────────────────────────────────────────────
pub async fn envoyer_message(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<NouveauMessagePayload>,
) -> AppResult<Json<MessageResponse>> {

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized)?;

    // ── Crée ou récupère la conversation ─────────────────────
    let conversation_id = match payload.conversation_id {
        Some(id) => {
            // Vérifie que la conversation appartient à l'utilisateur
            sqlx::query_scalar!(
                "SELECT id FROM conversations WHERE id = $1 AND utilisateur_id = $2",
                id, user_id
            )
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Conversation introuvable".to_string()))?
        }
        None => {
            // Nouvelle conversation — titre généré depuis les premiers mots du message
            let titre = payload.contenu
                .chars()
                .take(50)
                .collect::<String>();

            sqlx::query_scalar!(
                r#"
                INSERT INTO conversations (utilisateur_id, titre)
                VALUES ($1, $2)
                RETURNING id
                "#,
                user_id,
                titre,
            )
            .fetch_one(&state.db)
            .await?
        }
    };

    // ── Enregistre le message de l'utilisateur ───────────────
    let message_user = sqlx::query_as!(
        MessageChat,
        r#"
        INSERT INTO messages_chat (conversation_id, role, contenu, statut, image_id)
        VALUES ($1, 'user', $2, 'terminee', $3)
        RETURNING *
        "#,
        conversation_id,
        payload.contenu,
        payload.image_id,
    )
    .fetch_one(&state.db)
    .await?;

    // ── Crée le message assistant en attente ─────────────────
    let message_assistant = sqlx::query_as!(
        MessageChat,
        r#"
        INSERT INTO messages_chat (conversation_id, role, contenu, statut)
        VALUES ($1, 'assistant', '', 'en_attente')
        RETURNING *
        "#,
        conversation_id,
    )
    .fetch_one(&state.db)
    .await?;

    let message_assistant_id = message_assistant.id;

    // ── Lance l'appel au modèle IA en arrière-plan ───────────
    let db          = state.db.clone();
    let http_client = state.http_client.clone();
    let ai_url      = state.config.python_ai_url.clone();

    // Récupère l'historique de la conversation pour le contexte
    let historique = sqlx::query_as!(
        MessageChat,
        r#"
        SELECT * FROM messages_chat
        WHERE conversation_id = $1
          AND statut = 'terminee'
          AND role   = 'user'
        ORDER BY created_at ASC
        "#,
        conversation_id,
    )
    .fetch_all(&state.db)
    .await?;

    // URL de l'image si fournie
    let image_url = if let Some(img_id) = payload.image_id {
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
    };

    // Tâche background — appel au modèle Python
    tokio::spawn(async move {
        appeler_modele_ia(
            db,
            http_client,
            ai_url,
            message_assistant_id,
            historique,
            payload.contenu,
            image_url,
        )
        .await;
    });

    Ok(Json(MessageResponse {
        message_user,
        message_assistant,
        conversation_id,
    }))
}

// ─────────────────────────────────────────────
// Appelle le modèle Python et met à jour le message
// ─────────────────────────────────────────────
async fn appeler_modele_ia(
    db:                  sqlx::PgPool,
    http_client:         reqwest::Client,
    ai_url:              String,
    message_assistant_id: Uuid,
    historique:          Vec<MessageChat>,
    nouveau_message:     String,
    image_url:           Option<String>,
) {
    // Construit l'historique pour le contexte
    let mut messages: Vec<IaMessage> = historique
        .into_iter()
        .map(|m| IaMessage {
            role:    m.role,
            content: m.contenu,
        })
        .collect();

    // Ajoute le nouveau message
    messages.push(IaMessage {
        role:    "user".to_string(),
        content: nouveau_message,
    });

    let ia_request = IaRequest {
        messages,
        image_url,
    };

    // Appelle le serveur Python
    let result = http_client
        .post(format!("{}/chat", ai_url))
        .json(&ia_request)
        .timeout(std::time::Duration::from_secs(120))  // 2min max
        .send()
        .await;

    match result {
        Ok(response) if response.status().is_success() => {
            match response.json::<IaResponse>().await {
                Ok(ia_response) => {
                    // Met à jour le message avec la réponse
                    let _ = sqlx::query!(
                        r#"
                        UPDATE messages_chat
                        SET contenu = $1, statut = 'terminee'
                        WHERE id = $2
                        "#,
                        ia_response.response,
                        message_assistant_id,
                    )
                    .execute(&db)
                    .await;

                    tracing::info!("✅ Réponse IA reçue pour message {}", message_assistant_id);
                }
                Err(e) => {
                    tracing::error!("❌ Erreur parsing réponse IA : {}", e);
                    marquer_message_echoue(&db, message_assistant_id).await;
                }
            }
        }
        Ok(response) => {
            tracing::error!("❌ Erreur HTTP modèle IA : {}", response.status());
            marquer_message_echoue(&db, message_assistant_id).await;
        }
        Err(e) => {
            tracing::error!("❌ Impossible de contacter le modèle IA : {}", e);
            marquer_message_echoue(&db, message_assistant_id).await;
        }
    }
}

async fn marquer_message_echoue(db: &sqlx::PgPool, message_id: Uuid) {
    let _ = sqlx::query!(
        r#"
        UPDATE messages_chat
        SET contenu = 'Désolé, une erreur est survenue. Veuillez réessayer.',
            statut  = 'echouee'
        WHERE id = $1
        "#,
        message_id,
    )
    .execute(db)
    .await;
}

// ─────────────────────────────────────────────
// GET /api/chat/:message_id/statut
// Frontend poll pour voir si la réponse est prête
// ─────────────────────────────────────────────
pub async fn statut_message(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(message_id): Path<Uuid>,
) -> AppResult<Json<MessageChat>> {

    let message = sqlx::query_as!(
        MessageChat,
        "SELECT * FROM messages_chat WHERE id = $1",
        message_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Message introuvable".to_string()))?;

    Ok(Json(message))
}

// ─────────────────────────────────────────────
// GET /api/chat/conversations
// Liste toutes les conversations de l'utilisateur
// ─────────────────────────────────────────────
pub async fn get_conversations(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<Vec<Conversation>>> {

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized)?;

    let conversations = sqlx::query_as!(
        Conversation,
        r#"
        SELECT * FROM conversations
        WHERE utilisateur_id = $1
        ORDER BY updated_at DESC
        "#,
        user_id
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(conversations))
}

// ─────────────────────────────────────────────
// GET /api/chat/conversations/:id
// Récupère tous les messages d'une conversation
// ─────────────────────────────────────────────
pub async fn get_messages_conversation(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(conversation_id): Path<Uuid>,
) -> AppResult<Json<Vec<MessageChat>>> {

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized)?;

    // Vérifie que la conversation appartient à l'utilisateur
    sqlx::query_scalar!(
        "SELECT id FROM conversations WHERE id = $1 AND utilisateur_id = $2",
        conversation_id, user_id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Conversation introuvable".to_string()))?;

    let messages = sqlx::query_as!(
        MessageChat,
        r#"
        SELECT * FROM messages_chat
        WHERE conversation_id = $1
        ORDER BY created_at ASC
        "#,
        conversation_id
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(messages))
}

// ─────────────────────────────────────────────
// DELETE /api/chat/conversations/:id
// Supprime une conversation et ses messages
// ─────────────────────────────────────────────
pub async fn delete_conversation(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(conversation_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized)?;

    let result = sqlx::query!(
        "DELETE FROM conversations WHERE id = $1 AND utilisateur_id = $2",
        conversation_id, user_id
    )
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Conversation introuvable".to_string()));
    }

    Ok(Json(serde_json::json!({ "message": "Conversation supprimée" })))
}