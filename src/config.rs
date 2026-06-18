use dotenvy::dotenv;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub jwt_secret: String,
    pub python_ai_url: String,
    pub serial_port:    Option<String>,
    pub serial_baud:    u32,
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
            serial_port:   env::var("SERIAL_PORT").ok()
                               .filter(|s| !s.is_empty()),
            serial_baud:   env::var("SERIAL_BAUD")
                               .unwrap_or_else(|_| "9600".to_string())
                               .parse().unwrap_or(9600),
        }
    }
}