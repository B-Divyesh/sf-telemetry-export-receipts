use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderName, HeaderValue, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::SqlitePool;
use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use uuid::Uuid;

use crate::{
    config::Config,
    db,
    receipt::{self, PolicySnapshot, Receipt, StoredReceipt, TimeRange},
};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: SqlitePool,
    client: reqwest::Client,
    rate: Arc<Mutex<BTreeMap<String, (Instant, u32)>>>,
}

#[derive(Debug, Deserialize)]
pub struct ExportRequest {
    endpoint: String,
    #[serde(default = "post_method")]
    method: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    row_limit: u32,
    fields: Vec<String>,
    redaction_policy: String,
    purpose: String,
    #[serde(default)]
    query: BTreeMap<String, Value>,
}

fn post_method() -> String {
    "POST".into()
}

#[derive(Debug, Deserialize)]
struct ListQuery {
    requester: Option<String>,
    outcome: Option<String>,
    limit: Option<u32>,
}

#[derive(Serialize)]
struct PolicyView {
    configured: bool,
    allowed_paths: Vec<String>,
    max_range_hours: u64,
    max_rows: u32,
    redaction_policies: Vec<String>,
    identity_header: String,
    signing: &'static str,
}

pub async fn build(config: Config) -> Result<Router, sqlx::Error> {
    let db = db::connect(&config.database_url).await?;
    let state = AppState {
        config,
        db,
        client: reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("valid client"),
        rate: Arc::new(Mutex::new(BTreeMap::new())),
    };
    let static_files = ServeDir::new("dist").not_found_service(ServeFile::new("dist/index.html"));
    Ok(Router::new()
        .route("/health", get(health))
        .route("/api/v1/policy", get(policy))
        .route("/api/v1/exports", post(proxy_export))
        .route("/api/v1/receipts", get(list_receipts))
        .route("/api/v1/receipts/{id}", get(get_receipt))
        .route("/api/v1/receipts/{id}/markdown", get(get_receipt_markdown))
        .route("/api/v1/receipts/{id}/verify", get(verify_receipt))
        .fallback_service(static_files)
        .layer(middleware::from_fn(security_headers))
        .layer(TraceLayer::new_for_http())
        .with_state(state))
}

async fn health(State(state): State<AppState>) -> impl IntoResponse {
    Json(json!({"status":"ok", "build_sha": state.config.build_sha}))
}

async fn policy(State(state): State<AppState>) -> impl IntoResponse {
    Json(PolicyView {
        configured: state.config.upstream_base_url.is_some(),
        allowed_paths: state.config.allowed_paths.clone(),
        max_range_hours: state.config.max_range.as_secs() / 3600,
        max_rows: state.config.max_rows,
        redaction_policies: state.config.allowed_redactions.clone(),
        identity_header: state.config.identity_header.clone(),
        signing: "HMAC-SHA256",
    })
}

