# Telemetry Export Receipts

A self-hosted egress boundary for observability administrators. It places enforceable time, row, endpoint, field, and redaction declarations in front of approved telemetry export APIs, preserves upstream authorization, and creates a signed JSON/Markdown receipt for every allowed, denied, or upstream-failed attempt.

It does **not** store telemetry, replace your observability product's permissions, inspect result bodies, or provide dashboards for the telemetry itself.

## How it works

1. Your existing auth proxy authenticates the operator, strips any client-supplied identity header, and injects a trusted `X-Export-User` value.
2. The client posts an export envelope to `/api/v1/exports` with its existing upstream `Authorization`/`Cookie` header.
3. This service checks the exact endpoint allowlist, time window, row cap, fields, redaction policy, purpose, and per-identity rate limit.
4. It injects the accepted bounds into the upstream GET query or POST JSON body and forwards the request only to `UPSTREAM_BASE_URL`.
5. It returns the upstream response with `X-Export-Receipt-Id` and `X-Export-Receipt-Signature`. Only receipt metadata and a SHA-256 query digest are stored in SQLite.

Denied and failed attempts also receive a stored receipt. If receipt persistence fails after an upstream response, the result is withheld rather than creating an unreceipted export.

## Configure

Copy [`.env.example`](.env.example) into your secret/configuration system. All configuration is environment-only.

| Variable | Default | Purpose |
| --- | --- | --- |
| `PORT` | `8080` | HTTP port |
| `DATABASE_URL` | `sqlite://data/receipts.db?mode=rwc` | Persistent SQLite location |
| `TER_UPSTREAM_BASE_URL` | unset | Fixed, trusted observability origin; required to export |
| `TER_ALLOWED_EXPORT_PATHS` | three example paths | Comma-separated exact path allowlist |
| `TER_MAX_EXPORT_RANGE_HOURS` | `24` | Maximum declared window |
| `TER_MAX_EXPORT_ROWS` | `10000` | Maximum declared row count |
| `TER_ALLOWED_REDACTION_POLICIES` | `pii-basic,strict` | Approved policy labels |
| `TER_IDENTITY_HEADER` | `x-export-user` | Header injected by your trusted auth proxy |
| `TER_RECEIPT_SIGNING_KEY` | development-only value | HMAC key; mandatory when `TER_APP_ENV=production` |
| `TER_BUILD_SHA` | `development` | Returned by `/health` |

Generate a signing key with `openssl rand -hex 32`. Mount `data/` on persistent storage, restrict it to the service user, and back it up according to your audit retention policy. Rotate the key only with a documented verification transition: old HMAC receipts require the old key.

The approved upstream endpoint must document and honor the injected `start_time`, `end_time`, `limit`, `fields`, and `redaction_policy` parameters (as a GET query or top-level POST JSON). Create a narrow adapter endpoint if your vendor uses a different request shape; do not point this service at a broad, arbitrary query route.

## Run locally

Requirements: Node 22+, Rust 1.85+, and SQLite development libraries.

```sh
npm install
npm run build
TER_RECEIPT_SIGNING_KEY=local-secret cargo run
```

Open <http://localhost:8080>. During frontend-only development, run `npm run dev` and `npm run dev:server` in separate terminals.

Example request:

```sh
curl -X POST http://localhost:8080/api/v1/exports \
  -H 'Authorization: Bearer upstream-token' \
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

Deploy the full UI and receipt APIs behind your administrator SSO/VPN. This application deliberately does not replace upstream or perimeter authentication.

## Test and build

```sh
npm test          # Vitest + Rust unit/integration tests
npm run check     # strict TypeScript + Clippy
npm run build     # reproducible frontend output in dist/
npm run test:e2e  # Playwright keyboard, console, routes, and axe checks
docker build -t telemetry-export-receipts .
```

The multi-stage image serves the Vite build and Axum API together on port 8080 as a non-root user. In production, set `TER_APP_ENV=production`, provide `TER_RECEIPT_SIGNING_KEY`, configure an upstream, and mount `/app/data`.

## Paid unlock

The free, MIT-licensed core includes policy enforcement, signing, the receipt ledger, individual JSON/Markdown downloads, and signature verification. The optional **Fleet archive** UI unlock is US$49 once and bundles the currently filtered receipt set as JSON. Purchase and license verification use only the Sociobot billing API; no payment provider is embedded here. Accessibility and safety behavior are never gated.

See `/privacy` and `/terms` in a running installation. Sociobot/Dodo is merchant of record and handles refunds.

## Privacy and security notes

- No analytics, ads, third-party fonts, runtime CDNs, or result-body storage.
- HMAC receipt keys stay server-side. Browser code sees only signatures.
- Request bodies are not logged by application code. Configure surrounding proxies accordingly.
- Export attempts are limited to 60 per requester per process per minute.
- The service is single-tenant; use one database and signing key per administrative boundary.

## License

MIT. See [LICENSE](LICENSE).
