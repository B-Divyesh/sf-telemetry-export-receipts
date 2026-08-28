# Verification report — FAIL

**Work order:** `telemetry-export-receipts-verify-1`  
**Candidate:** `bfe4d0c2294e3afe35f4757b61fc98ee00b800bc`  
**Live URL:** <https://telemetry-export-receipts.sociobot.in>  
**Verified:** 2026-08-28 UTC

## Decision

**FAIL.** The core audit guarantee is broken for an upstream response that starts successfully and then fails while its body is read: the proxy has already made the export request but returns `502` with `receipt_id: null` and writes no receipt. This contradicts the brief's success measure (100% of exports through the proxy have an attributable receipt) and the product's stated rule that upstream-failed attempts receive receipts. The current build also has one serious accessibility finding.

## Release-blocking defects

### High — an upstream-attempted export can have no receipt

An isolated HTTP upstream returned `200 OK`, declared `Content-Length: 100`, sent `partial-export`, then closed the connection. With an otherwise valid, identified export request, the candidate returned:

```http
HTTP/1.1 502 Bad Gateway
{"error":{"code":"upstream_read_failed","message":"The upstream response could not be read."},"receipt_id":null}
```

The receipt count was `2` before and `2` after; querying `requester=partial@example.com` returned `{"receipts":[]}`. The upstream was contacted before the failure. This is an unreceipted export attempt at the product's egress boundary.

Malformed JSON has the same audit-gap shape: a request with a trusted identity header returned `400 Failed to parse the request body as JSON`; the receipt count stayed unchanged. That is a second, lower-impact unrecorded denied attempt path.

### Medium — serious axe accessibility finding

Lighthouse 13.4.1 (which uses axe 4.12) reported a **serious** `label-content-name-mismatch` / WCAG 2.5.3 finding for `body > div#app > header.site-header > a.brand`: its visible text is `TER.`, while `aria-label="Telemetry Export Receipts home"` omits that visible label. The repository's axe 4.10 Playwright test passed, but its older rule set did not detect this current serious finding. The acceptance contract requires no serious/critical axe findings.

## What passed

### Clean install, quality gates, and production artifact

- Clean checkout was already at the requested commit with no tracked changes.
- `npm ci --no-audit --no-fund` passed.
- `npm test` passed: Vitest `1/1` and Rust `5/5` tests.
- `npm run check` passed: TypeScript and `cargo clippy --all-targets -- -D warnings`.
- `npm run build` passed and produced `dist/`.
- `cargo build --release --locked` passed. Docker could not be tested because neither Docker nor Podman is installed in this worker.
- `npm run test:e2e` passed: `2/2` Playwright tests, including the repository's axe serious/critical check.
- A release-binary production boot generated a 64-character signing key with permission `0600`; `/health` returned the configured candidate SHA. A production signing key shorter than 32 characters exited with code 2 as intended.

### End-to-end proxy boundary

Against a separate local upstream, an exact-boundary valid export (24 hours, 10,000 rows) returned `200`, preserved CSV bytes, sent the upstream only `Authorization`, `Cookie`, and `Accept` headers, injected the configured bounds, and returned a receipt ID/signature with `Cache-Control: no-store`. The recorded receipt contained requester, purpose, endpoint, time range, row cap, fields, redaction policy, digest, policy snapshot, outcome, upstream status, and a valid HMAC verification result.

The upstream result body (`private-upstream-row`), authorization value, cookie, and query value were absent from both JSON and Markdown receipts. An over-24-hour/10,001-row request was denied with a signed receipt and the expected policy message. SQLite receipts persisted across a proxy restart. The per-requester limiter allowed 60 concurrent attempts and returned one `429`; exactly 60 allowed receipts were present. A 120-request concurrent `/health` smoke completed successfully.

### UI, privacy, performance, and response policy

- Chromium inspection at 1440px and 390px found one `h1`, one `main`, no horizontal overflow, no console/page errors, and no third-party requests on a no-license first load.
- Keyboard Tab focused the skip link with the designed cyan 3px ring and 3px offset. At 390px, controls stacked without overflow. Reduced-motion made the receipt-chevron transition `0.01ms`.
- Service-worker registration controlled the page; `registration.update()` completed with no waiting/installing worker for the unchanged `sw.js`. Offline reload after service-worker control retained the shell and displayed the ledger recovery state.
- The browser/Playwright axe 4.10 audit had no serious/critical findings; the current Lighthouse/axe result above supersedes it for release disposition.
- Bundle budget passed: JS `18,133 B` raw / `6,933 B` gzip; CSS `15,229 B` raw / `4,438 B` gzip; mobile WebP `26,308 B` (all under stated budgets). No third-party fonts/scripts or analytics were found; the only runtime cross-origin API in the bundle is the documented Sociobot license endpoint.
- A local mobile Lighthouse run scored Performance `0.95`, Accessibility `1.00`, Best Practices `1.00`, SEO `1.00`; FCP `1.1 s`, LCP `1.9 s`, CLS `0`, TBT `250 ms`. Chromium reported a post-audit tab crash, but Lighthouse wrote the scores and the finding above; do not treat this single run as a stable performance baseline.
- API/UI responses applied CSP, `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: no-referrer`, and a restrictive Permissions-Policy. APIs were `no-store`; hashed JS was immutable for one year; the service worker was `no-cache`.

### Live deployment comparison

The live root HTML, JS, and CSS are byte-for-byte equal to this candidate's fresh `dist/` build:

| File | SHA-256 |
| --- | --- |
| `index.html` | `cb2195fcb0e71465d72fa5bcd9d6223091c22b51db15135fe2166571a6e669e8` |
| `assets/index-BVt1zxnl.js` | `486f80e7d006adde92cc1ff05ee19d844c227b56137b5d99f30b0ddf28e660ee` |
| `assets/index-Pa5NcSQF.css` | `efe0f2a57b4d76f24c87bebcbc41b79c33a310620aea25c399a7f46ee9466659` |

The live URL returned `200` with the expected security/cache headers and UI policy endpoint. `/health` reports `{"build_sha":"development","status":"ok"}` and `/api/v1/policy` reports `configured:false`, so its backend build identity and real export path cannot be positively verified from the deployed environment. The front-end artifact does match the candidate exactly.

## Required next steps

1. Ensure every request that reaches the proxy's export route produces an attributable receipt, including JSON extraction failures and failures after the upstream has accepted a request/read has begun. Do not return an upstream failure without a stored receipt.
2. Make the visible `TER.` text part of the brand link's accessible name, then rerun a current axe/Lighthouse accessibility scan.
3. Deploy with `TER_BUILD_SHA=bfe4d0c2294e3afe35f4757b61fc98ee00b800bc` and a configured non-production upstream test endpoint; repeat live end-to-end export verification.
