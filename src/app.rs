use axum::{
    body::{to_bytes, Body},
    extract::{Path, Query, State},
    http::{header, HeaderName, HeaderValue, Request, StatusCode},
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
use subtle::ConstantTimeEq;
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
    rate: Arc<Mutex<BTreeMap<String, RateBucket>>>,
}

struct RateBucket {
    tokens: f64,
    updated: Instant,
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

struct ReceiptFailure {
    outcome: &'static str,
    status: StatusCode,
    code: &'static str,
    message: &'static str,
    reason: &'static str,
    upstream_status: Option<u16>,
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
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("valid client"),
        rate: Arc::new(Mutex::new(BTreeMap::new())),
    };
    let index = ServeFile::new("dist/index.html");
    let static_files = ServeDir::new("dist").not_found_service(index.clone());
    let protected_api = Router::new()
        .route("/api/v1/exports", post(proxy_export))
        .route("/api/v1/receipts", get(list_receipts))
        .route("/api/v1/receipts/{id}", get(get_receipt))
        .route("/api/v1/receipts/{id}/markdown", get(get_receipt_markdown))
        .route("/api/v1/receipts/{id}/verify", get(verify_receipt))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_admin));
    let api = Router::new()
        .route("/api/v1/policy", get(policy))
        .merge(protected_api)
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            api_rate_limit,
        ));
    Ok(Router::new()
        .route("/health", get(health))
        .merge(api)
        .route_service("/", index.clone())
        .route_service("/demo", index.clone())
        .route_service("/privacy", index.clone())
        .route_service("/terms", index)
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

async fn proxy_export(State(state): State<AppState>, request: Request<Body>) -> Response {
    // Parse inside the audited route so an identified caller gets a receipt even
    // when its envelope is malformed or too large for this narrow API contract.
    let (parts, body) = request.into_parts();
    let headers = parts.headers;
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
    // The export limiter runs after administrator and requester checks. That
    // lets a rejected, identified export receive the same audit treatment as
    // every other export attempt without consuming its untrusted body.
    if let Some(seconds) = take_rate_token(&state, "export", &headers, 20.0, 5.0).await {
        return record_rate_limited_export(
            &state,
            requester,
            headers.contains_key(header::AUTHORIZATION),
            seconds,
        )
        .await;
    }
    let mut request =
        match to_bytes(body, 256 * 1024).await {
            Ok(body) => match serde_json::from_slice::<ExportRequest>(&body) {
                Ok(request) => request,
                Err(_) => {
                    return record_failure(
                        &state,
                        &unparsed_export_request(),
                        requester,
                        ReceiptFailure {
                            outcome: "denied",
                            status: StatusCode::BAD_REQUEST,
                            code: "invalid_json",
                            message: "The export request must be valid JSON.",
                            reason: "Request body was not valid JSON.",
                            upstream_status: None,
                        },
                        headers.contains_key(header::AUTHORIZATION),
                    )
                    .await
                }
            },
            Err(_) => return record_failure(
                &state,
                &unparsed_export_request(),
                requester,
                ReceiptFailure {
                    outcome: "denied",
                    status: StatusCode::PAYLOAD_TOO_LARGE,
                    code: "invalid_request_body",
                    message: "The export request body could not be read or exceeded 256 KiB.",
                    reason:
                        "Request body could not be read or exceeded the 256 KiB envelope limit.",
                    upstream_status: None,
                },
                headers.contains_key(header::AUTHORIZATION),
            )
            .await,
        };

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
        return record_failure(
            &state,
            &request,
            requester,
            ReceiptFailure {
                outcome: "denied",
                status: StatusCode::SERVICE_UNAVAILABLE,
                code: "upstream_not_configured",
                message: "Set TER_UPSTREAM_BASE_URL before proxying exports.",
                reason: "Approved upstream is not configured.",
                upstream_status: None,
            },
            headers.contains_key(header::AUTHORIZATION),
        )
        .await;
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
        upstream.query(&get_query_pairs(&request.query))
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
                    // Headers prove that the export reached the configured
                    // upstream even if its body fails mid-stream. Keep that
                    // crossing distinguishable from a policy denial.
                    return record_failure(
                        &state,
                        &request,
                        requester,
                        ReceiptFailure {
                            outcome: "upstream_error",
                            status: StatusCode::BAD_GATEWAY,
                            code: "upstream_read_failed",
                            message: "The upstream response could not be read.",
                            reason: "Upstream response body could not be read.",
                            upstream_status: Some(status.as_u16()),
                        },
                        headers.contains_key(header::AUTHORIZATION),
                    )
                    .await;
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
            record_failure(
                &state,
                &request,
                requester,
                ReceiptFailure {
                    outcome: "upstream_error",
                    status: StatusCode::BAD_GATEWAY,
                    code: "upstream_unavailable",
                    message: "The approved upstream could not be reached.",
                    reason: "Upstream connection failed.",
                    upstream_status: None,
                },
                headers.contains_key(header::AUTHORIZATION),
            )
            .await
        }
    }
}

