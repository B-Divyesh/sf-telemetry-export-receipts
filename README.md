# Telemetry Export Receipts

A self-hosted egress boundary for observability administrators. It places enforceable time, row, endpoint, field, and redaction declarations in front of approved telemetry export APIs, preserves upstream authorization, and creates a signed JSON/Markdown receipt for every allowed, denied, or upstream-failed attempt.

It does **not** store telemetry, replace your observability product's permissions, inspect result bodies, or provide dashboards for the telemetry itself.

## How it works

1. An administrator supplies `X-TER-Admin-Token`. The service rejects anonymous receipt reads and export attempts.
2. Your auth proxy strips any client-supplied identity header and injects a trusted `X-Export-User` value.
3. The client posts an export envelope to `/api/v1/exports` with its existing upstream `Authorization`/`Cookie` header.
4. This service checks the endpoint, time window, row cap, fields, redaction policy, purpose, and client-IP rate limit.
5. It injects accepted bounds into the upstream GET query or POST JSON body and forwards only to `TER_UPSTREAM_BASE_URL`.
6. It returns the upstream response with receipt ID and signature headers. SQLite stores only receipt metadata and a query digest.

Denied and failed attempts also receive a stored receipt. If receipt persistence fails after an upstream response, the result is withheld rather than creating an unreceipted export.

## Configure

Copy [`.env.example`](.env.example) into your configuration system when you need overrides. Safe local defaults work with only `PORT`; generated secrets are persisted beside the SQLite database.

| Variable | Default | Purpose |
| --- | --- | --- |
| `PORT` | `8080` | HTTP port |
| `DATABASE_URL` | `sqlite:///data/receipts.db?mode=rwc` in the container | Persistent SQLite location |
| `TER_UPSTREAM_BASE_URL` | unset | Fixed, trusted observability origin; required to export |
| `TER_ALLOWED_EXPORT_PATHS` | three example paths | Comma-separated exact path allowlist |
| `TER_MAX_EXPORT_RANGE_HOURS` | `24` | Maximum declared window |
| `TER_MAX_EXPORT_ROWS` | `10000` | Maximum declared row count |
| `TER_ALLOWED_REDACTION_POLICIES` | `pii-basic,strict` | Approved policy labels |
| `TER_IDENTITY_HEADER` | `x-export-user` | Header injected by your trusted auth proxy |
| `TER_RECEIPT_SIGNING_KEY` | generated | Optional HMAC key override (at least 32 characters) |
| `TER_SIGNING_KEY_FILE` | `/data/receipt-signing.key` | First-boot key file when no override is supplied |
| `TER_ADMIN_TOKEN` | generated | Optional administrator-token override (at least 32 characters) |
| `TER_ADMIN_TOKEN_FILE` | `/data/admin-access.key` | First-boot administrator token file |
| `TER_BUILD_SHA` | `development` | Returned by `/health` |

If a secret override is absent, first boot creates a mode-0600 value in its configured file. Mount `/data` as durable storage and back up the database and signing key together. Run exactly one application replica because this product uses SQLite. The factory deployment mounts `/data` and fixes both minimum and maximum replicas to one. Rotate the signing key only with a documented verification transition because old receipts require the old key.

Read `/data/admin-access.key` through your host's protected administration channel. Enter it in the receipt desk, or send it as `X-TER-Admin-Token` to protected APIs. The browser keeps this token in sessionStorage for the current tab only. Do not put it in a URL.

The approved upstream endpoint must document and honor the injected `start_time`, `end_time`, `limit`, `fields`, and `redaction_policy` parameters (as a GET query or top-level POST JSON). Create a narrow adapter endpoint if your vendor uses a different request shape; do not point this service at a broad, arbitrary query route.

## Run locally

Requirements: Node 22+, Rust 1.89+, and SQLite development libraries.

```sh
npm ci
npm run build
cargo run
```

Open <http://localhost:8080>. The generated administrator token is in `data/admin-access.key` for a local run. During frontend-only development, run `npm run dev` and `npm run dev:server` in separate terminals.

Example request:

```sh
curl -X POST http://localhost:8080/api/v1/exports \
  -H 'Authorization: Bearer upstream-token' \
  -H 'X-TER-Admin-Token: your-administrator-token' \
  -H 'X-Export-User: ada@example.com' \
  -H 'Content-Type: application/json' \
  -d '{
    "endpoint":"/api/logs/export",
    "start":"2026-08-28T09:00:00Z",
    "end":"2026-08-28T10:00:00Z",
    "row_limit":5000,
    "fields":["timestamp","service","message"],
    "redaction_policy":"pii-basic",
    "purpose":"INC-204 response",
    "query":{"service":"checkout"}
  }'
```

Receipt APIs:

- `GET /api/v1/receipts?requester=&outcome=&limit=50`
- `GET /api/v1/receipts/:id`
- `GET /api/v1/receipts/:id/markdown`
- `GET /api/v1/receipts/:id/verify`
- `GET /api/v1/policy` and `GET /health`

The application denies receipt and export APIs without its administrator token. Also place the installation behind your administrator SSO or VPN. The outer proxy must strip client-supplied identity headers before injecting its authenticated identity.

## Test and build

```sh
npm test          # Vitest + Rust unit/integration tests
npm run check     # strict TypeScript + Clippy
npm run build     # reproducible frontend output in dist/
npm run test:e2e  # Playwright keyboard, console, routes, and axe checks
docker build -t telemetry-export-receipts .
```

The multi-stage image serves the Vite build and Axum API together on port 8080 as a non-root user. Configure an upstream, mount `/data`, and run one replica. Securely retain `/data/receipt-signing.key` and `/data/admin-access.key`.

## Paid unlock

The free, MIT-licensed core includes policy enforcement, signing, the receipt ledger, individual JSON/Markdown downloads, and signature verification. The optional **Fleet archive** UI unlock is US$49 once and bundles the currently filtered receipt set as JSON. Purchase and license verification use only the Sociobot billing API; no payment provider is embedded here. Accessibility and safety behavior are never gated.

See `/privacy` and `/terms` in a running installation. Sociobot/Dodo is merchant of record and handles refunds.

## Privacy and security notes

- No analytics, ads, third-party fonts, runtime CDNs, or result-body storage.
- HMAC receipt keys stay server-side. Browser code sees only signatures.
- Request bodies are not logged by application code. Configure surrounding proxies accordingly.
- Every API except `/health` is limited by the first `X-Forwarded-For` client IP. Export writes use a stricter bucket.
- The service is single-tenant. Use one replica, database, signing key, and administrator token per boundary.

## License

MIT. See [LICENSE](LICENSE).
