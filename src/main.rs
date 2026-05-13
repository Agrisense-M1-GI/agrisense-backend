use axum::Router;
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod db;
mod errors;
mod models;
mod routes;
mod middlewares;

// État partagé injecté dans toutes les routes
#[derive(Clone)]
pub struct AppState {
    pub db: db::DbPool,
    pub config: Arc<config::Config>,
    pub http_client: reqwest::Client,
}

#[tokio::main]
async fn main() {
    // Logs
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Config + DB
    let config = Arc::new(config::Config::from_env());
    let pool = db::init_pool(&config).await;

    // Migrations automatiques au démarrage
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Erreur lors des migrations");

    let state = AppState {
        db: pool,
        config: config.clone(),
        http_client: reqwest::Client::new(),
    };

    // CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .nest("/api", routes::all_routes())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("{}:{}", config.server_host, config.server_port);
    
    let listener = tokio::net::TcpListener::bind(&addr)
    .await
    .unwrap();

    tracing::info!("🚀 Serveur démarré sur http://{}", addr);

    axum::serve(listener, app)
        .await
        .unwrap();
}