fn unparsed_export_request() -> ExportRequest {
    let now = Utc::now();
    ExportRequest {
        endpoint: "(unparsed request)".into(),
        method: "UNPARSED".into(),
        start: now,
        end: now,
        row_limit: 0,
        fields: Vec::new(),
        redaction_policy: "(unparsed request)".into(),
        purpose: "Unparsed export request".into(),
        query: BTreeMap::new(),
    }
}

fn rate_limited_export_request() -> ExportRequest {
    let now = Utc::now();
    ExportRequest {
        endpoint: "(rate-limited request)".into(),
        method: "RATE_LIMITED".into(),
        start: now,
        end: now,
        row_limit: 0,
        fields: Vec::new(),
        redaction_policy: "(rate-limited request)".into(),
        purpose: "Rate-limited export request".into(),
        query: BTreeMap::new(),
    }
}

async fn record_failure(
    state: &AppState,
    request: &ExportRequest,
    requester: String,
    failure: ReceiptFailure,
    authorization_forwarded: bool,
) -> Response {
    match make_receipt(
        state,
        request,
        requester,
        failure.outcome,
        failure.upstream_status,
        Some(failure.reason.into()),
        authorization_forwarded,
    )
    .await
    {
        Ok(stored) => error(
            failure.status,
            failure.code,
            failure.message,
            Some(stored.receipt.id),
        ),
        Err(_) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "receipt_write_failed",
            "The export attempt could not be recorded.",
            None,
        ),
    }
}

async fn record_rate_limited_export(
    state: &AppState,
    requester: String,
    authorization_forwarded: bool,
    retry_after: u64,
) -> Response {
    let mut response = record_failure(
        state,
        &rate_limited_export_request(),
        requester,
        ReceiptFailure {
            outcome: "denied",
            status: StatusCode::TOO_MANY_REQUESTS,
            code: "rate_limited",
            message: "Too many export requests from this client. Retry after the stated delay.",
            reason: "Export rate limit exceeded before the request envelope was read.",
            upstream_status: None,
        },
        authorization_forwarded,
    )
    .await;
    response.headers_mut().insert(
        header::RETRY_AFTER,
        HeaderValue::from_str(&retry_after.to_string()).expect("valid retry delay"),
    );
    response
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

fn get_query_pairs(query: &BTreeMap<String, Value>) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    for (key, value) in query {
        match value {
            Value::Null => {}
            Value::Array(values) => {
                for value in values {
                    pairs.push((key.clone(), query_value(value)));
                }
            }
            value => pairs.push((key.clone(), query_value(value))),
        }
    }
    pairs
}

fn query_value(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Number(value) => value.to_string(),
        Value::Bool(value) => value.to_string(),
        value => serde_json::to_string(value).expect("JSON value is serializable"),
    }
}

async fn require_admin(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let supplied = request
        .headers()
        .get("x-ter-admin-token")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let expected = state.config.admin_token.as_bytes();
    let accepted =
        supplied.len() == expected.len() && bool::from(supplied.as_bytes().ct_eq(expected));
    if !accepted {
        // Auth failures are not attributable exports, but they still need a
        // request allowance so this route cannot bypass the API-wide limiter.
        if request.uri().path() == "/api/v1/exports" {
            if let Some(seconds) = take_rate_token(
                &state,
                "unauthenticated-export",
                request.headers(),
                20.0,
                5.0,
            )
            .await
            {
                let mut response = error(
                    StatusCode::TOO_MANY_REQUESTS,
                    "rate_limited",
                    "Too many requests from this client. Retry after the stated delay.",
                    None,
                );
                response.headers_mut().insert(
                    header::RETRY_AFTER,
                    HeaderValue::from_str(&seconds.to_string()).expect("valid retry delay"),
                );
                return response;
            }
        }
        return error(
            StatusCode::UNAUTHORIZED,
            "admin_access_required",
            "Supply the administrator access token in X-TER-Admin-Token.",
            None,
        );
    }
    next.run(request).await
}

