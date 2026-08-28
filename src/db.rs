use crate::receipt::{Receipt, StoredReceipt};
use sqlx::{sqlite::SqlitePoolOptions, Row, SqlitePool};

pub async fn connect(url: &str) -> Result<SqlitePool, sqlx::Error> {
    if let Some(path) = url
        .strip_prefix("sqlite://")
        .and_then(|v| v.split('?').next())
    {
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent).ok();
        }
    }
    let pool = SqlitePoolOptions::new()
        .max_connections(if url == "sqlite::memory:" { 1 } else { 5 })
        .connect(url)
        .await?;
    sqlx::query(
        include_str!("../migrations/0001_receipts.sql")
            .split("CREATE INDEX")
            .next()
            .unwrap(),
    )
    .execute(&pool)
    .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS receipts_created_at ON receipts(created_at DESC)")
        .execute(&pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS receipts_requester ON receipts(requester)")
        .execute(&pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS receipts_outcome ON receipts(outcome)")
        .execute(&pool)
        .await?;
    Ok(pool)
}

pub async fn insert(
    pool: &SqlitePool,
    receipt: &Receipt,
    signature: &str,
) -> Result<(), sqlx::Error> {
    let json = serde_json::to_string(receipt).expect("receipt is serializable");
    sqlx::query("INSERT INTO receipts (id, created_at, requester, endpoint, outcome, receipt_json, signature) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(&receipt.id).bind(receipt.created_at.to_rfc3339()).bind(&receipt.requester)
        .bind(&receipt.endpoint).bind(&receipt.outcome).bind(json).bind(signature)
        .execute(pool).await?;
    Ok(())
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Option<StoredReceipt>, sqlx::Error> {
    let row = sqlx::query("SELECT receipt_json, signature FROM receipts WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    row.map(parse_row).transpose()
}

pub async fn list(
    pool: &SqlitePool,
    requester: Option<&str>,
    outcome: Option<&str>,
    limit: u32,
) -> Result<Vec<StoredReceipt>, sqlx::Error> {
    let rows = sqlx::query("SELECT receipt_json, signature FROM receipts WHERE (? IS NULL OR requester LIKE ?) AND (? IS NULL OR outcome = ?) ORDER BY created_at DESC LIMIT ?")
        .bind(requester).bind(requester.map(|v| format!("%{v}%")))
        .bind(outcome).bind(outcome).bind(limit.min(200) as i64).fetch_all(pool).await?;
    rows.into_iter().map(parse_row).collect()
}

fn parse_row(row: sqlx::sqlite::SqliteRow) -> Result<StoredReceipt, sqlx::Error> {
    let json: String = row.try_get("receipt_json")?;
    let receipt = serde_json::from_str(&json).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
    Ok(StoredReceipt {
        receipt,
        signature: row.try_get("signature")?,
    })
}
