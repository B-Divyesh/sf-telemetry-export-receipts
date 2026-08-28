# Telemetry Export Receipts — build handoff

## What shipped

- A Rust/Axum reverse proxy for exact, administrator-approved export endpoints. It enforces requester identity, GET/POST only, time-window and row bounds, declared fields, approved redaction labels, purpose, and a 60-attempt/minute identity limit before contacting the upstream.
- Existing `Authorization`, `Cookie`, and `Accept` values are forwarded to the fixed configured origin. Redirect following is disabled, so credentials cannot leave that origin through an upstream redirect.
- SQLite receipts for allowed, policy-denied, and upstream-failed attempts. Receipts contain a versioned policy snapshot, requester, purpose, bounds, fields, SHA-256 query digest, outcome, and HMAC-SHA256 signature. Result bodies and credentials are never written. If persistence fails, a successful upstream result is withheld.
- JSON, Markdown download, signature verification, filtered ledger, policy, and health endpoints.
- A responsive operator UI with the product-specific “night-market proof desk” system, original generated gate illustration, live policy board, loading/empty/error/offline states, filterable and keyboard-expandable receipts, integration example, and legal pages.
- The optional US$49 one-time Fleet archive unlock: hosted Sociobot checkout, return-token capture, local token/verdict storage, once-daily verification, offline cached unlock, restore field, revoked-license handling, and filtered JSON packaging. Core export, receipt, safety, and accessibility behavior remain free.
- Multi-stage container packaging, non-root runtime, persistent `/app/data` volume with a mode-0600 generated signing key on first boot, structured JSON logs, secure response headers, no runtime CDN/fonts/analytics, and graceful shutdown.

## Run and deploy

```sh
npm install
npm run build
TER_RECEIPT_SIGNING_KEY=local-secret cargo run
```

Production configuration is documented in `.env.example` and `README.md`. The container build command is `docker build -t telemetry-export-receipts .`; it serves the frontend and API together on `PORT=8080`. Supply `TER_RECEIPT_SIGNING_KEY` or retain the key generated on first boot. Mount `/app/data` and place the service behind the administrator auth proxy/SSO that strips and injects `TER_IDENTITY_HEADER`.

## Verification performed

- `npm test`: passed (1 frontend unit test, 5 Rust unit/integration tests). The integration test uses a real local Axum upstream, asserts body passthrough, fetches the receipt, and confirms result content is absent.
- `npm run check`: strict TypeScript and Clippy with warnings denied passed.
- `npm run build`: passed; `dist/index.html` is at the required root. Initial JS is 18.13 KB raw / 6.89 KB gzip; CSS is 15.23 KB raw / 4.44 KB gzip; hero WebP is 61 KB desktop and 26 KB mobile.
- `npm run test:e2e`: 2 Playwright tests passed in Chromium, including keyboard skip-link behavior, empty state, legal routes, console capture, and axe serious/critical checks.
- Factory `verify-url.sh`: HTTP 200, title present, `lang=en`, exactly one h1, main landmark present, all images have alt text, and no browser console/page errors.
- Lighthouse 12.8.2 mobile: Performance 100, Accessibility 100, Best Practices 100, SEO 100; LCP 1.5 s, CLS 0, total blocking time 30 ms.
- Load smoke: 250 concurrent `/health` requests completed successfully in 1.062 s (~235 requests/s from this development container).
- `cargo build --release --locked`: passed. A production-mode boot smoke generated a 64-character mode-0600 key in a temporary volume and returned healthy on port 8099. The Dockerfile was inspected but a Docker/Podman daemon was not available in the worker image.
- Visual screenshots were reviewed at desktop and 390 × 844 px. No text artifacts, brands, or unintended symbols were present in the generated illustration.

## Known gaps / next steps

- Upstream export products use different payload shapes. This v1 defines an explicit narrow adapter contract: the approved endpoint must honor top-level `start_time`, `end_time`, `limit`, `fields`, and `redaction_policy`. Add a product-specific adapter endpoint when the upstream does not support that shape; do not approve a broad arbitrary-query endpoint.
- Receipt reads rely on the deployment’s administrator SSO/VPN perimeter, by design; this service preserves rather than replaces product authentication. Verify the gateway strips client-supplied identity headers.
- Rate-limit counters are process-local. Multi-replica deployments should add a shared limiter at the ingress.
- HMAC verification requires the installation key. Plan and document key rotation if long-lived receipt verification spans rotations.
- The live checkout verification path is contract-tested in code but was not exercised with a paid license in this disposable worker.

## Asset provenance

The hero scene was generated on 2026-08-28 with the factory Azure OpenAI image deployment using the prompt recorded in `.factory/design.md`. Source and prompt sidecar are in `assets/src/`; optimized WebP/JPEG derivatives are in `frontend/public/assets/`. The favicon and interface icons are original hand-authored SVG geometry.
