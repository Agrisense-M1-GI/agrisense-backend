use axum::Router;
use axum::routing::{post, put, get};
use tower_http::services::ServeDir;
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tokio::sync::mpsc;

mod config;
mod db;
mod errors;
mod models;
mod routes;
mod middlewares;
mod serial_reader;
mod scheduler;

// État partagé injecté dans toutes les routes
#[derive(Clone)]
pub struct AppState {
    pub db: db::DbPool,
    pub config: Arc<config::Config>,
    pub http_client: reqwest::Client,
    pub serial_tx:   Option<mpsc::Sender<String>>,  // canal vers le port série
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


    // Canal pour envoyer des commandes au port série
    let (serial_tx, serial_rx) = mpsc::channel::<String>(32);

    let state = Arc::new(AppState {
        db: pool,
        config: config.clone(),
        http_client: reqwest::Client::new(),
        serial_tx:   Some(serial_tx),
    });

    // CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Lance le lecteur série en arrière-plan
    tokio::spawn(serial_reader::lancer_lecteur_serie(
        state.db.clone(),
        config.clone(),
        serial_rx,           // passe le receiver
    ));

    tracing::info!("📡 Lecteur série démarré en arrière-plan");

    // Scheduler 
    let config_sched  = config.clone();
    let client_sched  = reqwest::Client::new();
    tokio::spawn(async move {
        scheduler::lancer_scheduler(config_sched, client_sched).await;
    });

    tracing::info!("⏰ Scheduler démarré");

    let app = Router::new()
        .nest("/api", routes::all_routes(state.clone()))
        // Routes node directement sans /api
        .route("/node/:node_id/mode",           get(routes::node::get_mode))
        .route("/node/mode",                    put(routes::node::set_mode))
        .route("/node/:node_id/upload/image",   post(routes::node::upload_image))
        .route("/node/:node_id/upload/metrics", post(routes::node::upload_metrics))
        .nest_service("/fichiers", ServeDir::new("data/nodes"))
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

    // Crée le dossier de stockage s'il n'existe pas
    tokio::fs::create_dir_all("data/nodes")
        .await
        .expect("Impossible de créer le dossier data/nodes");
    tracing::info!("📁 Dossier data/nodes prêt");

    let addr = format!("{}:{}", config.server_host, config.server_port);
    
    let listener = tokio::net::TcpListener::bind(&addr)
    .await
    .unwrap();

    tracing::info!("🚀 Serveur démarré sur http://{}", addr);

    axum::serve(listener, app)
        .await
        .unwrap();
}