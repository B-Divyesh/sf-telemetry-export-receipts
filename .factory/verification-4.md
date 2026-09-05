# Verification 4 — Record every telemetry export — FAIL

- Work order: `telemetry-export-receipts-verify-4`
- Implementation reviewed: `7ab3a56a860376e0f785877fe6ce3b0e22dd5ad1`
- Documentation and live build: `c1ff573b6803b5a180d7184f8f203db46e4c4b74`
- Live URL: <https://telemetry-export-receipts.sociobot.in>
- Verified: 2026-09-05 UTC
- Findings: **6** — 1 critical, 1 high, 4 medium
- Untested declared claims: **0**

## Verdict

**FAIL.** The repaired source and all five declared claim commands pass. The live image now matches current `main`, despite the work-order note saying the older image was still live. Acceptance is blocked because the live service has three replicas and no durable `/data` mount, while SQLite, the signing key, and the administrator token all live under `/data`. A separate source defect also leaves a rate-limited export without a receipt. Four medium findings cover checkout, license-token caching, route focus, and the incomplete claims register.

No product code was changed during this verification. Live checks used anonymous reads, the sample sandbox, synthetic license markers, and rate checks on the public policy endpoint. They did not read or change real receipt data.

## Findings

### Critical — live receipt state is not durable or shared

The live app runs image `sf-telemetry-export-receipts:c1ff573b6803` with one active revision and **three running replicas**. Its template allows one to three replicas. Both `volumeMounts` and `volumes` are absent.

The candidate defaults the SQLite database, receipt-signing key, and administrator token to `/data`. Without a fleet volume, that path is each container's local filesystem. Requests can therefore reach separate databases and signing keys, and a restart can remove the audit record. This leaves the critical persistence finding from verification 3 open in the deployed product, even though the source now supports the correct mount and a one-replica deployment.

Evidence: `/work/.evidence/live-container-config.json`, `/work/.evidence/live-revisions.json`, and the repeated scoped Azure query run on 2026-09-05. A live restart was not performed because verification does not authorize a state-changing restart and the static configuration already disproves persistence.

### High — rate-limited exports do not get receipts

In a clean local candidate run, 21 rapid, identified, over-bound export requests used one administrator token and one client address. Requests 1–20 returned policy denials and created 20 signed receipts. Request 21 returned `429` with `Retry-After: 1`, `receipt_id: null`, and no database row.

The rate middleware returns before the audited export handler. This contradicts the README statement that every allowed, denied, or upstream-failed attempt receives a stored receipt and the brief's requirement that every export through the proxy is attributable.

Evidence: `/work/.evidence/local-rate-limit-receipt-gap.json`.

### Medium — the purchase link is unavailable

The visible **Buy one-time license** link returned HTTP 404 with an API error instead of checkout. The core remains usable, but visitors cannot buy the advertised US$49 Fleet archive license. This is an unexpected broken link, not the site's designed unknown-route 404.

Evidence: `/work/.evidence/live-links.json` and `/work/.evidence/checkout-response.txt`.

### Medium — a returned license token remains in Cache Storage

A fresh browser opened the return URL with a synthetic license marker. The page removed the query from the visible address and stored the token in the documented localStorage key. The service worker also cached the complete navigation URL, including the `license` query, under `ter-shell-v3`.

The URL should be normalized or excluded from caching before a license-bearing navigation is stored. Keeping a credential in a cache key defeats the intent of removing it from the address and creates an undocumented second stored copy.

Evidence: `/work/.evidence/live-license-cache.json`.

### Medium — route changes do not move focus or announce the page

The route titles, URLs, back button, and scroll restoration work. After following the Privacy link, however, focus remained on `BODY` and the polite announcer stayed empty. The same happened after returning to the receipt desk. The site uses full-page links and does not meet the required route-change behavior of focusing and announcing the new `h1`.

Evidence: `/work/.evidence/live-history-focus.json`.

### Medium — the public claims register is incomplete

`.factory/claims.json` contains five claims, but the live pages and README make additional testable statements without one dedicated `@claim` command each. Missing claim groups include:

1. Every allowed, denied, and upstream-failed export gets a receipt.
2. JSON and Markdown receipts are signed and downloadable.
3. Result bodies and upstream credentials are never stored.
4. Only the documented upstream headers are forwarded, and successful results return receipt headers.
5. Receipt reads and exports require administrator access, with browser access kept to the tab.
6. The full product has no analytics, ads, external fonts, or runtime CDN scripts.
7. Every API except health is limited by client address and returns `Retry-After` on 429.
8. The US$49 one-time purchase, restore, verification-cache, and archive-unlock flow works.

