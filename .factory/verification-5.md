# Verification 5 — Record every telemetry export — PASS

- **Work order:** `telemetry-export-receipts-verify-5`
- **Implementation reviewed:** `8382bce957f7aa66257b10da53c53c101d6bd71e`
- **Documentation commit:** `2d164960d093c6a42deb89416d01e5cd73137d34`
- **Live URL:** <https://telemetry-export-receipts.sociobot.in>
- **Verified:** 2026-09-05 UTC

## Verdict

**PASS.** There are **zero findings** and **zero untested declared claims**. The live frontend is byte-identical to the implementation candidate when built with the deployed documentation build SHA. All quality gates and all 13 declared claim commands passed from a separate clean checkout.

## Live product evidence

- Fresh desktop (1440 px) and phone (390 px) contexts both showed the job, audience, and **Try it with sample data** at scroll position zero. The job is recording bounded telemetry exports; the audience is observability teams; the first action opens the isolated sample.
- The sample opened at `/demo`, showed realistic allowed, denied, and upstream-error receipts, kept the persistent **Demo — sample data, nothing is saved** banner, reset cleanly, used empty browser storage, and made only same-origin requests. It did not contact or modify real receipt data.
- The landing page, demo, Privacy, and Terms routes returned 200. Each has its route title, one `h1`, `main`, header, and footer. The designed unknown-route page returned its deliberate HTTP 404 with a return path; this is expected, not a defect.
- Keyboard starts at the visible skip link. Privacy navigation focused its `h1` and announced “Privacy loaded.” At phone width there was no horizontal overflow and no visible target below 44 × 44 px. Reduced motion sets transitions to `0.01ms`.
- Axe 4.12.1 found zero serious or critical issues on `/`, `/demo`, `/privacy`, `/terms`, and the designed 404 route. The factory URL verifier passed: HTTPS 200, title, `lang=en`, one `h1`, `main`, image alt text, labeled buttons, and no console errors.
- After service-worker control, a phone `/demo` reload while offline returned 200 and retained the demo banner and sample receipt content. The active worker had no waiting or installing replacement.
- Live `/health` returned 200 and build SHA `2d164960d093c6a42deb89416d01e5cd73137d34`. This is the documentation-only successor to the implementation candidate. Building candidate `8382bce` with that SHA made live HTML, JS, and CSS byte-identical.
- Anonymous receipt reads and an export carrying only an identity header both returned 401. This confirms the single administrator boundary rather than exposing receipt data publicly.
- A fresh-address burst of 200 live `/api/v1/policy` requests returned 49 × 200 and 151 × 429; sampled 429 responses carried `Retry-After: 1`.
- The public upstream is intentionally unconfigured, so no real public export or public restart was attempted. In an isolated local runtime, an identified policy denial created a receipt, the release binary was stopped and restarted using the same state directory, and the receipt verified successfully after restart. The second boot used persisted signing and administrator secrets.
- The paid offer is present at **US$49 once**. The current checkout endpoint returned 303 to `checkout.dodopayments.com` and completed at HTTP 200. The actual verification endpoint returned `valid:false, reason:invalid` for a synthetic invalid token. In a fresh browser, the token was removed from the URL, the invalid state stayed locked, and Cache Storage contained no URL with `license`.
- All ordinary links resolved successfully. The 404 page’s own skip-link target retains its deliberate 404 status and works within the designed page.

## Clean checkout quality gates

Commands were run from a separate checkout at `8382bce957f7aa66257b10da53c53c101d6bd71e` after `npm ci --no-audit --no-fund`.

| Command | Result |
| --- | --- |
| `npm test` | PASS — Vitest 1/1; Rust 15/15 plus startup 1/1 |
| `npm run check` | PASS — TypeScript and Clippy with warnings denied |
| `npm run build` | PASS — produced `dist/` |
| `npm run test:e2e` | PASS — 14/14 Playwright tests |
| `cargo build --release --locked` | PASS |
| `npm audit --audit-level=high` | PASS — zero vulnerabilities |
| `npm audit --omit=dev --audit-level=high` | PASS — zero vulnerabilities |
| `cargo fmt --all -- --check` and `git diff --check` | PASS |

The built initial JS is 9.02 KB gzip and CSS is 4.60 KB gzip.

## Declared claim results

Every entry in `.factory/claims.json` was run exactly as declared from the clean checkout. **13/13 passed; 0 untested.**

| Claim | Result |
| --- | --- |
| `demo-sandbox` | PASS |
| `offline-reload` | PASS |
| `denied-receipt` | PASS |
| `archive-json` | PASS |
| `port-only-startup` | PASS |
| `recorded-exports` | PASS |
| `signed-downloads` | PASS |
| `privacy-forwarding` | PASS |
| `bounded-get-export` | PASS |
| `administrator-access` | PASS |
| `no-third-party-runtime` | PASS |
| `api-rate-limit` | PASS |
| `paid-license-unlock` | PASS |

## Earlier finding disposition

| Earlier finding | Current disposition |
| --- | --- |
| Truncated upstream response and malformed JSON lacked a receipt | Fixed; relevant Rust regressions and declared claims pass |
| Brand accessible name mismatch | Fixed; current axe result has no serious or critical issue |
| Receipt state was inconsistent across replicas | Fixed by the documented one-replica durable `/data` deployment; local shared-SQLite and restart checks pass |
| API limits were absent or lacked `Retry-After` | Fixed; live burst proves enforcement and header |
| GET array fields failed before the upstream | Fixed; exact-boundary declared claim passes |
| Receipt ledger and identity boundary were public | Fixed; anonymous live reads and identity-only export are 401 |
| Phone controls were below 44 px | Fixed; current phone measurement found none below the threshold |
| Vite/Vitest advisories | Fixed; both audits report zero vulnerabilities |
| Startup provenance and HSTS were incomplete | Fixed; startup test passes and live HTTPS sends HSTS |
| Rate-limited identified exports lacked a receipt | Fixed; `api-rate-limit` declared claim passes |
| Service worker could cache a license-bearing URL | Fixed; fresh invalid-return check found no such cache entry |
| Route changes lacked focus and announcements | Fixed; live Privacy navigation focused the heading and announced the route |
| Missing public claims | Fixed; 13 outcome-based claims are declared and tested |

## Evidence files

Detailed machine-readable browser, axe, offline, license, link, accessibility, and URL-verifier evidence is in `/work/.evidence/` for this worker run. No production receipt, administrator token, or purchase was created or accessed.
