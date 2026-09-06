# Record every telemetry export — verification 7

- Work order: `telemetry-export-receipts-verify-7`
- Verdict: **PASS**
- Findings: **0**
- Untested declared claims: **0**
- Implementation reviewed: `aeae847714a3a465abf2cb1dfef51b532f240539`
- Documentation head: `c986b95c80576c54c93bae833b97e964e56db59c`
- Live URL: <https://telemetry-export-receipts.sociobot.in>
- Verified: 2026-09-06 UTC

## Verdict

**PASS.** There are zero findings of every severity and zero untested declared
claims. The live frontend's 12 shipped files match the implementation candidate
when built with the documentation build identity. The documentation-head change
adds the missing `@claim:port-only-startup` annotation and report material only;
it does not change deployed runtime behavior.

No product code, production receipt, administrator token, telemetry data,
purchase, or valid license was created, read, or changed during this check.

## Product understanding before scrolling

Fresh 1440 × 900 desktop and 390 × 844 phone browsers both opened at scroll
position zero with:

- Job: **Record every telemetry export.**
- Audience: observability teams that need bounded downloads and a requester
  record.
- First action: **Try it with sample data** — it says that it loads allowed,
  denied, and upstream-error sample receipts.

## Live user paths

- The first action opened `/demo` in one click. The realistic Ada receipt began
  at 808 px on desktop and 590 px on phone, inside both opening viewports.
- The persistent **Demo — sample data, nothing is saved** label remained shown.
  Opening a receipt, filtering to **No receipts match**, and **Reset demo** all
  worked. Reset restored the sample.
- Demo browser storage was empty, it made only same-origin requests, and it
  made no receipt API request. Leaving demo did not copy sample records to the
  real receipt desk.
- Fresh invalid-license return stripped the token from the address, showed
  **License no longer active**, left Fleet archive locked, retained the buy
  link, and left no license-bearing Cache Storage URL. Free receipt-desk access
  remains available.

## Live accessibility, routes, privacy, and links

- `/`, `/demo`, `/privacy`, and `/terms` returned 200 with route titles, one
  `h1`, and one `main`. The designed unknown route returned the expected 404
  with **Page not found**; its functioning skip link still has that deliberate
  status.
- Axe 4.12.1 found no serious or critical issue on the five routes at phone
  width. Fresh desktop and phone pages had no console or page error.
- Keyboard testing confirmed the visible skip link, Privacy heading focus, and
  home-heading focus after Back. The live `/demo` service worker controlled the
  page, had no waiting or installing update, and reloaded its sample offline.
- The 13 ordinary public links resolved. The checkout link is checked below;
  the 14th discovered link is the 404 page's own `#main` skip link and is
  intentionally served with its page's 404 status.

An independent Lighthouse CLI attempt could not complete because the supplied
Chromium process crashed its tab before results were written. This was an
environmental browser failure: the affected reduced-motion test then passed in
isolation and the full Playwright suite passed 17/17. The current browser,
accessibility, metadata, console, offline, and bundle checks above passed; the
previous stable live Lighthouse evidence remains 100/100/100/100.

## Backend, isolation, recovery, and limits

- Live `/health` returned 200 with build
  `c986b95c80576c54c93bae833b97e964e56db59c`.
- Anonymous live receipt reads and exports returned 401. This is the intended
  single-tenant administrator boundary; the isolated demo cannot reach it.
- A fresh 160-request policy burst from one forwarded address returned 45×200
  and 115×429. A 429 included `Retry-After: 1`; a second address and `/health`
  each remained 200.
- A fresh isolated release runtime wrote a policy-denial receipt, stopped, and
  restarted against the same SQLite file. The receipt read as 200 after restart,
  its signature verified, and the marked private query value was absent.
- The public upstream remains intentionally unconfigured. Normal forwarding is
  therefore proven only against isolated local upstreams by the declared
  receipt, GET-bound, header/privacy, upstream-failure, and write-failure
  tests.

## Paid offer and license behavior

- Current Live checkout returned 303 to the Dodo Live host; Test checkout
  returned 303 to the Dodo Test host. Both hosted pages returned 200 and showed
  **Telemetry Export Receipts Archive License** at **$49** once.
- Current Live and Test verification endpoints returned 200 with
  `valid:false`, `reason:"invalid"`, and `Cache-Control: no-store` for a
  synthetic invalid token.
- Checkout availability does not prove an entitlement. Recorded valid and
  revoked responses cover return/restore unlock and refunded-license locking;
  no purchase or valid credential was fabricated.

## Candidate and clean-checkout checks

The candidate was freshly cloned and detached at `aeae847`, then installed with
`npm ci --no-audit --no-fund`. Building it with
`VITE_BUILD_SHA=c986b95c80576c54c93bae833b97e964e56db59c` made every one of the
12 shipped frontend files byte-identical to live.

| Check | Result |
| --- | --- |
| All 16 exact commands in `.factory/claims.json` | PASS |
| Claim annotation count at documentation head | PASS — each ID exactly once |
| `npm test` | PASS — Vitest 1/1, Rust library 16/16, startup 2/2 |
| `npm run check` | PASS |
| `npm run build` | PASS — `dist/` produced |
| `npm run test:e2e` | PASS — 17/17 after a clean retry |
| `cargo build --release --locked` | PASS |
| `npm audit --audit-level=high` and production-only audit | PASS — zero vulnerabilities |
| `cargo fmt --all -- --check` and `git diff --check` | PASS |

The initial browser-suite run reached 16 passing tests before Chromium's
headless shell segfaulted. The previously affected reduced-motion test passed
alone, then the complete suite passed 17/17; this was treated as a runner
retry, not a product assertion failure.

## Declared claims

All commands were run exactly as registered and passed:

`demo-sandbox`, `offline-reload`, `denied-receipt`, `archive-json`,
`port-only-startup`, `recorded-exports`, `signed-downloads`,
`privacy-forwarding`, `bounded-get-export`, `administrator-access`,
`no-third-party-runtime`, `api-rate-limit`, `paid-license-unlock`,
`receipt-write-failure`, `request-body-logs`, and `revoked-license-lock`.

The landing page, legal pages, and README were cross-checked against the
register. No unlisted public claim was found. This deterministic proxy and
receipt workflow has no useful missing AI step.

## Earlier finding disposition

| Earlier finding | Current disposition |
| --- | --- |
| Upstream failures, malformed JSON, or truncation lacked receipts | Fixed and regression-covered. |
| Brand accessible name, focus, mobile target, and motion issues | Fixed; fresh axe, keyboard, phone, and offline checks pass. |
| Replica/storage, rate limit, GET bound, anonymous-access, and HSTS issues | Fixed; live health/boundary/rate checks and local restart pass. |
| Checkout, cache-safe licensing, and free-desk access | Fixed; current Live/Test checkout, invalid validation, and browser locking pass. |
| Demo first view and missing landing process/non-goal sections | Fixed; fresh desktop/phone demo and landing checks pass. |
| Four Review 2 claim gaps | Fixed by the administrator, write-failure, request-log, and revoked-license claim tests. |