async fn proxy_export(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(mut request): Json<ExportRequest>,
) -> Response {
    let identity_name = match HeaderName::from_bytes(state.config.identity_header.as_bytes()) {
        Ok(name) => name,
        Err(_) => {
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "invalid_identity_header",
                "The configured identity header is invalid.",
                None,
            )
        }
    };
    let requester = match headers
        .get(identity_name)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        Some(value) if value.len() <= 254 => value.to_owned(),
        _ => {
            return error(
                StatusCode::UNAUTHORIZED,
                "identity_required",
                "A trusted requester identity header is required.",
                None,
            )
        }
    };
    if !take_rate_token(&state, &requester).await {
        return error(
            StatusCode::TOO_MANY_REQUESTS,
            "rate_limited",
            "This requester exceeded 60 export attempts per minute.",
            None,
        );
    }

    request.method = request.method.to_ascii_uppercase();
    let denial = validate(&state.config, &request);
    if let Some(reason) = denial {
        let stored = make_receipt(
            &state,
            &request,
            requester,
            "denied",
            None,
            Some(reason.clone()),
            headers.contains_key(header::AUTHORIZATION),
        )
        .await;
        return match stored {
            Ok(receipt) => error(
                StatusCode::FORBIDDEN,
                "policy_denied",
                &reason,
                Some(receipt.receipt.id),
            ),
            Err(_) => error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "receipt_write_failed",
                "The denied attempt could not be recorded.",
                None,
            ),
        };
    }

    let Some(base) = &state.config.upstream_base_url else {
        return error(
            StatusCode::SERVICE_UNAVAILABLE,
            "upstream_not_configured",
            "Set TER_UPSTREAM_BASE_URL before proxying exports.",
            None,
        );
    };
    request
        .query
        .insert("start_time".into(), json!(request.start));
    request.query.insert("end_time".into(), json!(request.end));
    request
        .query
        .insert("limit".into(), json!(request.row_limit));
    request.query.insert("fields".into(), json!(request.fields));
    request
        .query
        .insert("redaction_policy".into(), json!(request.redaction_policy));

    let method = match reqwest::Method::from_bytes(request.method.as_bytes()) {
        Ok(value) => value,
        Err(_) => {
            return error(
                StatusCode::BAD_REQUEST,
                "invalid_method",
                "Only GET and POST exports are supported.",
                None,
            )
        }
    };
    let mut upstream = state
        .client
        .request(method, format!("{base}{}", request.endpoint));
    for name in [header::AUTHORIZATION, header::COOKIE, header::ACCEPT] {
        if let Some(value) = headers.get(&name).and_then(|v| v.to_str().ok()) {
            upstream = upstream.header(name.as_str(), value);
        }
    }
    upstream = if request.method == "GET" {
        upstream.query(&request.query)
    } else {
        upstream.json(&request.query)
    };
    let result = upstream.send().await;
    match result {
        Ok(response) => {
            let status = response.status();
            let content_type = response.headers().get(header::CONTENT_TYPE).cloned();
            let body = match response.bytes().await {
                Ok(bytes) => bytes,
                Err(_) => {
                    return error(
                        StatusCode::BAD_GATEWAY,
                        "upstream_read_failed",
                        "The upstream response could not be read.",
                        None,
                    )
                }
            };
            let outcome = if status.is_success() {
                "allowed"
            } else {
                "upstream_error"
            };
            match make_receipt(&state, &request, requester, outcome, Some(status.as_u16()), None, headers.contains_key(header::AUTHORIZATION)).await {
                Ok(stored) => {
                    let mut result = Response::builder().status(status)
                        .header("x-export-receipt-id", &stored.receipt.id)
                        .header("x-export-receipt-signature", &stored.signature)
                        .header("cache-control", "no-store");
                    if let Some(value) = content_type { result = result.header(header::CONTENT_TYPE, value); }
                    result.body(Body::from(body)).unwrap()
                }
                Err(_) => error(StatusCode::INTERNAL_SERVER_ERROR, "receipt_write_failed", "The upstream answered, but its mandatory receipt could not be stored; the result was withheld.", None),
            }
        }
        Err(_) => {
            let stored = make_receipt(
                &state,
                &request,
                requester,
                "upstream_error",
                None,
                Some("Upstream connection failed".into()),
                headers.contains_key(header::AUTHORIZATION),
            )
            .await
            .ok();
            error(
                StatusCode::BAD_GATEWAY,
                "upstream_unavailable",
                "The approved upstream could not be reached.",
                stored.map(|v| v.receipt.id),
            )
        }
    }
}

