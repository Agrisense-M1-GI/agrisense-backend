use sqlx::postgres::PgPoolOptions;
use crate::config::Config;

pub type DbPool = sqlx::PgPool;

pub async fn init_pool(config: &Config) -> DbPool {
    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&config.database_url)
        .await
        .expect("Impossible de se connecter à PostgreSQL")
}