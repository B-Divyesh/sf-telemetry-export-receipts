# Independent verification 3 — FAIL

- **Work order:** `telemetry-export-receipts-verify-3`
- **Candidate:** `89c53799e0091008b6aa94be63e2c47232fd70bc`
- **Live URL:** <https://telemetry-export-receipts.sociobot.in>
- **Verified:** 2026-08-28 09:07 UTC
- **Method:** fresh clone at the candidate SHA; product code was not modified

## Decision

**FAIL.** The repaired malformed-envelope and truncated-upstream receipt paths work, and the live image identifies itself as the requested candidate. Release is still blocked because live receipt state is inconsistent between requests, mandatory API rate limiting is incomplete and non-conforming, valid GET exports never reach the upstream, and the public deployment has no trusted administrative perimeter around receipt data or the identity header.

## Defects

### Critical — live receipts are intermittently missing

A fresh, intentionally denied live export created receipt `01a04799-71c6-7a91-91f6-c53e331f71a7`. Twenty immediate `GET /api/v1/receipts/{id}` requests returned **6 × 200 and 14 × 404**. Twenty unfiltered ledger reads alternated between 50 receipts and zero receipts (7 populated, 13 empty). A desktop browser loaded 50 entries while a fresh 390 px context loaded none.

This is consistent with multiple live instances using isolated local SQLite databases; the external symptom is conclusive even though infrastructure configuration was not changed or inspected. A receipt acknowledged to the caller cannot reliably be retrieved or audited, directly violating the brief's 100% attributable-receipt success measure. Persist and share the database and signing-key boundary across all serving replicas, or constrain the service to one durable replica with a persistent volume and verified recovery.

### High — mandatory rate limiting is absent or malformed

- Local and live bursts of 200 requests to `GET /api/v1/policy` returned **200 × 200** and no 429. Receipt read endpoints are likewise outside the implemented limiter.
- On `POST /api/v1/exports`, both local and live runs allowed 60 requests and first returned 429 on request **61**, but the 429 response had **no `Retry-After` header**.
- Locally, 61 rapid export attempts with the same `X-Forwarded-For` and 61 different requester identities all bypassed the limit (61 × 403 policy denials, 0 × 429). The limiter is keyed by the caller-controlled/trusted identity value, not by the first forwarded client-IP hop required by the backend contract.

The separate Sociobot license verification endpoint passed: after one setup request, a 120-request burst first returned 429 at burst request 30 and included `Retry-After: 4`.

### High — documented GET exports fail before reaching the upstream

Against an isolated upstream that logged every request, an exact-boundary POST export (24 hours, 10,000 rows) returned 200 and reached the upstream. The equivalent valid request with `method: "GET"` returned:

```json
{"error":{"code":"upstream_unavailable","message":"The approved upstream could not be reached."},"receipt_id":"01a04795-4327-7fe3-b147-c16d20291233"}
```

The upstream received no GET request. The receipt was signed but incorrectly described a connection failure. The query map always includes an array-valued `fields` member, which the current URL-query serialization path cannot encode. GET is explicitly accepted by validation and documented in the README, so this removes a material integration mode.

### High — the live administrative trust boundary is public

The live receipt list and individual JSON/Markdown/verification endpoints require no authentication; the ledger returned requester and purpose metadata to an anonymous browser. The public export endpoint also accepts an Internet client-supplied `X-Export-User` as trusted identity. No SSO/VPN challenge or header-sanitizing perimeter was observed. The application does not require sign-in, so the Sociobot Entra authority check is not applicable; however, its documented deployment precondition is not present on the shipped public URL.

The live upstream is currently unconfigured, limiting present exposure to receipt metadata and forged denied attempts. Before any real upstream is configured, place the deployment behind an administrative perimeter that strips client identity headers and injects the authenticated identity. Receipt reads must not be public by default.

### Medium — several interactive targets are below 44 × 44 CSS px

At 390 px, the brand link measured 99 × 34 px. Footer navigation links measured approximately 20–22 px high, and the license ticket's Terms/Privacy links measured 14 px high. Desktop had the same issue (brand 39 px high; legal links 14–22 px high). Focus styling is strong, but these targets miss the attached touch-target contract.

### Medium — pinned development tooling has known high/critical advisories

`npm audit --audit-level=high` reported one high Vite advisory group and one critical Vitest advisory (`vite 7.1.3`, `vitest 3.2.4`). `npm audit --omit=dev --audit-level=high` found zero production dependency vulnerabilities, so the deployed Rust/static runtime is not implicated; local development and CI tooling should still be upgraded and retested.

### Low — startup configuration provenance and HSTS are incomplete

First production boot logs that a signing key was generated, but a restart with the persisted key logs only `server ready`; it does not emit the mandatory generated-versus-supplied configuration summary. Live HTTPS responses also omit `Strict-Transport-Security`. Other required security headers are present.

## Passing evidence

### Clean install and repository gates

The independent clone was detached at the candidate and remained clean after testing.

- `npm ci --no-audit --no-fund` — passed; 63 packages installed.
- `npm test` — passed: Vitest 1/1 and Rust 7/7, including malformed JSON and truncated upstream regressions.
- `npm run check` — passed: `tsc --noEmit` and `cargo clippy --all-targets -- -D warnings`.
- `npm run build` — passed and produced `dist/`.
- `cargo build --release --locked` — passed.
- `npm run test:e2e` — passed 2/2 with Playwright 1.58.2 and axe 4.12.1.
- Docker/Podman is not installed in this verifier, so a new local image build was not possible. The release binary and frontend production artifact were tested directly; live hash/build identity checks cover the deployed candidate.