fn validate(config: &Config, request: &ExportRequest) -> Option<String> {
    if request.endpoint.contains("..")
        || !request.endpoint.starts_with('/')
        || !config.allowed_paths.contains(&request.endpoint)
    {
        return Some("Endpoint is not on the export allowlist.".into());
    }
    if request.method != "GET" && request.method != "POST" {
        return Some("Only GET and POST exports are allowed.".into());
    }
    if request.end <= request.start {
        return Some("The end of the time range must be after the start.".into());
    }
    let seconds = (request.end - request.start).num_seconds();
    if seconds > config.max_range.as_secs() as i64 {
        return Some(format!(
            "Time range exceeds the {} hour policy cap.",
            config.max_range.as_secs() / 3600
        ));
    }
    if request.row_limit == 0 || request.row_limit > config.max_rows {
        return Some(format!(
            "Row limit must be between 1 and {}.",
            config.max_rows
        ));
    }
    if request.fields.is_empty()
        || request.fields.len() > 64
        || request
            .fields
            .iter()
            .any(|v| v.trim().is_empty() || v.len() > 128)
    {
        return Some("Declare between 1 and 64 valid fields.".into());
    }
    if !config
        .allowed_redactions
        .contains(&request.redaction_policy)
    {
        return Some("Redaction policy is not approved.".into());
    }
    if request.purpose.trim().len() < 4 || request.purpose.len() > 240 {
        return Some("Purpose must be between 4 and 240 characters.".into());
    }
    None
}

async fn make_receipt(
    state: &AppState,
    request: &ExportRequest,
    requester: String,
    outcome: &str,
    upstream_status: Option<u16>,
    denial_reason: Option<String>,
    authorization_forwarded: bool,
) -> Result<StoredReceipt, sqlx::Error> {
    let receipt = Receipt {
        schema: "telemetry-export-receipt.v1".into(),
        id: Uuid::now_v7().to_string(),
        created_at: Utc::now(),
        requester,
        purpose: request.purpose.trim().into(),
        endpoint: request.endpoint.clone(),
        method: request.method.clone(),
        time_range: TimeRange {
            start: request.start,
            end: request.end,
        },
        row_limit: request.row_limit,
        fields: request.fields.clone(),
        redaction_policy: request.redaction_policy.clone(),
        query_sha256: receipt::query_hash(&request.query),
        policy: PolicySnapshot {
            max_range_seconds: state.config.max_range.as_secs(),
            max_rows: state.config.max_rows,
            allowed_path: state.config.allowed_paths.contains(&request.endpoint),
            authorization_forwarded,
            result_body_recorded: false,
        },
        outcome: outcome.into(),
        upstream_status,
        denial_reason,
    };
    let signature = receipt::sign(&receipt, &state.config.signing_key);
    db::insert(&state.db, &receipt, &signature).await?;
    Ok(StoredReceipt { receipt, signature })
}

async fn list_receipts(State(state): State<AppState>, Query(query): Query<ListQuery>) -> Response {
    match db::list(
        &state.db,
        query.requester.as_deref(),
        query.outcome.as_deref(),
        query.limit.unwrap_or(50),
    )
    .await
    {
        Ok(receipts) => Json(json!({"receipts": receipts})).into_response(),
        Err(_) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "receipt_read_failed",
            "Receipts could not be read.",
            None,
        ),
    }
}

async fn get_receipt(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    match db::get(&state.db, &id).await {
        Ok(Some(receipt)) => Json(receipt).into_response(),
        Ok(None) => error(
            StatusCode::NOT_FOUND,
            "not_found",
            "No receipt has that ID.",
            None,
        ),
        Err(_) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "receipt_read_failed",
            "The receipt could not be read.",
            None,
        ),
    }
}

async fn get_receipt_markdown(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    match db::get(&state.db, &id).await {
        Ok(Some(receipt)) => (
            [
                (header::CONTENT_TYPE, "text/markdown; charset=utf-8"),
                (header::CONTENT_DISPOSITION, "attachment"),
            ],
            receipt.markdown(),
        )
            .into_response(),
        Ok(None) => error(
            StatusCode::NOT_FOUND,
            "not_found",
            "No receipt has that ID.",
            None,
        ),
        Err(_) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "receipt_read_failed",
            "The receipt could not be read.",
            None,
        ),
    }
}

async fn verify_receipt(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    match db::get(&state.db, &id).await {
        Ok(Some(stored)) => Json(json!({"id": id, "valid": receipt::verify(&stored.receipt, &stored.signature, &state.config.signing_key), "algorithm":"HMAC-SHA256"})).into_response(),
        Ok(None) => error(StatusCode::NOT_FOUND, "not_found", "No receipt has that ID.", None),
        Err(_) => error(StatusCode::INTERNAL_SERVER_ERROR, "receipt_read_failed", "The receipt could not be read.", None),
    }
}