Broader Rust, Playwright, and manual checks cover much of this behavior, and every public claim area was exercised or reached a definite failure in this run. The formal register is still incomplete, so future claim-only verification would omit material promises.

## Declared claim commands

All commands ran from the clean clone `/tmp/ter-verify4.FOvOHL` at `c1ff573`.

| Claim | Command | Result |
| --- | --- | --- |
| `demo-sandbox` | `npx playwright test -g '@claim:demo-sandbox'` | PASS — 1 test; three sample receipts, empty storage, same-origin requests |
| `offline-reload` | `npx playwright test -g '@claim:offline-reload'` | PASS — 1 test |
| `denied-receipt` | `npx playwright test -g '@claim:denied-receipt'` | PASS — 1 test; receipt verified and raw marker absent |
| `archive-json` | `npx vitest run -t '@claim:archive-json'` | PASS — 1 test |
| `port-only-startup` | `cargo test --test startup port_only_startup_generates_and_reports_configuration_sources` | PASS — 1 test |

The clean checkout remained unchanged. Evidence is in `/work/.evidence/claim-*.log`.

## Live browser results

### First screen and sample

Fresh 1440×900 desktop and 390×844 phone contexts showed the job before scrolling: **Record every telemetry export.** The audience is observability teams, and the first action is **Try it with sample data**. The screen also states the signed formats, result-body boundary, and price.

One click opened `/demo`. Both sizes showed a persistent **Demo — sample data, nothing is saved** label and three realistic receipts: allowed, policy-denied, and upstream-error. Expanding a receipt showed requester, purpose, endpoint, bounds, fields, redaction policy, digest, and signature. Filtering, the empty state, Reset demo, and Start for real worked. Demo storage remained empty and all requests were same-origin. Leaving the demo showed administrator access instead of sample records.

Evidence: `/work/.evidence/live-browser.json` and `/work/.evidence/screens/`.

### Accessibility, routes, and recovery

- `/`, `/demo`, `/privacy`, and `/terms` returned 200 with route-specific titles, `lang=en`, one `h1`, one `main`, header, navigation, footer, and no horizontal overflow.
- The intentionally unknown route returned HTTP 404 and the designed Page not found screen with a return action. Its browser network error is expected and is not a defect.
- Axe 4.12.1 found no serious or critical violations on all five routes at desktop and phone widths.
- Every visible phone control measured at least 44×44 CSS pixels. The first Tab reached the skip link with a 3 px cyan focus outline and 3 px offset. Receipt summaries worked with Enter and Space without trapping focus.
- At 200% page scale, the 390 px page retained the heading and had no horizontal overflow.
- Reduced motion changed transitions to 0.01 ms, removed smooth scrolling, and left no running animation.
- Offline reload after service-worker control retained `/demo`, its banner, and all three samples. The active worker had no waiting replacement.
- Forced local ledger failure showed a specific recovery action; retry succeeded after the fault was removed. Invalid administrator access produced a bound, announced error, and the token stayed in sessionStorage rather than localStorage.

Evidence: `/work/.evidence/live-a11y-routes.json`, `/work/.evidence/live-offline.json`, `/work/.evidence/local-ui-recovery.json`, and `/work/.evidence/live-history-focus.json`.

### Links, privacy, and paid state

Internal pages, fragments, and the source link resolved. The checkout finding is listed above. A synthetic invalid license was removed from the visible URL, checked only with `api.sociobot.in`, left paid controls locked, and produced no console error. No analytics, external font, advertising, or unrelated third-party request occurred. The cache finding is listed above.

The Privacy and Terms pages are present and correctly titled. The demo did not read or write receipt APIs, browser storage, SQLite, or real data.

## Backend results

- Live `/health` returned 200 with build `c1ff573b6803b5a180d7184f8f203db46e4c4b74`.
- Live anonymous receipt reads and an export carrying only a requester identity both returned 401.
- A concurrent live policy burst with one fixed forwarded address returned 44×200 and 156×429; every 429 checked had `Retry-After: 1`. Changing the first forwarded address produced independent allowances, consistent with the documented first-hop key.
- The public upstream is intentionally unconfigured. A successful real export was therefore exercised against an isolated local upstream instead of changing live data.
- Exact 24-hour/10,000-row GET and POST requests reached the local upstream. Array fields became repeated GET keys. Allowed responses, upstream 503, connection failure, malformed JSON, and recovery each created verifiable receipts where the handler ran.
- Validation covered over-range, zero and over-limit rows, equal timestamps, empty/too many/overlong fields, unapproved redaction, traversal, unlisted paths, disallowed methods, short purpose, malformed JSON, and overlong identity.
- JSON and Markdown retrieval worked and signatures verified. A local restart kept the receipt valid and omitted the raw query marker. The critical live mount finding prevents the same guarantee in production.
- The product is intentionally single-tenant. Administrator authentication isolates its receipt desk from anonymous callers; separate application tenants are outside its design.