async fn api_rate_limit(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if request.uri().path() == "/api/v1/exports" {
        return next.run(request).await;
    }
    let retry_after = take_rate_token(&state, "read", request.headers(), 40.0, 20.0).await;
    if let Some(seconds) = retry_after {
        let mut response = error(
            StatusCode::TOO_MANY_REQUESTS,
            "rate_limited",
            "Too many requests from this client. Retry after the stated delay.",
            None,
        );
        response.headers_mut().insert(
            header::RETRY_AFTER,
            HeaderValue::from_str(&seconds.to_string()).expect("valid retry delay"),
        );
        return response;
    }
    next.run(request).await
}

async fn take_rate_token(
    state: &AppState,
    class: &str,
    headers: &axum::http::HeaderMap,
    capacity: f64,
    refill_per_second: f64,
) -> Option<u64> {
    let client = first_forwarded_for(headers);
    let key = format!("{class}:{client}");
    let mut rates = state.rate.lock().await;
    let now = Instant::now();
    let bucket = rates.entry(key).or_insert(RateBucket {
        tokens: capacity,
        updated: now,
    });
    let elapsed = now.duration_since(bucket.updated).as_secs_f64();
    bucket.tokens = (bucket.tokens + elapsed * refill_per_second).min(capacity);
    bucket.updated = now;
    if bucket.tokens >= 1.0 {
        bucket.tokens -= 1.0;
        None
    } else {
        Some(((1.0 - bucket.tokens) / refill_per_second).ceil().max(1.0) as u64)
    }
}

fn first_forwarded_for(headers: &axum::http::HeaderMap) -> String {
    let candidate = headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty() && value.len() <= 128)
        .unwrap_or_default();
    candidate
        .parse::<std::net::IpAddr>()
        .map(|ip| ip.to_string())
        .or_else(|_| {
            candidate
                .parse::<std::net::SocketAddr>()
                .map(|address| address.ip().to_string())
        })
        .unwrap_or_else(|_| "unknown".into())
}

fn error(status: StatusCode, code: &str, message: &str, receipt_id: Option<String>) -> Response {
    (
        status,
        Json(json!({"error": {"code": code, "message": message}, "receipt_id": receipt_id})),
    )
        .into_response()
}

