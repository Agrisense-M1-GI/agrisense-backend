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

pub fn all_routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    let public =Router::new()
        // Health
        .route("/health", get(health::health_handler))
        // Auth — routes publiques
        .route("/auth/register", post(auth::register))
        .route("/auth/login",    post(auth::login))
        .route("/humidite",      post(humidite::recevoir_mesure))
        .route("/images",        post(images::recevoir_image))
        .route("/temperature", post(temperature::recevoir_mesure));

    
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
        .route("/capteurs/:id/etat", patch(capteurs::update_etat_capteur))
        // Seuils
        .route("/seuils",  get(seuils::get_seuil))
        .route("/seuils",  post(seuils::upsert_seuil))
        // Applique le middleware JWT sur toutes les routes protégées
        .route("/humidite/:capteur_id",         get(humidite::get_historique))
        .route("/humidite/:capteur_id/derniere", get(humidite::get_derniere_mesure))
        // Images
        .route("/images/:capteur_id",                get(images::get_historique_images))
        .route("/images/:capteur_id/non-traitees",   get(images::get_images_non_traitees))
        .route("/images/detail/:id",                 get(images::get_image))
        .route("/temperature/:capteur_id",          get(temperature::get_historique))
        .route("/temperature/:capteur_id/derniere", get(temperature::get_derniere_mesure))
        .route_layer(middleware::from_fn_with_state(state, require_auth));

    Router::new()
        .merge(public)
        .merge(protected)
}