# Review 1 — Record every telemetry export — FAIL

- Work order: `telemetry-export-receipts-review-1`
- Implementation reviewed: `8382bce957f7aa66257b10da53c53c101d6bd71e`
- Repository documentation head at review start: `ea1e53b90ea1dfb5bc5c9fed07405176d441d573`
- Live build ID: `2d164960d093c6a42deb89416d01e5cd73137d34`
- Live URL: <https://telemetry-export-receipts.sociobot.in>
- Reviewed: 2026-09-05 UTC
- Findings: **2 medium**
- Untested declared claims: **0**

## Verdict

**FAIL.** The proxy, receipt guarantees, isolated sample, paid-license path,
accessibility, offline behavior, and quality gates work. Two required landing
and demo structures are missing. There are **2 findings** and **0 untested
declared claims**.

## Product understanding before scrolling

Fresh 1440 × 900 desktop and 390 × 844 phone contexts showed the same clear
first screen at scroll position zero:

- Job: **Record every telemetry export.**
- Audience: observability teams that need to limit downloads and record the
  requester.
- First action: **Try it with sample data**.

The action was visible without scrolling at both sizes. The page uses plain
operational headings and a job-specific title. It also states three short
facts: signed formats, no result-body storage, and the optional US$49 archive.

## Findings

### Medium — the populated demo is below the first post-click screen

Choosing **Try it with sample data** opens `/demo` at the top of the same
landing hero. The page repeats the same **Try it with sample data** link, which
points back to `/demo`. No receipt or receipt-desk control is visible in the
first post-click viewport.

Measured at scroll position zero:

| Viewport | Receipt desk top | First sample receipt top | Visible in viewport |
| --- | ---: | ---: | --- |
| 1440 × 900 | 916 px | 1,257 px | No |
| 390 × 844 | 1,260 px | 2,294 px | No |

This fails the demo-sandbox contract that the first screen after the one-click
sample action already look like the product in use. The action also lacks the
required adjacent explanation of what clicking will load. Put the populated
receipt desk in the first `/demo` viewport, or navigate and focus the user on
that desk, while keeping the demo label and reset controls.

Evidence: `/work/.evidence/review-1-demo-first-screen.json`,
`/work/.evidence/review-1-live-demo-desktop.png`, and
`/work/.evidence/review-1-live-demo-phone.png`.

### Medium — the landing page omits two required information sections

The landing `<main>` has four sections: the hero, Receipt desk, **Keep
permissions. Bound the query.**, and the paid archive. It has no **How it
works** sequence in three steps and no dedicated **What it does not do** or
privacy section before the paid offer.

Short privacy facts, the integration paragraph, legal routes, and README are
present, but they do not satisfy the required landing-page order. Add three
plain steps that show request, policy check, and signed receipt. Add a plain
section stating that the product does not store telemetry, replace upstream
permissions, or provide telemetry dashboards.

Evidence: live heading and section extraction recorded during this review;
the source is `frontend/src/main.ts` at the implementation commit.

## One-click sample and privacy

After scrolling to the desk, the sample is realistic and complete. It contains
three receipts for allowed, policy-denied, and upstream-error outcomes.
Expanding the first receipt shows the requester, purpose, endpoint, bounds,
field names, redaction rule, digest, and signature.

The **Demo — sample data, nothing is saved** label remains present. Filtering
to an empty state works. **Reset demo** restores all three receipts. **Start
for real** removes the sample and returns to the administrator boundary.
Throughout this flow, localStorage and sessionStorage remained empty, no
receipt API was called, and no third party was contacted. No real receipt or
production data was read or changed.

## Paid license

- The product states **US$49 once** and names the Fleet archive deliverable.
- The live checkout returned 303 to `checkout.dodopayments.com`, then 200.
- The hosted page showed **Telemetry Export Receipts Archive License** and
  **$49.00** as a one-time purchase.
- The real verification API returned
  `valid:false, reason:"invalid"` for a synthetic invalid token with
  `Cache-Control: no-store`.
- A fresh browser removed that token from the visible URL, kept the paid archive
  locked, showed the buy link, and cached no license-bearing URL.
- The exact declared valid-return and restore test passed against its recorded
  valid verification response. Free receipt functions do not depend on the
  license.

No purchase was made and no credential was read or recorded.

## Live browser, routes, and accessibility

- `/`, `/demo`, `/privacy`, and `/terms` returned 200 with route-specific
  titles, `lang=en`, one `h1`, one `main`, header, and footer.
- The deliberate unknown route returned HTTP 404 with the designed Page not
  found screen and a return action. Its browser resource message is the
  expected result of that intentional 404, not a defect.
- Axe 4.12.1 found zero serious or critical issues on all five routes at phone
  width. The repository suite checks desktop and phone widths.
- The first Tab focuses the skip link with a visible 3 px cyan outline and
  3 px offset. Route navigation and browser Back focus the new `h1` and update
  the polite announcement.
- All visible phone controls measured at least 44 × 44 CSS px. There was no
  horizontal overflow. At 200% page scale the heading stayed visible without
  horizontal overflow.
- Reduced motion changed transitions to 0.01 ms. Keyboard receipt controls,
  labels, errors, and recovery states passed the Playwright suite.
- The factory URL verifier passed with no root-page console or page errors,
  no missing alt text, and no unlabeled buttons.
