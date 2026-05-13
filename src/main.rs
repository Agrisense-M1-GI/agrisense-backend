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
mod middlewares;  // ← déjà présent normalement

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
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=debug".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)        // affiche le module source du log
                .with_thread_ids(false)   // pas besoin des IDs de thread
                .with_level(true)         // affiche le niveau (INFO, DEBUG...)
                .pretty(),                // format lisible
        )
        .init();

    // Config + DB
    let config = Arc::new(config::Config::from_env());
    let pool = db::init_pool(&config).await;

    // Migrations automatiques au démarrage
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Erreur lors des migrations");

    let state = Arc::new(AppState {
        db: pool,
        config: config.clone(),
        http_client: reqwest::Client::new(),
    });

    // CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .nest("/api", routes::all_routes(state.clone()))
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| {
                    tracing::info_span!(
                        "requête",
                        method  = %request.method(),
                        uri     = %request.uri(),
                        version = ?request.version(),
                    )
                })
                .on_request(|request: &axum::http::Request<_>, _span: &tracing::Span| {
                    tracing::info!(
                        "→ {} {}",
                        request.method(),
                        request.uri()
                    );
                })
                .on_response(|response: &axum::http::Response<_>, latency: std::time::Duration, _span: &tracing::Span| {
                    tracing::info!(
                        "← {} | {:?}",
                        response.status(),
                        latency
                    );
                })
                .on_failure(|error: tower_http::classify::ServerErrorsFailureClass, latency: std::time::Duration, _span: &tracing::Span| {
                    tracing::error!(
                        "✗ erreur {:?} après {:?}",
                        error,
                        latency
                    );
                })
        )
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