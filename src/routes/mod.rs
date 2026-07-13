use axum::{middleware, routing::{delete, get, patch, post, put}, Router};
use crate::{middlewares::auth::require_auth, AppState};
use std::sync::Arc;

mod auth;
mod capteurs;
mod champs;
mod cultures;
mod health;
mod seuils;
mod utilisateurs;
mod humidite;
mod images;
mod temperature;
pub mod node;
mod capture;
mod chat;
mod notifications;
mod seuils_temperature;
mod recommandations;

pub fn all_routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    let public =Router::new()
        // Health
        .route("/health", get(health::health_handler))
        // Auth — routes publiques
        .route("/auth/register", post(auth::register))
        .route("/auth/login",    post(auth::login))
        // Images et donnees scalaires du noeud capteur
        .route("/images",        post(images::recevoir_image))
        .route("/humidite",      post(humidite::recevoir_mesure))
        .route("/temperature",   post(temperature::recevoir_mesure))
        // appelle par le modele IA
        .route("/ia/callback/image",     post(recommandations::callback_image))
        .route("/ia/callback/metriques", post(recommandations::callback_metriques))
        .route("/ia/metrics-source",     get(recommandations::metrics_source));

    
    let protected = Router::new()
        // Utilisateur
        .route("/utilisateurs/me",  get(utilisateurs::get_me))
        .route("/utilisateurs/me",  put(utilisateurs::update_me))
        // Champs
        .route("/champs",           get(champs::get_champs))
        .route("/champs",           post(champs::create_champ))
        .route("/champs/:id",       get(champs::get_champ))
        .route("/champs/:id",       put(champs::update_champ))
        .route("/champs/:id",       delete(champs::delete_champ))
        // Cultures (imbriquées sous champs)
        .route("/champs/:champ_id/cultures",      get(cultures::get_cultures))
        .route("/champs/:champ_id/cultures",      post(cultures::create_culture))
        .route("/champs/:champ_id/cultures/:id",  put(cultures::update_culture))
        .route("/champs/:champ_id/cultures/:id",  delete(cultures::delete_culture))
        // Capteurs
        .route("/capteurs",          get(capteurs::get_capteurs))
        .route("/capteurs",          post(capteurs::create_capteur))
        .route("/capteurs/:id",      get(capteurs::get_capteur))
        .route("/capteurs/:id",      put(capteurs::update_capteur))
        .route("/capteurs/:id",      delete(capteurs::delete_capteur))
        .route("/capteurs/:id/etat", patch(capteurs::update_etat_capteur))
        // Seuils
        .route("/seuils",  get(seuils::get_seuil))
        .route("/seuils",  post(seuils::upsert_seuil))
        .route("/humidite/:capteur_id",         get(humidite::get_historique))
        .route("/humidite/:capteur_id/derniere", get(humidite::get_derniere_mesure))
        .route("/seuils/temperature", get(seuils_temperature::get_seuil))
        .route("/seuils/temperature", post(seuils_temperature::upsert_seuil))
        // Images
        .route("/images/:capteur_id",                get(images::get_historique_images))
        .route("/images/:capteur_id/non-traitees",   get(images::get_images_non_traitees))
        .route("/images/detail/:id",                 get(images::get_image))
        .route("/temperature/:capteur_id",          get(temperature::get_historique))
        .route("/temperature/:capteur_id/derniere", get(temperature::get_derniere_mesure))
        // Demandes de capture
        .route("/capturer",              post(capture::demander_capture))
        .route("/capturer/historique",   get(capture::historique_captures))
        .route("/capturer/:job_id",      get(capture::statut_capture))
        // Chat
        .route("/chat",                              post(chat::envoyer_message))
        .route("/chat/:message_id/statut",           get(chat::statut_message))
        .route("/chat/conversations",                get(chat::get_conversations))
        .route("/chat/conversations/:id",            get(chat::get_messages_conversation))
        .route("/chat/conversations/:id",            delete(chat::delete_conversation))
        // Notifications
        .route("/notifications",                     get(notifications::get_notifications))
        .route("/notifications/:id/lue",             patch(notifications::mark_as_lue))
        // Recommandation
        .route("/recommandations",                 get(recommandations::get_recommandations))
        .route("/recommandations/non-lues",        get(recommandations::get_recommandations_non_lues))
        .route("/recommandations/tout-lire",       patch(recommandations::tout_marquer_lue))
        .route("/recommandations/:id",             get(recommandations::get_recommandation))
        .route("/recommandations/:id/lue",         patch(recommandations::marquer_lue))
        .route("/recommandations/:id",             delete(recommandations::delete_recommandation))
        // Analyses journalieres
        .route("/analyses",                        get(recommandations::get_analyses))
        .route("/analyses/aujourd-hui",            get(recommandations::get_analyse_aujourd_hui))
        .route("/analyses/:id",                    get(recommandations::get_analyse))
        .route("/analyses/:id/lue",                patch(recommandations::marquer_analyse_lue))
        // Applique le middleware JWT sur toutes les routes protégées
        .route_layer(middleware::from_fn_with_state(state, require_auth));

    Router::new()
        .merge(public)
        .merge(protected)
        
}