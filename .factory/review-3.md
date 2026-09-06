# Record every telemetry export — review 3

- Work order: `telemetry-export-receipts-review-3`
- Verdict: **PASS**
- Findings: **0**
- Untested public claims: **0**
- Implementation reviewed: `aeae847714a3a465abf2cb1dfef51b532f240539`
- Runtime build identity: `c986b95c80576c54c93bae833b97e964e56db59c`
- Documentation head at review start: `2a53c0e067ef87bbed928a6412f2ed0fb4e24ff1`
- Live URL: <https://telemetry-export-receipts.sociobot.in>
- Reviewed: 2026-09-06 UTC

## Verdict

**PASS.** This review found zero defects of every severity and left zero public
claims untested. The live product matches the reviewed implementation, works
from the sample through its receipt and license paths, and retains every
earlier fix.

No product code, production receipt, administrator credential, telemetry
record, purchase, or valid license was created, read, or changed. The review
used only bundled sample data, synthetic invalid values, isolated local state,
and public product endpoints.

## Product understanding before scrolling

Fresh 1440 × 900 desktop and 390 × 844 phone browsers opened at scroll
position zero with:

- Job: **Record every telemetry export.**
- Audience: observability teams that need bounded downloads and a requester
  record.
- First action: **Try it with sample data**. Adjacent text says it loads
  allowed, denied, and upstream-error receipts.

Both screens also showed three facts: signed JSON and Markdown, no stored
result bodies, and the optional $49 archive. The title names the product and
job in plain words.

## One-click sample and real-data isolation

- The first action opened `/demo` in one click. The first sample receipt began
  at 808 px on desktop and 590 px on phone, inside both opening viewports.
- Three realistic receipts covered allowed, policy-denied, and upstream-error
  outcomes. A receipt exposed requester, purpose, endpoint, bounds, fields,
  redaction policy, query digest, and signature.
- Keyboard Enter and Space opened and closed a receipt. Arrow keys operated
  the outcome selector.
- **Demo — sample data, nothing is saved** remained visible through receipt
  inspection, filtering, the empty state, and reset. Reset restored all three
  records.
- Demo localStorage and sessionStorage remained empty. It made no receipt API
  request and contacted no third party.
- **Start for real** removed the sample and returned to the administrator
  boundary. No sample record moved into the real desk.

## Declared claims

All 16 commands in `.factory/claims.json` ran exactly as registered from a
fresh remote clone detached at the implementation candidate. Every command
passed:

`demo-sandbox`, `offline-reload`, `denied-receipt`, `archive-json`,
`port-only-startup`, `recorded-exports`, `signed-downloads`,
`privacy-forwarding`, `bounded-get-export`, `administrator-access`,
`no-third-party-runtime`, `api-rate-limit`, `paid-license-unlock`,
`receipt-write-failure`, `request-body-logs`, and `revoked-license-lock`.

At the documentation head, each claim ID appears in exactly one test. Landing,
legal, demo, and README statements were cross-checked against the register; no
missing or incomplete public claim was found. The deterministic proxy workflow
has no useful missing AI step.

## Paid offer and license behavior

- The product states **$49 once** and names the Fleet archive deliverable.
- Live checkout returned 303 to `checkout.dodopayments.com`; Test checkout
  returned 303 to `test.checkout.dodopayments.com`. Both final pages returned
  200 and showed **Telemetry Export Receipts Archive License** at **$49.00**.
- Current Live and Test verification endpoints returned 200 with
  `valid:false`, `reason:"invalid"`, and `Cache-Control: no-store` for a
  synthetic token.
- A fresh live browser removed that token from the visible URL, kept the
  archive locked, showed the purchase link, cached no license-bearing URL, and
  left the free receipt desk available.
- Recorded valid and revoked responses prove return/restore unlock and refund
  locking. Checkout availability alone was not treated as proof of
  entitlement.

No purchase was attempted and no real license was used.

## Live pages, accessibility, privacy, and recovery

- `/`, `/demo`, `/privacy`, and `/terms` returned 200 with route titles,
  `lang=en`, one `h1`, one `main`, a header, and a footer.
- `/review-3-missing` deliberately returned HTTP 404 with the designed **Page
  not found** screen and return action. Its single failed-resource console
  message is the expected result of that deliberate 404, not a defect.
- Axe 4.12.1 found zero violations on all five routes. The factory URL verifier
  found no root-page console or page errors, missing alt text, or unlabeled
  buttons.
- The skip link had a visible 3 px cyan focus ring. Route changes and Back
  focused and announced the new heading.
- Every visible control measured at least 44 × 44 CSS px at 390 px. Pages had
  no horizontal overflow, including at 200% scale.
- Reduced motion set transitions to 0.01 ms. The sample reloaded offline from
  an active service worker with no waiting or installing update.
