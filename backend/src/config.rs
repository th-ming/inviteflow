use std::{env, path::PathBuf};

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub admin_token: String,
    pub db_driver: String,
    pub database_url: String,
    pub database_path: String,
    pub data_file: PathBuf,
    pub github_token: String,
    pub github_repo: String,
    pub github_path: String,
    pub allowed_origin: String,
    pub rate_limit: u64,
    pub rate_window_ms: u64,
    pub frontend_dir: PathBuf,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        dotenvy::dotenv().ok();

        let admin_token =
            env::var("ADMIN_TOKEN").map_err(|_| "ADMIN_TOKEN là bắt buộc".to_string())?;
        let port = env::var("PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8080);
        let rate_limit = env::var("RATE_LIMIT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);
        let rate_window_ms = env::var("RATE_WINDOW_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60_000);

        let config = Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port,
            admin_token,
            db_driver: env::var("DB_DRIVER").unwrap_or_else(|_| "github".into()),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "https://example.firebaseio.com".into())
                .trim_end_matches('/')
                .to_string(),
            database_path: env::var("DATABASE_PATH")
                .unwrap_or_else(|_| "responses".into())
                .trim_matches('/')
                .to_string(),
            data_file: PathBuf::from(
                env::var("DATA_FILE").unwrap_or_else(|_| "data/responses.json".into()),
            ),
            github_token: env::var("GITHUB_TOKEN").unwrap_or_default(),
            github_repo: env::var("GITHUB_REPO").unwrap_or_else(|_| "th-ming/inviteflow-db".into()),
            github_path: env::var("GITHUB_PATH").unwrap_or_else(|_| "responses.json".into()),
            allowed_origin: env::var("ALLOWED_ORIGIN").unwrap_or_else(|_| "*".into()),
            rate_limit,
            rate_window_ms,
            frontend_dir: PathBuf::from(
                env::var("FRONTEND_DIR").unwrap_or_else(|_| "../frontend/public".into()),
            ),
        };

        if config.db_driver.eq_ignore_ascii_case("firebase")
            && config.database_url.contains("example.")
        {
            return Err("DATABASE_URL là bắt buộc".into());
        }
        if config.db_driver.eq_ignore_ascii_case("github") && config.github_token.is_empty() {
            return Err("GITHUB_TOKEN là bắt buộc".into());
        }

        Ok(config)
    }
}
