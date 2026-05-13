use dotenvy::dotenv;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub jwt_secret: String,
    pub python_ai_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok();
        Self {
            database_url: env::var("DATABASE_URL")
                .expect("DATABASE_URL manquant dans .env"),
            server_host: env::var("SERVER_HOST")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("SERVER_PORT doit être un nombre"),
            jwt_secret: env::var("JWT_SECRET")
                .expect("JWT_SECRET manquant dans .env"),
            python_ai_url: env::var("PYTHON_AI_URL")
                .unwrap_or_else(|_| "http://localhost:8000".to_string()),
        }
    }
}