use std::{env, fs, io::Write, path::Path, time::Duration};
use uuid::Uuid;

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
        let signing_key = signing_key(production)?;

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

fn signing_key(production: bool) -> Result<String, String> {
    if let Ok(value) = env::var("TER_RECEIPT_SIGNING_KEY") {
        if production && value.len() < 32 {
            return Err("TER_RECEIPT_SIGNING_KEY must contain at least 32 characters".into());
        }
        return Ok(value);
    }
    if !production {
        return Ok("local-development-key-change-me".into());
    }

    let path =
        env::var("TER_SIGNING_KEY_FILE").unwrap_or_else(|_| "data/receipt-signing.key".into());
    if let Ok(value) = fs::read_to_string(&path) {
        let value = value.trim().to_owned();
        if value.len() >= 32 {
            return Ok(value);
        }
        return Err(format!("{path} contains an invalid signing key"));
    }
    if let Some(parent) = Path::new(&path).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create signing key directory: {e}"))?;
    }
    let value = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    match options.open(&path) {
        Ok(mut file) => {
            file.write_all(value.as_bytes())
                .map_err(|e| format!("cannot write signing key: {e}"))?;
            tracing::warn!(key_file = %path, "generated a persistent receipt signing key; back up this file");
            Ok(value)
        }
        Err(_) => fs::read_to_string(&path)
            .map(|v| v.trim().to_owned())
            .map_err(|e| format!("cannot create or read signing key: {e}")),
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
