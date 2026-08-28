# Telemetry Export Receipts — repair handoff

## Status: PASS — deployed

This repair addresses every release-blocking finding in independent verification report `f64e9361bae84b61b53c3084fdf70141907ed025` for candidate `bfe4d0c2294e3afe35f4757b61fc98ee00b800bc` while retaining the Rust/Axum + SQLite container and Vite/TypeScript frontend artifact.

## Repairs

- An identified request is now parsed inside the export route, allowing malformed JSON and unreadable/oversize envelopes to produce a signed, persisted denied-attempt receipt instead of being rejected before the audit boundary.
- A failed upstream response-body read now persists an `upstream_error` receipt before returning `502`. It records the received upstream status (including `200` for a truncated success response), never the body or credentials. Upstream connection failures use the same mandatory persistence path.
- The visible `TER.` wordmark is included verbatim in the brand link accessible name: `TER. — Telemetry Export Receipts home`.

## Regression coverage

- `truncated_upstream_response_gets_a_signed_failure_receipt` uses a raw TCP peer that sends `200`, declares `Content-Length: 100`, writes `partial-export`, and closes. It proves the peer received exactly one export request, the proxy returns `502 upstream_read_failed` with a receipt ID, and the persisted signed receipt has requester `partial@example.com`, outcome `upstream_error`, upstream status `200`, and no result body.
- `malformed_json_from_an_identified_requester_gets_a_signed_receipt` proves invalid JSON is receipted without persisting the submitted secret-like body string.
- The Playwright dashboard test asserts the exact accessible brand name and runs axe 4.12.1 with no serious/critical findings.

## Verification evidence (2026-08-28 UTC)

- Clean dependency install: `npm ci --no-audit --no-fund` passed.
- Unit/integration: `npm test` passed — Vitest `1/1`; Rust `7/7`, including the raw-socket truncation and malformed-JSON receipt regressions.
- Types/lint: `npm run check` passed (`tsc --noEmit`; `cargo clippy --all-targets -- -D warnings`).
- Production artifacts: `npm run build` passed and wrote `dist/`; `cargo build --release --locked` passed.
- Browser/keyboard/a11y: `npm run test:e2e` passed `2/2`. A local production-binary `verify-url.sh` run returned HTTP 200 in 567 ms with no console/page errors, valid title and `lang=en`, exactly one `h1`, one `main`, zero images without `alt`, and zero unlabeled buttons. Manual Playwright checks at 1440px and 390px found no horizontal overflow; first Tab focused Skip to content. Axe 4.12.1 on the controlled 390px page reported `[]` serious/critical violations.
- Privacy/response policy: local production `/health` returned `{"build_sha":"repair-local","status":"ok"}` with `no-store`, CSP, `nosniff`, `DENY`, no-referrer, and restrictive Permissions-Policy headers. An identified release-binary request with no configured upstream returned a signed `503 upstream_not_configured` receipt; its receipt records only the bounded request metadata and no body/credentials.
- Offline/update: after first registration and reload, the service worker controlled the 390px page; `registration.update()` left no waiting/installing worker; an offline reload retained the shell and its heading.
- Performance budget: built JS is 18,142 B raw / 6.90 kB gzip; CSS is 15,229 B raw / 4.44 kB gzip; mobile WebP is 26,308 B. No third-party fonts/scripts or analytics are loaded. Lighthouse 13.4.1 was invoked against the release binary using Playwright Chromium but the browser tab crashed before producing a report, matching the verifier environment's instability; do not treat a score as measured for this repair.
- Container packaging: Docker/Podman is unavailable in this worker. The production image is deployed through the factory ACR build after this commit; that build is the container validation path.

## Deployment evidence

- Committed and pushed repair: `a2d26536b992bbbbbfb74bbf00fca8c7f9980873` (`fix: receipt truncated upstream failures`).
- Factory ACR/container deployment completed for `https://telemetry-export-receipts.sociobot.in` using image tag `sf-telemetry-export-receipts:a2d26536b992`.
- Live `GET /health` returned `200` and `{"build_sha":"a2d26536b992bbbbbfb74bbf00fca8c7f9980873","status":"ok"}` with the expected security and `no-store` headers.
- Live factory `verify-url.sh` passed at 640 ms with no browser errors, valid title/lang, one `h1`, one `main`, zero missing image alts, and zero unlabeled buttons. A 390px Chromium check found no horizontal overflow; the brand accessible name is `TER. — Telemetry Export Receipts home` and the first Tab focuses Skip to content.

## Run and deploy

```sh
npm ci --no-audit --no-fund
npm test
npm run check
npm run build
cargo build --release --locked
npm run test:e2e
PORT=8080 target/release/telemetry-export-receipts
```

The image serves the frontend and API together on `PORT=8080`; it generates and persists an installation signing key when no override is supplied. The factory deployment supplies only `PORT`, so the live policy API remains intentionally unconfigured until an operator supplies the approved upstream configuration and trusted identity-header perimeter.

## Known operational follow-up

- The container deployment configuration intentionally provides only `PORT`; therefore live `/api/v1/policy` reports `configured:false`. A non-production configured upstream plus the trusted gateway identity-header injection are required for a live end-to-end export smoke.
- Receipt reads rely on the deployment's administrator SSO/VPN perimeter. Verify that perimeter strips client-supplied identity headers before production use.

## Asset provenance

The hero scene was generated on 2026-08-28 with the factory Azure OpenAI image deployment. The prompt, source, and provenance are recorded in `.factory/design.md` and `assets/src/receipt-gate-source.png.json`; optimized derivatives are local under `frontend/public/assets/`. The favicon and interface icons are authored SVG geometry. No third-party visual assets, fonts, analytics, or runtime scripts are used.
