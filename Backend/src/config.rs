use std::env;

#[derive(Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub mongodb_uri: String,
    pub mongodb_db: String,
    pub jwt_secret: String,
    pub jwt_exp_minutes: i64,
    pub cors_origin: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let host = env::var("APP_HOST").unwrap_or_else(|_| "127.0.0.1".into());
        let port = env::var("APP_PORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(8000);

        Self {
            host,
            port,
            mongodb_uri: env::var("MONGODB_URI").expect("MONGODB_URI is required"),
            mongodb_db: env::var("MONGODB_DB").unwrap_or_else(|_| "pcosew".into()),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET is required"),
            jwt_exp_minutes: env::var("JWT_EXP_MINUTES")
                .ok()
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(120),
            cors_origin: env::var("CORS_ORIGIN").unwrap_or_else(|_| "http://localhost:5173".into()),
        }
    }
}