async fn take_rate_token(state: &AppState, identity: &str) -> bool {
    let mut rates = state.rate.lock().await;
    let entry = rates.entry(identity.into()).or_insert((Instant::now(), 0));
    if entry.0.elapsed() >= Duration::from_secs(60) {
        *entry = (Instant::now(), 0);
    }
    entry.1 += 1;
    entry.1 <= 60
}

fn error(status: StatusCode, code: &str, message: &str, receipt_id: Option<String>) -> Response {
    (
        status,
        Json(json!({"error": {"code": code, "message": message}, "receipt_id": receipt_id})),
    )
        .into_response()
}

async fn security_headers(request: Request<Body>, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
    headers.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    headers.insert(
        "permissions-policy",
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );
    headers.insert("content-security-policy", HeaderValue::from_static("default-src 'self'; img-src 'self' data:; style-src 'self'; script-src 'self'; connect-src 'self' https://api.sociobot.in; base-uri 'none'; frame-ancestors 'none'; form-action 'self' https://api.sociobot.in"));
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::to_bytes, http::Request};
    use tower::ServiceExt;

    async fn app() -> Router {
        build(Config::test()).await.unwrap()
    }

    #[tokio::test]
    async fn health_reports_build() {
        let response = app()
            .await
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn missing_identity_is_rejected() {
        let body = json!({"endpoint":"/api/logs/export","start":"2026-01-01T00:00:00Z","end":"2026-01-01T00:30:00Z","row_limit":10,"fields":["message"],"redaction_policy":"pii-basic","purpose":"audit review"});
        let response = app()
            .await
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/exports")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn over_bound_attempt_gets_a_signed_receipt() {
        let router = app().await;
        let body = json!({"endpoint":"/api/logs/export","start":"2026-01-01T00:00:00Z","end":"2026-01-01T02:00:00Z","row_limit":10,"fields":["message"],"redaction_policy":"pii-basic","purpose":"audit review"});
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/exports")
                    .header("content-type", "application/json")
                    .header("x-export-user", "sam@example.com")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let bytes = to_bytes(response.into_body(), 64_000).await.unwrap();
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        let id = value["receipt_id"].as_str().unwrap();
        let verify = router
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/receipts/{id}/verify"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(verify.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn allowed_export_forwards_body_and_creates_receipt() {
        let upstream = Router::new().route(
            "/api/logs/export",
            post(|| async { (StatusCode::OK, "timestamp,message\n1,hello") }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, upstream).await.unwrap() });

        let mut config = Config::test();
        config.upstream_base_url = Some(format!("http://{address}"));
        let router = build(config).await.unwrap();
        let body = json!({"endpoint":"/api/logs/export","start":"2026-01-01T00:00:00Z","end":"2026-01-01T00:30:00Z","row_limit":10,"fields":["message"],"redaction_policy":"pii-basic","purpose":"incident review"});
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/exports")
                    .header("content-type", "application/json")
                    .header("authorization", "Bearer upstream-token")
                    .header("x-export-user", "ada@example.com")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let id = response.headers()["x-export-receipt-id"]
            .to_str()
            .unwrap()
            .to_owned();
        let bytes = to_bytes(response.into_body(), 64_000).await.unwrap();
        assert_eq!(&bytes[..], b"timestamp,message\n1,hello");

        let receipt_response = router
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/receipts/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let receipt_bytes = to_bytes(receipt_response.into_body(), 64_000)
            .await
            .unwrap();
        let receipt_json: Value = serde_json::from_slice(&receipt_bytes).unwrap();
        assert_eq!(receipt_json["outcome"], "allowed");
        assert_eq!(receipt_json["policy"]["result_body_recorded"], false);
        assert!(!String::from_utf8_lossy(&receipt_bytes).contains("hello"));
    }
}