- Offline `/demo` reload returned 200 with its label and sample receipt. The
  service worker was activated with no waiting or installing update.
- The Privacy and Terms pages are complete. Ordinary internal, source, legal,
  checkout, and fragment links resolved. The 404 status is intentional.

## Backend and recovery

- Live `/health` returned 200 and build
  `2d164960d093c6a42deb89416d01e5cd73137d34`.
- Anonymous receipt reads and an identity-only export both returned 401.
- A 200-request policy burst using one fresh forwarded address returned
  44 × 200 and 156 × 429. Every 429 carried `Retry-After: 1`.
- The public upstream is intentionally unconfigured. Normal successful export,
  exact-boundary GET forwarding, POST forwarding, policy denial, upstream 503,
  truncated response, malformed JSON, header filtering, private-body omission,
  and rate-limited receipt behavior passed isolated Rust tests.
- A fresh local release runtime created a signed denial receipt, stopped, and
  restarted against the same SQLite directory. The receipt remained readable
  and valid; the signing key changed from generated to persisted provenance.
  The private query marker was absent.
- A browser-injected ledger failure showed a specific recovery action. Retrying
  after the fault was removed restored the empty ledger. An invalid
  administrator token produced a bound alert and was kept only in
  sessionStorage.
- The product is deliberately single-tenant. Anonymous isolation and the one
  administrator boundary are enforced; separate application tenants are not a
  claimed feature.

## Clean-checkout gates

All commands ran from a separate detached checkout at
`8382bce957f7aa66257b10da53c53c101d6bd71e` after
`npm ci --no-audit --no-fund` installed 61 packages.

| Command | Result |
| --- | --- |
| `npm test` | PASS — Vitest 1/1; Rust library 15/15; startup 1/1 |
| `npm run check` | PASS — TypeScript and Clippy with warnings denied |
| `npm run build` | PASS — `dist/` produced |
| `npm run test:e2e` | PASS — Playwright 14/14 |
| `cargo build --release --locked` | PASS |
| `npm audit --audit-level=high` | PASS — zero vulnerabilities |
| `npm audit --omit=dev --audit-level=high` | PASS — zero vulnerabilities |
| `cargo fmt --all -- --check` | PASS |
| `git diff --check` | PASS |

Docker and Podman are unavailable in this worker. The release binary,
Dockerfile contract, live build identity, and deployed artifact were checked.

Building the implementation candidate with
`VITE_BUILD_SHA=2d164960d093c6a42deb89416d01e5cd73137d34` produced HTML, JS, and CSS
that are byte-identical to live. The later `ea1e53b` commit changes reports
only, so it does not require another product image.

The initial JS is 9.02 KB gzip and CSS is 4.60 KB gzip. The phone hero is
26.3 KB. Fresh mobile Lighthouse scores were 100 Performance, 100
Accessibility, 100 Best Practices, and 100 SEO; FCP was 1.2 s, LCP 1.5 s,
CLS 0, TBT 50 ms, and total transfer 69 KiB.

## Declared claims

Every command in `.factory/claims.json` ran exactly as declared from the clean
checkout. **13/13 passed; 0 untested.**

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

The landing page and README claim inventory is covered by these entries. No
additional untested public claim was found. AI assistance would not improve
this deterministic policy and receipt boundary, so there is no missed AI
feature finding.

## Earlier finding disposition

| Earlier finding | Current disposition |
| --- | --- |
| Truncated upstream response lacked a receipt | Fixed; Rust regression passes |
| Malformed JSON lacked a receipt | Fixed; Rust regression and declared claims pass |
| Brand accessible name omitted visible text | Fixed; accessible name includes `TER.` and axe is clean |
| Live receipts were inconsistent across replicas | Fixed in the accepted deployment; implementation is unchanged, and fresh local restart persistence passes |
| API limits were absent or lacked `Retry-After` | Fixed; fresh live burst proves 429 and `Retry-After` |
| GET array fields failed before upstream | Fixed; exact-boundary declared claim passes |
| Receipt and identity boundary was public | Fixed; fresh anonymous live checks return 401 |
| Phone controls were below 44 px | Fixed; fresh phone measurement found none |
| Vite/Vitest advisories | Fixed; fresh audits report zero vulnerabilities |
| Startup provenance was incomplete | Fixed; clean PORT-only startup claim passes |
| HSTS was missing | Fixed; live sends one-year HSTS with subdomains |
| Rate-limited identified exports lacked a receipt | Fixed; `api-rate-limit` declared claim passes |
| Checkout was unavailable | Fixed; fresh hosted checkout reaches the registered $49 offer |
| Service worker cached a license-bearing URL | Fixed; fresh invalid-return check found no sensitive cache key |
| Route changes lacked focus and announcements | Fixed; fresh forward and Back checks pass |
| Public claims register was incomplete | Fixed; all 13 claim commands pass |

## Evidence

Machine-readable and screenshot evidence is under `/work/.evidence/`, including
`review-1-live-browser.json`, `review-1-live-backend.json`,
`review-1-license-browser.json`, `review-1-checkout-browser.json`,
`review-1-restart-persistence.json`, `review-1-ui-recovery.json`,
`review-1-links.json`, `review-1-lighthouse.json`, the factory URL verifier,
all claim logs, and all gate logs.
