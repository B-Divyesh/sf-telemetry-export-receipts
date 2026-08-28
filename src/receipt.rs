use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub schema: String,
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub requester: String,
    pub purpose: String,
    pub endpoint: String,
    pub method: String,
    pub time_range: TimeRange,
    pub row_limit: u32,
    pub fields: Vec<String>,
    pub redaction_policy: String,
    pub query_sha256: String,
    pub policy: PolicySnapshot,
    pub outcome: String,
    pub upstream_status: Option<u16>,
    pub denial_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySnapshot {
    pub max_range_seconds: u64,
    pub max_rows: u32,
    pub allowed_path: bool,
    pub authorization_forwarded: bool,
    pub result_body_recorded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredReceipt {
    #[serde(flatten)]
    pub receipt: Receipt,
    pub signature: String,
}

impl StoredReceipt {
    pub fn markdown(&self) -> String {
        let r = &self.receipt;
        format!(
            "# Export receipt {}\n\n- **Outcome:** {}\n- **Requester:** {}\n- **Created:** {}\n- **Purpose:** {}\n- **Endpoint:** `{} {}`\n- **Range:** {} to {}\n- **Row cap:** {}\n- **Fields:** {}\n- **Redaction:** {}\n- **Query SHA-256:** `{}`\n- **Signature (HMAC-SHA256):** `{}`\n\nResult bodies are not recorded.\n",
            r.id, r.outcome, r.requester, r.created_at.to_rfc3339(), r.purpose, r.method,
            r.endpoint, r.time_range.start.to_rfc3339(), r.time_range.end.to_rfc3339(),
            r.row_limit, r.fields.join(", "), r.redaction_policy, r.query_sha256, self.signature
        )
    }
}

pub fn sign(receipt: &Receipt, key: &str) -> String {
    let bytes = serde_json::to_vec(receipt).expect("receipt is serializable");
    let mut mac = HmacSha256::new_from_slice(key.as_bytes()).expect("HMAC accepts any key size");
    mac.update(&bytes);
    hex::encode(mac.finalize().into_bytes())
}

pub fn verify(receipt: &Receipt, signature: &str, key: &str) -> bool {
    let Ok(bytes) = hex::decode(signature) else {
        return false;
    };
    let mut mac = HmacSha256::new_from_slice(key.as_bytes()).expect("HMAC accepts any key size");
    mac.update(&serde_json::to_vec(receipt).expect("receipt is serializable"));
    mac.verify_slice(&bytes).is_ok()
}

pub fn query_hash(query: &BTreeMap<String, serde_json::Value>) -> String {
    hex::encode(Sha256::digest(
        serde_json::to_vec(query).expect("query is serializable"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_detects_changes() {
        let now = Utc::now();
        let mut receipt = Receipt {
            schema: "ter.v1".into(),
            id: "one".into(),
            created_at: now,
            requester: "sam@example.com".into(),
            purpose: "incident".into(),
            endpoint: "/export".into(),
            method: "POST".into(),
            time_range: TimeRange {
                start: now,
                end: now,
            },
            row_limit: 10,
            fields: vec!["message".into()],
            redaction_policy: "strict".into(),
            query_sha256: "abc".into(),
            policy: PolicySnapshot {
                max_range_seconds: 10,
                max_rows: 10,
                allowed_path: true,
                authorization_forwarded: true,
                result_body_recorded: false,
            },
            outcome: "allowed".into(),
            upstream_status: Some(200),
            denial_reason: None,
        };
        let signature = sign(&receipt, "key");
        assert!(verify(&receipt, &signature, "key"));
        receipt.row_limit = 11;
        assert!(!verify(&receipt, &signature, "key"));
    }
}
