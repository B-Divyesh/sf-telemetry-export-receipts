# Verification 6 — Record every telemetry export — PASS

- Work order: `telemetry-export-receipts-verify-6`
- Implementation reviewed: `a55e7cc99e55af66617e3979430472cd24aea336`
- Documentation and live build: `72a83c83554f55c1e10181c4efb6321e6cd0435c`
- Live URL: <https://telemetry-export-receipts.sociobot.in>
- Verified: 2026-09-06 UTC
- Findings: **0**
- Untested declared claims: **0**

## Verdict

**PASS.** There are zero findings of every severity and zero untested declared claims. All 13 declared claim commands passed exactly as written from a separate clean checkout. The live product matches the implementation candidate, with the later documentation-only SHA supplied as its build identity.

No product code was changed during this verification. No production receipt, administrator token, purchase, or valid license was created or read.

## Product understanding before scrolling

Fresh 1440 × 900 desktop and 390 × 844 phone contexts showed the required information at scroll position zero:

- Job: **Record every telemetry export.**
- Audience: observability teams that need bounded downloads and a requester record.
- First action: **Try it with sample data**.

The first action is visible at both sizes and says that it loads allowed, denied, and upstream-error sample receipts.

## Live sample and product flow

- One click opens `/demo` with three realistic allowed, denied, and upstream-error receipts.
- The first receipt starts at 808 px on desktop and 590 px on phone, inside each opening viewport.
- **Demo — sample data, nothing is saved**, **Reset demo**, and **Start for real** remain visible.
- Opening a receipt shows its purpose, endpoint, bounds, fields, query digest, and signature. Reset closes it and restores the three-record sample.
- Demo and reset leave localStorage, sessionStorage, and IndexedDB empty. They make no receipt/export API request and no third-party request.
- Starting for real removes the demo banner and returns to the administrator boundary. No sample is copied into real state.
- The landing order includes the receipt desk, **How it works** in three steps, **What it does not do**, and then the paid archive.

## Accessibility, routes, links, offline, and performance

- `/`, `/demo`, `/privacy`, and `/terms` return 200 with route-specific titles, `lang=en`, one `h1`, one `main`, header, navigation, and footer.
- The designed unknown route returns the deliberate HTTP 404 with **Page not found** and a return action. Its 404 resource message is expected and is not a defect.
- Axe 4.12.1 found no serious or critical issue on all five routes. The factory URL verifier passed with no root-page console/page error, missing image alt, or unlabeled button.
- The first Tab focuses the skip link with a 3 px cyan outline and 3 px offset. Receipt rows work with Enter and Space. Forward, Back, and Forward again focus the route `h1` and update the polite announcement.
- No visible phone target is below 44 × 44 CSS px. There is no horizontal overflow. The heading remains visible without overflow at 200% page scale.
- Reduced motion changes transition and animation duration to 0.01 ms and smooth scrolling to `auto`.
- After service-worker control, a phone `/demo` reload works offline with its label and all three samples. The active worker is activated with no waiting or installing update.
- All 36 same-origin links and fragments found across the public routes resolve. The source link resolves, and the checkout is covered below.
- Mobile Lighthouse 13.0.1: 100 Performance, 100 Accessibility, 100 Best Practices, and 100 SEO; FCP 1.1 s, LCP 1.3 s, CLS 0, TBT 40 ms, and 75 KiB total transfer.
- The candidate build is 9.68 KB gzip JavaScript and 5.27 KB gzip CSS; the phone hero is 26.3 KB.

## Paid offer and license behavior

- The product states **US$49 once** and names the paid deliverable as the Telemetry Export Receipts Archive License / Fleet archive.
- Current Live checkout returns 303 to `checkout.dodopayments.com`, and Test checkout returns 303 to `test.checkout.dodopayments.com`; both hosted pages return 200.
- The live hosted offer shows the named archive license and a $49.00 total.
- The actual Live and Test verification endpoints return `valid:false`, `reason:"invalid"`, and `Cache-Control: no-store` for a synthetic invalid token.
- A fresh live browser strips the invalid token from the address, shows **License no longer active**, keeps the archive locked, keeps the buy link visible, and caches no license-bearing URL.
- The declared valid-return and restore claim passes against its recorded valid verification response. Free receipt functions remain available without a license.

A checkout redirect proves offer availability, not entitlement. No valid production license was available or fabricated, and no purchase was attempted.

## Backend, isolation, limits, and recovery

