use axum::Router;
use crate::AppState;

pub fn all_routes() -> Router<AppState> {
    Router::new()
    // Les routes seront ajoutées ici phase par phase :
    // .nest("/auth", auth::router())
    // .nest("/champs", champs::router())
    // etc.
}