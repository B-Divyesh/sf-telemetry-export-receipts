use std::{env, time::Duration};

#[derive(Clone, Debug)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub upstream_base_url: Option<String>,
    pub allowed_paths: Vec<String>,
    pub max_range: Duration,
    pub max_rows: u32,
    pub allowed_redactions: Vec<String>,
    pub identity_header: String,
    pub signing_key: String,
    pub build_sha: String,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let production = env::var("TER_APP_ENV").unwrap_or_default() == "production";
        let signing_key = env::var("TER_RECEIPT_SIGNING_KEY")
            .unwrap_or_else(|_| "local-development-key-change-me".into());
        if production && signing_key == "local-development-key-change-me" {
            return Err("TER_RECEIPT_SIGNING_KEY is required when TER_APP_ENV=production".into());
        }

        Ok(Self {
            port: parse("PORT", 8080)?,
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite://data/receipts.db?mode=rwc".into()),
            upstream_base_url: env::var("TER_UPSTREAM_BASE_URL")
                .ok()
                .map(|v| v.trim_end_matches('/').to_string()),
            allowed_paths: csv(
                "TER_ALLOWED_EXPORT_PATHS",
                "/api/logs/export,/api/traces/export,/api/metrics/export",
            ),
            max_range: Duration::from_secs(parse::<u64>("TER_MAX_EXPORT_RANGE_HOURS", 24)? * 3600),
            max_rows: parse("TER_MAX_EXPORT_ROWS", 10_000)?,
            allowed_redactions: csv("TER_ALLOWED_REDACTION_POLICIES", "pii-basic,strict"),
            identity_header: env::var("TER_IDENTITY_HEADER")
                .unwrap_or_else(|_| "x-export-user".into())
                .to_ascii_lowercase(),
            signing_key,
            build_sha: env::var("TER_BUILD_SHA").unwrap_or_else(|_| "development".into()),
        })
    }

    pub fn test() -> Self {
        Self {
            port: 0,
            database_url: "sqlite::memory:".into(),
            upstream_base_url: Some("http://127.0.0.1:9".into()),
            allowed_paths: vec!["/api/logs/export".into()],
            max_range: Duration::from_secs(3600),
            max_rows: 100,
            allowed_redactions: vec!["pii-basic".into()],
            identity_header: "x-export-user".into(),
            signing_key: "test-signing-key".into(),
            build_sha: "test".into(),
        }
    }
}

fn parse<T: std::str::FromStr>(name: &str, default: T) -> Result<T, String> {
    match env::var(name) {
        Ok(value) => value
            .parse()
            .map_err(|_| format!("{name} has an invalid value")),
        Err(_) => Ok(default),
    }
}

fn csv(name: &str, default: &str) -> Vec<String> {
    env::var(name)
        .unwrap_or_else(|_| default.into())
        .split(',')
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_owned)
        .collect()
}