- Live `/health` returns 200 with build `72a83c83554f55c1e10181c4efb6321e6cd0435c`. The only change after implementation `a55e7cc` is `.factory/handoff.md`.
- Building `a55e7cc` with `VITE_BUILD_SHA=72a83c83554f55c1e10181c4efb6321e6cd0435c` produces root HTML and every shipped frontend asset byte-identical to live.
- Anonymous receipt reads and an export with only a requester identity return 401.
- The product is deliberately single-tenant. Its isolation boundary is one administrator token; the demo cannot reach that boundary or production receipt state.
- A fresh fixed-address burst of 160 live policy requests returned 44 × 200 and 116 × 429. A 429 included `Retry-After: 1`; a different first forwarded address retained an independent 200 allowance, and health remained 200.
- The deployed product has one running replica, minimum and maximum replicas set to one, and its product Azure Files volume mounted at `/data`.
- A fresh local release runtime started with only `PORT`, generated owner-only secrets, created a signed policy-denial receipt, stopped, and restarted with the same state directory. The receipt remained readable and valid, the private query marker remained absent, and second boot reported persisted key sources.
- The Rust suite covers allowed, denied, upstream-503, truncated-response, malformed-JSON, exact-boundary GET, header filtering, signature, shared-SQLite, administrator, and rate-limited-receipt paths.
- A browser-injected policy failure showed its error. A ledger failure showed **Try again**; removing the fault and retrying restored the empty ledger. Invalid administrator access produced a bound `role=alert` error.

The public upstream is intentionally unconfigured, so a real live successful export and live receipt restart were not attempted. The clean isolated upstream and restart tests cover those claims without touching production data.

## Clean-checkout quality gates

Commands ran from a fresh remote clone detached at `a55e7cc` after `npm ci --no-audit --no-fund` installed 61 packages.

| Command | Result |
| --- | --- |
| `npm test` | PASS — Vitest 1/1, Rust library 15/15, startup 1/1 |
| `npm run check` | PASS — TypeScript and Clippy with warnings denied |
| `npm run build` | PASS — produced `dist/` |
| `npm run test:e2e` | PASS — Playwright 16/16 |
| `cargo build --release --locked` | PASS |
| `npm audit --audit-level=high` | PASS — zero vulnerabilities |
| `npm audit --omit=dev --audit-level=high` | PASS — zero vulnerabilities |
| `cargo fmt --all -- --check` | PASS |
| `git diff --check` | PASS |

Docker and Podman are unavailable in this worker. The release binary, Dockerfile contract, live build identity, product-only deployment shape, and deployed files were checked instead.

## Declared claims

Every command in `.factory/claims.json` ran exactly as declared. **13/13 passed; 0 untested.**

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

The live pages and README were cross-checked against the register. No unlisted public claim was found. This deterministic proxy and receipt workflow has no useful missing AI step.

## Earlier finding disposition

| Earlier finding | Current disposition |
| --- | --- |
| Truncated upstream response or malformed JSON lacked a receipt | Fixed; Rust regressions pass |
| Brand accessible name omitted visible text | Fixed; current axe checks are clean |
| Live receipt state was inconsistent across replicas | Fixed; one replica and durable `/data` mount are live, shared-SQLite and restart tests pass |
| API limits were absent, mis-keyed, or lacked `Retry-After` | Fixed; local claim and fresh live burst pass |
| Exact-boundary GET array fields failed before upstream | Fixed; declared GET claim passes |
| Receipt and requester boundary was public | Fixed; fresh live anonymous checks return 401 |
| Phone targets were below 44 px | Fixed; fresh phone measurement finds none |
| Vite/Vitest advisories | Fixed; both current audits report zero vulnerabilities |
| Startup provenance or HSTS was incomplete | Fixed; startup claim passes and live sends one-year HSTS |
| Identified 429 exports lacked receipts | Fixed; declared rate-limit receipt claim passes |
| Checkout was unavailable | Fixed; fresh Test and Live checkout checks reach the registered offers |
| Service worker cached license-bearing URLs | Fixed; fresh live invalid return leaves no sensitive cache key |
| Route changes lacked focus and announcements | Fixed; fresh forward/back/forward checks pass |
| Claims register omitted public promises | Fixed; all 13 registered commands pass |
| Demo opened above the populated sample | Fixed; first receipt is in the opening desktop and phone viewport |
| Landing omitted How it works and non-goals | Fixed; both sections are present before the paid offer |

## Evidence

Screenshots, the Lighthouse JSON, and factory URL-verifier output are under `/work/.evidence/verification-6/`. The required report copy is `/work/.evidence/qa-report.md`.
