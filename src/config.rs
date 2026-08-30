use std::{env, fs, io::Write, path::PathBuf, time::Duration};
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
    pub admin_token: String,
    pub build_sha: String,
    pub provenance: ConfigProvenance,
}

#[derive(Clone, Debug)]
pub struct ConfigProvenance {
    pub database: &'static str,
    pub signing_key: &'static str,
    pub admin_token: &'static str,
    pub upstream: &'static str,
}

struct Secret {
    value: String,
    source: &'static str,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let state_dir = default_state_dir();
        let default_database = format!(
            "sqlite://{}?mode=rwc",
            state_dir.join("receipts.db").display()
        );
        let (database_url, database_source) = match env::var("DATABASE_URL") {
            Ok(value) => (value, "supplied"),
            Err(_) => (default_database, "default durable path"),
        };
        let signing = load_or_create_secret(
            "TER_RECEIPT_SIGNING_KEY",
            "TER_SIGNING_KEY_FILE",
            state_dir.join("receipt-signing.key"),
            "receipt signing key",
        )?;
        let admin = load_or_create_secret(
            "TER_ADMIN_TOKEN",
            "TER_ADMIN_TOKEN_FILE",
            state_dir.join("admin-access.key"),
            "administrator access token",
        )?;
        let upstream_base_url = env::var("TER_UPSTREAM_BASE_URL")
            .ok()
            .map(|v| v.trim_end_matches('/').to_string());

        Ok(Self {
            port: parse("PORT", 8080)?,
            database_url,
            upstream_base_url: upstream_base_url.clone(),
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
            signing_key: signing.value,
            admin_token: admin.value,
            build_sha: env::var("TER_BUILD_SHA").unwrap_or_else(|_| "development".into()),
            provenance: ConfigProvenance {
                database: database_source,
                signing_key: signing.source,
                admin_token: admin.source,
                upstream: if upstream_base_url.is_some() {
                    "supplied"
                } else {
                    "unset"
                },
            },
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
            admin_token: "test-admin-token".into(),
            build_sha: "test".into(),
            provenance: ConfigProvenance {
                database: "test",
                signing_key: "test",
                admin_token: "test",
                upstream: "test",
            },
        }
    }
}

fn default_state_dir() -> PathBuf {
    let mounted = PathBuf::from("/data");
    if mounted.is_dir() {
        mounted
    } else {
        PathBuf::from("data")
    }
}

fn load_or_create_secret(
    value_env: &str,
    file_env: &str,
    default_path: PathBuf,
    label: &str,
) -> Result<Secret, String> {
    if let Ok(value) = env::var(value_env) {
        if value.len() < 32 {
            return Err(format!("{value_env} must contain at least 32 characters"));
        }
        return Ok(Secret {
            value,
            source: "supplied",
        });
    }

    let path = env::var(file_env)
        .map(PathBuf::from)
        .unwrap_or(default_path);
    if let Ok(value) = fs::read_to_string(&path) {
        let value = value.trim().to_owned();
        if value.len() >= 32 {
            return Ok(Secret {
                value,
                source: "persisted",
            });
        }
        return Err(format!("{} contains an invalid secret", path.display()));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {} directory: {e}", path.display()))?;
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
                .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
            tracing::warn!(secret_file = %path.display(), %label, "generated persistent secret; back up this file");
            Ok(Secret {
                value,
                source: "generated",
            })
        }
        Err(_) => {
            let value = fs::read_to_string(&path)
                .map(|v| v.trim().to_owned())
                .map_err(|e| format!("cannot create or read {}: {e}", path.display()))?;
            if value.len() < 32 {
                return Err(format!("{} contains an invalid secret", path.display()));
            }
            Ok(Secret {
                value,
                source: "persisted",
            })
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