Evidence: `/work/.evidence/live-api.json`, `/work/.evidence/live-api-concurrent.json`, `/work/.evidence/live-rate-limit-fixed-200.json`, `/work/.evidence/local-boundary-recovery.json`, `/work/.evidence/local-validation-matrix.json`, `/work/.evidence/local-json-markdown.json`, and `/work/.evidence/restart-persistence.json`.

## Earlier finding disposition

| Earlier finding | Current disposition |
| --- | --- |
| Truncated upstream response had no receipt | Source fixed; Rust regression and isolated upstream check pass |
| Malformed JSON had no receipt | Source fixed; claim/validation checks pass |
| Brand accessible name omitted visible text | Fixed; axe 4.12.1 has no serious/critical result |
| Live receipts were inconsistent across replicas | **Open:** live still has three replicas and no durable volume |
| Policy/receipt APIs lacked conforming limits and `Retry-After` | Fixed for the stated limiter behavior; live fixed-address burst proves 429 and `Retry-After` |
| GET array encoding failed before upstream | Fixed; exact-boundary isolated GET check passes |
| Receipt and identity boundary was public | Fixed in the app; live anonymous reads and identity-only exports return 401 |
| Touch targets were under 44 px | Fixed; measured on phone across routes |
| Vite/Vitest advisories | Fixed; both npm audits report zero vulnerabilities |
| Startup provenance was incomplete | Fixed; PORT-only startup test passes for generated and persisted sources |
| HSTS was missing | Fixed; live responses include one-year HSTS with subdomains |

The new 429 receipt gap is distinct from the earlier missing-rate-limit finding: the limiter now works, but its rejection happens outside receipt creation.

## Clean-checkout quality results

- `npm ci --no-audit --no-fund` — PASS, 61 packages.
- `npm test` — PASS, Vitest 1/1 and Rust 14/14.
- `npm run check` — PASS, TypeScript and Clippy with warnings denied.
- `npm run build` — PASS, `dist/` produced.
- `cargo build --release --locked` — PASS.
- `npm run test:e2e` — PASS, Playwright 11/11.
- `npm audit --audit-level=high` — PASS, zero vulnerabilities.
- `npm audit --omit=dev --audit-level=high` — PASS, zero vulnerabilities.
- `cargo fmt --all -- --check` and `git diff --check` — PASS.
- Factory `verify-url.sh` on live — PASS: 200, title, `lang`, one `h1`, `main`, alt text, labeled buttons, and no unexpected console errors.
- Docker/Podman was unavailable. The deployed image identity, live health SHA, production release binary, and Dockerfile contract were checked instead.

## Artifact and performance evidence

Building with `VITE_BUILD_SHA=c1ff573b6803b5a180d7184f8f203db46e4c4b74` produced byte-identical live HTML, JS, CSS, and service worker files. The initial JS is 23.97 kB raw / 8.72 kB gzip, CSS is 16.14 kB raw / 4.60 kB gzip, and the phone hero is 26.3 kB.

Live Lighthouse mobile scores were 100 Performance, 100 Accessibility, 100 Best Practices, and 100 SEO. FCP was 1.1 s, LCP 1.3 s, CLS 0, TBT 0 ms, and total transfer 67 KiB.

## Required next steps

1. Mount the fleet-created `sf-telemetry-export-receipts-data` share at `/data` and set minimum and maximum replicas to one. Then verify create/read/signature continuity through the public address and across a controlled revision restart.
2. Make an identified 429 export produce a stored, signed denial receipt without weakening the limiter.
3. Register or repair the Sociobot checkout target, then exercise purchase return, verification, restore, and archive download.
4. Prevent the service worker from caching navigation URLs that contain `license`; remove any existing sensitive cache entries during activation.
5. Move focus to and announce each route's `h1`, including browser back and forward.
6. Add the missing public claims to `.factory/claims.json`, each with one focused claim test.