async fn security_headers(request: Request<Body>, next: Next) -> Response {
    let path = request.uri().path().to_owned();
    let mut response = next.run(request).await;
    let is_known_page = matches!(path.as_str(), "/" | "/demo" | "/privacy" | "/terms");
    let is_html = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("text/html"));
    if is_html && !is_known_page {
        *response.status_mut() = StatusCode::NOT_FOUND;
    }
    let headers = response.headers_mut();
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
    headers.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    headers.insert(
        "strict-transport-security",
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );
    headers.insert(
        "permissions-policy",
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );
    headers.insert("content-security-policy", HeaderValue::from_static("default-src 'self'; img-src 'self' data:; style-src 'self'; script-src 'self'; connect-src 'self' https://api.sociobot.in; base-uri 'none'; frame-ancestors 'none'; form-action 'self' https://api.sociobot.in"));
    if !headers.contains_key(header::CACHE_CONTROL) {
        let value = if path.starts_with("/api/") || path == "/health" {
            "no-store"
        } else if path.starts_with("/assets/index-") {
            "public, max-age=31536000, immutable"
        } else if path.starts_with("/assets/") || path == "/favicon.svg" {
            "public, max-age=86400"
        } else {
            "no-cache"
        };
        headers.insert(header::CACHE_CONTROL, HeaderValue::from_static(value));
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::to_bytes,
        http::{HeaderMap, Request},
    };
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
                    .header("x-ter-admin-token", "test-admin-token")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn forged_identity_without_admin_access_is_rejected() {
        let body = json!({"endpoint":"/api/logs/export","start":"2026-01-01T00:00:00Z","end":"2026-01-01T00:30:00Z","row_limit":10,"fields":["message"],"redaction_policy":"pii-basic","purpose":"audit review"});
        let response = app()
            .await
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/exports")
                    .header("content-type", "application/json")
                    .header("x-export-user", "forged@example.com")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 64_000).await.unwrap()).unwrap();
        assert_eq!(value["error"]["code"], "admin_access_required");
    }

    #[tokio::test]
    async fn anonymous_receipt_reads_are_rejected() {
        let response = app()
            .await
            .oneshot(
                Request::builder()
                    .uri("/api/v1/receipts")
                    .body(Body::empty())
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
                    .header("x-ter-admin-token", "test-admin-token")
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
                    .header("x-ter-admin-token", "test-admin-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(verify.status(), StatusCode::OK);
    }

    // @claim:signed-downloads
    #[tokio::test]
    async fn claim_receipts_are_signed_and_downloadable_as_json_and_markdown() {
        let router = app().await;
        let body = json!({"endpoint":"/api/logs/export","start":"2026-01-01T00:00:00Z","end":"2026-01-01T02:00:00Z","row_limit":10,"fields":["message"],"redaction_policy":"pii-basic","purpose":"audit review"});
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/exports")
                    .header("x-ter-admin-token", "test-admin-token")
                    .header("x-export-user", "download@example.com")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let response: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 64_000).await.unwrap()).unwrap();
        let receipt_id = response["receipt_id"].as_str().unwrap();
        let json_download = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/receipts/{receipt_id}"))
                    .header("x-ter-admin-token", "test-admin-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(json_download.status(), StatusCode::OK);
        let json_download: Value =
            serde_json::from_slice(&to_bytes(json_download.into_body(), 64_000).await.unwrap())
                .unwrap();
        let signature = json_download["signature"].as_str().expect("signature");
        assert_eq!(json_download["id"], receipt_id);
        let markdown_download = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/receipts/{receipt_id}/markdown"))
                    .header("x-ter-admin-token", "test-admin-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(markdown_download.status(), StatusCode::OK);
        assert_eq!(
            markdown_download.headers()[header::CONTENT_TYPE],
            "text/markdown; charset=utf-8"
        );
        let markdown = String::from_utf8(
            to_bytes(markdown_download.into_body(), 64_000)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(markdown.contains(receipt_id));
        assert!(markdown.contains(signature));
        let verification = router
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/receipts/{receipt_id}/verify"))
                    .header("x-ter-admin-token", "test-admin-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let verification: Value =
            serde_json::from_slice(&to_bytes(verification.into_body(), 64_000).await.unwrap())
                .unwrap();
        assert_eq!(verification["valid"], true);
    }

    // @claim:privacy-forwarding
    #[tokio::test]
    async fn claim_allowed_exports_forward_only_permitted_headers_and_store_no_result_data() {
        let captured = Arc::new(Mutex::new(None));
        let upstream_headers = Arc::clone(&captured);
        let upstream = Router::new().route(
            "/api/logs/export",
            post(move |headers: HeaderMap| {
                let upstream_headers = Arc::clone(&upstream_headers);
                async move {
                    *upstream_headers.lock().await = Some(headers);
                    (StatusCode::OK, "timestamp,message\n1,private-upstream-row")
                }
            }),
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
                    .header("x-ter-admin-token", "test-admin-token")
                    .header("content-type", "application/json")
                    .header("authorization", "Bearer upstream-token")
                    .header("cookie", "session=upstream-cookie")
                    .header("accept", "text/csv")
                    .header("x-export-user", "ada@example.com")
                    .header("x-unapproved-client-header", "must-not-forward")
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
        assert_eq!(&bytes[..], b"timestamp,message\n1,private-upstream-row");
        let upstream_headers = captured.lock().await.take().expect("upstream request");
        assert_eq!(
            upstream_headers[header::AUTHORIZATION],
            "Bearer upstream-token"
        );
        assert_eq!(upstream_headers[header::COOKIE], "session=upstream-cookie");
        assert_eq!(upstream_headers[header::ACCEPT], "text/csv");
        assert!(!upstream_headers.contains_key("x-export-user"));
        assert!(!upstream_headers.contains_key("x-ter-admin-token"));
        assert!(!upstream_headers.contains_key("x-unapproved-client-header"));

        let receipt_response = router
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/receipts/{id}"))
                    .header("x-ter-admin-token", "test-admin-token")
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
        let receipt_text = String::from_utf8_lossy(&receipt_bytes);
        assert!(!receipt_text.contains("private-upstream-row"));
        assert!(!receipt_text.contains("upstream-token"));
        assert!(!receipt_text.contains("upstream-cookie"));
    }

    // @claim:recorded-exports
    #[tokio::test]
    async fn claim_allowed_denied_and_upstream_failed_exports_have_signed_receipts() {
        let upstream = Router::new().route(
            "/api/logs/export",
            post(|Json(payload): Json<Value>| async move {
                if payload["scenario"] == "failure" {
                    (StatusCode::SERVICE_UNAVAILABLE, "upstream unavailable")
                } else {
                    (StatusCode::OK, "timestamp,message\n1,ok")
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, upstream).await.unwrap() });

        let mut config = Config::test();
        config.upstream_base_url = Some(format!("http://{address}"));
        let router = build(config).await.unwrap();
        let cases = [
            (
                "allowed",
                StatusCode::OK,
                json!({"endpoint":"/api/logs/export","start":"2026-01-01T00:00:00Z","end":"2026-01-01T00:30:00Z","row_limit":10,"fields":["message"],"redaction_policy":"pii-basic","purpose":"incident review","query":{"scenario":"allowed"}}),
            ),
            (
                "denied",
                StatusCode::FORBIDDEN,
                json!({"endpoint":"/api/logs/export","start":"2026-01-01T00:00:00Z","end":"2026-01-01T02:00:00Z","row_limit":10,"fields":["message"],"redaction_policy":"pii-basic","purpose":"incident review"}),
            ),
            (
                "upstream_error",
                StatusCode::SERVICE_UNAVAILABLE,
                json!({"endpoint":"/api/logs/export","start":"2026-01-01T00:00:00Z","end":"2026-01-01T00:30:00Z","row_limit":10,"fields":["message"],"redaction_policy":"pii-basic","purpose":"incident review","query":{"scenario":"failure"}}),
            ),
        ];

        for (outcome, status, body) in cases {
            let response = router
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/v1/exports")
                        .header("content-type", "application/json")
                        .header("x-ter-admin-token", "test-admin-token")
                        .header("x-export-user", "audit@example.com")
                        .header("x-forwarded-for", "203.0.113.56")
                        .body(Body::from(body.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), status);
            let receipt_id = if let Some(id) = response.headers().get("x-export-receipt-id") {
                id.to_str().unwrap().to_owned()
            } else {
                let body = to_bytes(response.into_body(), 64_000).await.unwrap();
                serde_json::from_slice::<Value>(&body).unwrap()["receipt_id"]
                    .as_str()
                    .unwrap()
                    .to_owned()
            };
            let receipt = router
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(format!("/api/v1/receipts/{receipt_id}"))
                        .header("x-ter-admin-token", "test-admin-token")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            let receipt: Value =
                serde_json::from_slice(&to_bytes(receipt.into_body(), 64_000).await.unwrap())
                    .unwrap();
            assert_eq!(receipt["outcome"], outcome);
            let verification = router
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(format!("/api/v1/receipts/{receipt_id}/verify"))
                        .header("x-ter-admin-token", "test-admin-token")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            let verification: Value =
                serde_json::from_slice(&to_bytes(verification.into_body(), 64_000).await.unwrap())
                    .unwrap();
            assert_eq!(verification["valid"], true);
        }
    }

    // @claim:bounded-get-export
    #[tokio::test]
    async fn claim_get_export_repeats_array_fields_and_reaches_upstream() {
        use axum::http::Uri;

        let upstream = Router::new().route(
            "/api/logs/export",
            get(|uri: Uri| async move { (StatusCode::OK, uri.query().unwrap_or("").to_owned()) }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, upstream).await.unwrap() });

        let mut config = Config::test();
        config.upstream_base_url = Some(format!("http://{address}"));
        config.max_range = Duration::from_secs(24 * 3600);
        config.max_rows = 10_000;
        let router = build(config).await.unwrap();
        let body = json!({
            "endpoint":"/api/logs/export",
            "method":"GET",
            "start":"2026-01-01T00:00:00Z",
            "end":"2026-01-02T00:00:00Z",
            "row_limit":10000,
            "fields":["timestamp","message"],
            "redaction_policy":"pii-basic",
            "purpose":"incident review",
            "query":{"service":"checkout"}
        });
        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/exports")
                    .header("content-type", "application/json")
                    .header("x-ter-admin-token", "test-admin-token")
                    .header("x-export-user", "ada@example.com")
                    .header("x-forwarded-for", "198.51.100.7, 10.0.0.4")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().contains_key("x-export-receipt-id"));
        let query = String::from_utf8(
            to_bytes(response.into_body(), 64_000)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        let parts: Vec<&str> = query.split('&').collect();
        assert!(parts.contains(&"fields=timestamp"));
        assert!(parts.contains(&"fields=message"));
        assert!(parts.contains(&"limit=10000"));
        assert!(parts.contains(&"service=checkout"));
    }

    #[tokio::test]
    async fn api_rate_limit_uses_forwarded_ip_and_sets_retry_after() {
        let router = app().await;
        for _ in 0..40 {
            let response = router
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/api/v1/policy")
                        .header("x-forwarded-for", "203.0.113.10, 10.0.0.2")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }
        let limited = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/policy")
                    .header("x-forwarded-for", "203.0.113.10, 192.0.2.99")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(limited.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(limited.headers()[header::RETRY_AFTER], "1");

        let other_client = router
            .oneshot(
                Request::builder()
                    .uri("/api/v1/policy")
                    .header("x-forwarded-for", "203.0.113.11")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(other_client.status(), StatusCode::OK);
    }

    // @claim:api-rate-limit
    #[tokio::test]
    async fn claim_api_rate_limit_uses_client_address_and_receipts_for_exports() {
        let router = app().await;
        // Every protected read route shares the same client-address bucket and
        // exposes a retry delay once that route's allowance is exhausted.
        for (index, route) in [
            "/api/v1/policy",
            "/api/v1/receipts",
            "/api/v1/receipts/missing",
            "/api/v1/receipts/missing/markdown",
            "/api/v1/receipts/missing/verify",
        ]
        .iter()
        .enumerate()
        {
            let client = format!("198.51.100.{}", index + 20);
            for _ in 0..40 {
                let response = router
                    .clone()
                    .oneshot(
                        Request::builder()
                            .uri(*route)
                            .header("x-ter-admin-token", "test-admin-token")
                            .header("x-forwarded-for", &client)
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();
                assert_ne!(response.status(), StatusCode::TOO_MANY_REQUESTS);
            }
            let limited_read = router
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(*route)
                        .header("x-ter-admin-token", "test-admin-token")
                        .header("x-forwarded-for", &client)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(limited_read.status(), StatusCode::TOO_MANY_REQUESTS);
            assert_eq!(limited_read.headers()[header::RETRY_AFTER], "1");
        }
        for _ in 0..20 {
            let response = router
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/v1/exports")
                        .header("x-forwarded-for", "198.51.100.99")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        }
        let anonymous_limited = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/exports")
                    .header("x-forwarded-for", "198.51.100.99")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(anonymous_limited.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(anonymous_limited.headers()[header::RETRY_AFTER], "1");
        let body = json!({"endpoint":"/api/logs/export","start":"2026-01-01T00:00:00Z","end":"2026-01-01T02:00:00Z","row_limit":10,"fields":["message"],"redaction_policy":"pii-basic","purpose":"audit review"});
        for index in 0..20 {
            let response = router
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/v1/exports")
                        .header("content-type", "application/json")
                        .header("x-ter-admin-token", "test-admin-token")
                        .header("x-export-user", format!("user-{index}@example.com"))
                        .header("x-forwarded-for", "192.0.2.8")
                        .body(Body::from(body.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::FORBIDDEN);
        }
        let limited = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/exports")
                    .header("content-type", "application/json")
                    .header("x-ter-admin-token", "test-admin-token")
                    .header("x-export-user", "another-name@example.com")
                    .header("x-forwarded-for", "192.0.2.8")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(limited.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(limited.headers()[header::RETRY_AFTER], "1");
        let limited_json: Value =
            serde_json::from_slice(&to_bytes(limited.into_body(), 64_000).await.unwrap()).unwrap();
        let receipt_id = limited_json["receipt_id"]
            .as_str()
            .expect("the rate-limited export has a receipt");
        let verification = router
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/receipts/{receipt_id}/verify"))
                    .header("x-ter-admin-token", "test-admin-token")
                    .header("x-forwarded-for", "192.0.2.8")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(verification.status(), StatusCode::OK);
        let verification: Value =
            serde_json::from_slice(&to_bytes(verification.into_body(), 64_000).await.unwrap())
                .unwrap();
        assert_eq!(verification["valid"], true);
    }

    #[tokio::test]
    async fn shared_sqlite_boundary_keeps_receipts_visible_across_instances() {
        let path = std::env::temp_dir().join(format!("ter-{}.db", Uuid::new_v4()));
        let mut config = Config::test();
        config.database_url = format!("sqlite://{}?mode=rwc", path.display());
        let first = build(config.clone()).await.unwrap();
        let second = build(config).await.unwrap();
        let body = json!({"endpoint":"/api/logs/export","start":"2026-01-01T00:00:00Z","end":"2026-01-01T02:00:00Z","row_limit":10,"fields":["message"],"redaction_policy":"pii-basic","purpose":"audit review"});
        let response = first
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/exports")
                    .header("content-type", "application/json")
                    .header("x-ter-admin-token", "test-admin-token")
                    .header("x-export-user", "sam@example.com")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 64_000).await.unwrap()).unwrap();
        let id = value["receipt_id"].as_str().unwrap();

        let response = second
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/receipts/{id}/verify"))
                    .header("x-ter-admin-token", "test-admin-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 64_000).await.unwrap()).unwrap();
        assert_eq!(value["valid"], true);
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn malformed_json_from_an_identified_requester_gets_a_signed_receipt() {
        let router = app().await;
        let malformed_body = "{ definitely not JSON; private-token";
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/exports")
                    .header("x-ter-admin-token", "test-admin-token")
                    .header("content-type", "application/json")
                    .header("x-export-user", "malformed@example.com")
                    .body(Body::from(malformed_body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 64_000).await.unwrap()).unwrap();
        assert_eq!(value["error"]["code"], "invalid_json");
        let id = value["receipt_id"].as_str().expect("receipt id");

        let receipt_response = router
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/receipts/{id}"))
                    .header("x-ter-admin-token", "test-admin-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let receipt_bytes = to_bytes(receipt_response.into_body(), 64_000)
            .await
            .unwrap();
        let receipt: Value = serde_json::from_slice(&receipt_bytes).unwrap();
        assert_eq!(receipt["requester"], "malformed@example.com");
        assert_eq!(receipt["outcome"], "denied");
        assert_eq!(receipt["denial_reason"], "Request body was not valid JSON.");
        assert!(!String::from_utf8_lossy(&receipt_bytes).contains("private-token"));
    }

    #[tokio::test]
    async fn truncated_upstream_response_gets_a_signed_failure_receipt() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let requests = Arc::new(AtomicUsize::new(0));
        let upstream_requests = Arc::clone(&requests);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = [0_u8; 4096];
            let _ = stream.read(&mut request).await.unwrap();
            upstream_requests.fetch_add(1, Ordering::SeqCst);
            // Deliberately advertise more bytes than are sent, then close the
            // connection. This is the same mid-body peer failure observed by
            // the independent verifier and cannot be represented by Axum.
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/csv\r\nContent-Length: 100\r\nConnection: close\r\n\r\npartial-export")
                .await
                .unwrap();
            stream.shutdown().await.unwrap();
        });

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
                    .header("x-ter-admin-token", "test-admin-token")
                    .header("content-type", "application/json")
                    .header("x-export-user", "partial@example.com")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
        assert_eq!(
            requests.load(Ordering::SeqCst),
            1,
            "upstream received the export"
        );
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 64_000).await.unwrap()).unwrap();
        assert_eq!(value["error"]["code"], "upstream_read_failed");
        let id = value["receipt_id"].as_str().expect("receipt id");

        let receipt_response = router
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/receipts/{id}"))
                    .header("x-ter-admin-token", "test-admin-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let receipt: Value = serde_json::from_slice(
            &to_bytes(receipt_response.into_body(), 64_000)
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(receipt["requester"], "partial@example.com");
        assert_eq!(receipt["outcome"], "upstream_error");
        assert_eq!(receipt["upstream_status"], 200);
        assert_eq!(
            receipt["denial_reason"],
            "Upstream response body could not be read."
        );
    }
}