- An invalid administrator token produced a bound alert and remained only in
  sessionStorage. An injected ledger failure showed **Try again**; retrying
  returned safely to the administrator prompt.
- Internal routes, source, checkout, robots, sitemap, and fragment paths
  resolved as intended. Security headers include HSTS, CSP with
  `frame-ancestors 'none'`, `nosniff`, and `no-referrer`.
- Fresh mobile Lighthouse scores were 100 Performance, 100 Accessibility, 100
  Best Practices, and 100 SEO. FCP was 1.2 s, LCP 1.5 s, CLS 0, TBT 50 ms,
  and total transfer 75 KiB.

## Backend, isolation, persistence, and limits

- Live `/health` returned 200 with build
  `c986b95c80576c54c93bae833b97e964e56db59c`.
- A 160-request burst from one synthetic forwarded address returned 44 × 200
  and 116 × 429. A limited response had `Retry-After: 1`; another address and
  `/health` remained available.
- Anonymous receipt reads and identity-only exports returned 401. The product
  uses one administrator boundary; the sample cannot reach it.
- The live product has one running replica, minimum and maximum replicas of
  one, and its product Azure Files volume mounted at `/data`.
- A fresh isolated release server created a signed policy-denial receipt, then
  stopped and restarted against the same SQLite file. The receipt remained
  readable and valid. Signing and administrator files were mode 0600; signing
  provenance changed from generated to persisted. The marked private query
  value appeared in neither the receipt nor SQLite.
- Normal forwarding, exact-boundary GET arrays, POST forwarding, header
  filtering, allowed, denied, upstream-503, malformed-JSON, truncated-response,
  write-failure, request-log, and rate-limited receipt paths passed isolated
  Rust tests.

The public upstream is intentionally unconfigured. Successful forwarding was
tested against isolated local upstreams, not production telemetry.

## Candidate and live artifact

Building `aeae847` with
`VITE_BUILD_SHA=c986b95c80576c54c93bae833b97e964e56db59c` produced 12 files that
were byte-identical to the live frontend. The current JavaScript is 9.69 KiB
gzip, CSS is 5.27 KiB gzip, and the phone hero image is 26.3 KiB.

The change from `aeae847` to `c986b95` adds the missing startup claim
annotation and report material; `2a53c0e` adds verification reports only.
Neither requires a new product image.

## Clean-checkout quality gates

The documented Node 22 and Rust prerequisites were present, and `npm ci
--no-audit --no-fund` installed 61 packages.

| Check | Result |
| --- | --- |
| All 16 exact claim commands | PASS |
| `npm test` | PASS — Vitest 1/1, Rust library 16/16, startup 2/2 |
| `npm run check` | PASS — TypeScript and Clippy with warnings denied |
| `npm run build` | PASS — `dist/` produced |
| `npm run test:e2e` | PASS — Playwright 17/17 |
| `cargo build --release --locked` | PASS |
| Both npm high-severity audits | PASS — zero vulnerabilities |
| `cargo fmt --all -- --check` and `git diff --check` | PASS |

Docker and Podman are unavailable in this worker. The release binary,
Dockerfile contract, running product image, and byte-identical live frontend
were checked instead.

## Earlier finding disposition

| Earlier finding | Current disposition |
| --- | --- |
| Upstream failure, malformed JSON, or truncation could lack a receipt | Fixed; current Rust regressions pass. |
| Brand accessible name, focus, motion, or phone targets failed | Fixed; fresh keyboard, axe, reduced-motion, zoom, and target checks pass. |
| Receipt state was inconsistent across replicas or restart | Fixed; one live replica with `/data`, plus isolated restart persistence, pass. |
| API limits were absent, mis-keyed, or lacked `Retry-After` | Fixed; declared claim and fresh live burst pass. |
| Exact-boundary GET fields failed before upstream | Fixed; declared GET claim passes. |
| Receipt and requester boundary was public | Fixed; fresh anonymous checks return 401. |
| Vite/Vitest advisories existed | Fixed; both current audits report zero vulnerabilities. |
| Startup provenance or HSTS was incomplete | Fixed; startup claim and live headers pass. |
| Identified 429 exports lacked receipts | Fixed; declared rate-limit claim passes. |
| Checkout was unavailable or token URLs could be cached | Fixed; current Live/Test offers and cache-safe invalid return pass. |
| Route changes lacked focus and announcements | Fixed; fresh forward and Back checks pass. |
| Demo opened above the sample or landing sections were missing | Fixed; both opening viewports and the required section order pass. |
| Four Review 2 public claims lacked complete tests | Fixed; all four dedicated claim commands pass. |

## Evidence

Screenshots, Lighthouse JSON, and factory URL-verifier output are under
`/work/.evidence/review-3/`. The required report copy is
`/work/.evidence/qa-report.md`; the machine result is
`/work/.evidence/qa-result.json`.