### Proxy and receipt behavior

- An exact-boundary POST (24 hours, 10,000 rows) returned the upstream CSV unchanged, preserved only the intended Authorization/Cookie/Accept values, injected the declared bounds, and returned receipt ID/signature headers.
- JSON and Markdown receipt reads included requester, purpose, endpoint, time range, row cap, fields, redaction, query digest, policy snapshot, upstream status, and a valid HMAC verification result.
- Over-range, zero/over-limit rows, equal timestamps, empty fields, unapproved redaction, short purpose, traversal/disallowed endpoints, disallowed method, invalid timestamps, malformed JSON, and a body over 256 KiB were denied with receipts for identified callers. Missing identity returned 401 without a receipt.
- A raw upstream that returned `200`, declared 100 bytes, and closed after `partial-export` produced `502 upstream_read_failed`, a signed `upstream_error` receipt with upstream status 200, and a valid verification result.
- Result-body strings, Authorization, Cookie, and raw query values were absent from SQLite.
- A generated production signing key was 64 characters and mode 0600. A persisted receipt remained valid after restart. A short supplied production key exited with status 2.
- 100 concurrent valid POST exports completed in about 1.13 s with 100 responses, 100 unique receipts, and no missing records in the single local database.
- A raw release-binary start with only `PORT` served successfully; `/health` returned `development`. A production-mode start generated and persisted the key as expected.

### Live identity and artifact match

Live `/health` returned:

```json
{"build_sha":"89c53799e0091008b6aa94be63e2c47232fd70bc","status":"ok"}
```

The rebuilt frontend and live files were byte-identical:

| File | SHA-256 |
| --- | --- |
| `index.html` | `70ef46fe0d5f544c695245bcbf2f75172462d48b0febeb624edc23eea4a21252` |
| `assets/index-BIqKVx_0.js` | `0e0ad9ec524bbe1546a81148a6f0db1d5dbad414b6cc47e06042977c5ad8c1ce` |
| `assets/index-Pa5NcSQF.css` | `efe0f2a57b4d76f24c87bebcbc41b79c33a310620aea25c399a7f46ee9466659` |
| `sw.js` | `fdae5cab757c47ed13bc8f08e1893a8491fc5f4afc01c5a299b7f198e4c12e49` |
| `favicon.svg` | `a46df410265521eab040bde5bd643b478b3d772277dee954bb4bcd832af855a8` |
| Desktop/mobile WebP and JPEG | exact matches |

Live `/api/v1/policy` returned `configured:false`; therefore a successful real export cannot be exercised on the public deployment until an approved upstream and trusted identity perimeter are supplied.

### Browser, accessibility, privacy, and recovery

- Factory `verify-url.sh` passed in 621 ms: HTTPS 200, title, `lang=en`, one h1, a main landmark, zero missing image alts, zero unlabeled buttons, and no online console/page errors.
- Independent Chromium checks at 1440 × 900 and 390 × 844 found no horizontal overflow. Body text is 16 px. The first Tab focuses the skip link with a 3 px cyan outline, 3 px offset, and dark separator; native forms and receipt disclosure are keyboard-operable, with Enter changing `aria-expanded` to true.
- Axe 4.12.1 found zero serious/critical violations on desktop, mobile, `/privacy`, and `/terms`.
- Forced API failures rendered specific policy and ledger error states with a Try again action; removing the fault and activating retry restored the ledger.
- Reduced motion changed animation/transition duration to 0.01 ms, animation iteration count to 1, and smooth scrolling to `auto`.
- First load made six same-origin requests and no analytics/font/CDN requests. An invalid restored license was stored under the documented localStorage keys, removed from the address bar, verified only with `api.sociobot.in`, remained locked, and generated no console/page errors or cookies on the product origin.
- Service-worker registration became controlling after reload; `registration.update()` left one activated worker and no waiting/installing worker. Offline reload retained title, h1, main content, image, an offline notice, and an actionable ledger error state. The two expected offline API requests logged `ERR_INTERNET_DISCONNECTED`.

### Performance and response policy

- Lighthouse 13.0.1 mobile: Performance **93**, Accessibility **100**, Best Practices **100**, SEO **100**; FCP 1.1 s, LCP 1.4 s, CLS 0, TBT 320 ms, Speed Index 1.1 s, total transfer 61 KiB.
- Built budgets: JS 18,142 B raw / 6.90 kB gzip; CSS 15,229 B raw / 4.44 kB gzip; mobile hero WebP 26,308 B. No font files or third-party runtime scripts.
- HTML uses `no-cache`; APIs and health use `no-store`; hashed JS/CSS use one-year immutable caching; service worker uses `no-cache`; other assets use one day.
- CSP, `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: no-referrer`, and restrictive Permissions-Policy are present on normal and error responses. Cross-origin preflight from an unapproved origin returned 405 with no CORS grant.

## Required release actions

1. Put receipts and their signing key on one verified durable persistence boundary shared by every live replica; repeat create/read/verify tests through the public load balancer and across restart/revision events.
2. Apply IP-based rate limiting to every API endpoint except health, key it from the first trusted `X-Forwarded-For` hop, and include a valid `Retry-After` on all 429 responses.
3. Repair GET query encoding and add an integration test that proves a required array-valued fields declaration reaches an HTTP upstream.
4. Protect the live ledger and trusted identity boundary with the intended administrator authentication proxy before configuring any upstream.
5. Increase undersized targets, upgrade audited dev tooling, emit configuration provenance at startup, and add HSTS at ingress/application policy.
