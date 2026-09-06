# Review 2 — Record every telemetry export — FAIL

- Work order: `telemetry-export-receipts-review-2`
- Implementation reviewed: `a55e7cc99e55af66617e3979430472cd24aea336`
- Repository documentation head at review start: `ab97454d2918f7d89b35e06dfa2949c015f93c5e`
- Live build ID: `72a83c83554f55c1e10181c4efb6321e6cd0435c`
- Live URL: <https://telemetry-export-receipts.sociobot.in>
- Reviewed: 2026-09-06 UTC
- Findings: **1 medium**
- Untested or incompletely tested public claims: **4**

## Verdict

**FAIL.** The live product works, all 13 declared claim commands return success,
and the earlier runtime and page-structure defects are fixed. The claims
contract is still incomplete. Four public promises are missing complete,
dedicated claim coverage. This leaves **1 medium finding** and **4 untested or
incompletely tested public claims**.

## Product understanding before scrolling

Fresh 1440 × 900 desktop and 390 × 844 phone contexts showed the same clear
first screen at scroll position zero:

- Job: **Record every telemetry export.**
- Audience: observability teams that need to limit downloads and record the
  requester.
- First action: **Try it with sample data**.

The action and its adjacent explanation were visible before scrolling. The
three short facts state the signed formats, result-body boundary, and optional
$49 archive.

## Finding

### Medium — four public promises lack complete claim tests

The exact commands in `.factory/claims.json` all pass, but they do not cover all
of the claims visitors and operators are told to rely on:

1. `administrator-access` says receipt reads and exports require administrator
   access. Its tagged command only opens the desk with a valid test token and
   checks sessionStorage. Anonymous receipt and export rejection pass separate,
   untagged Rust tests and fresh live checks, but not this claim command.
2. README says an upstream result is withheld if its mandatory receipt cannot
   be stored. No claim entry or test induces a receipt-write failure after an
   upstream response and proves that the result body is withheld.
3. README says request bodies are not logged by application code. No claim
   entry captures process logs for a marked request and proves that the marker
   is absent.
4. `/terms` says a refund revokes the license. The paid claim only records a
   valid verification response. The live check used an ordinary invalid token;
   neither path exercises a recorded `valid:false, reason:"revoked"` response
   or a refunded entitlement.

The first behavior is currently covered outside its declared claim command.
Source inspection supports the next two behaviors, and the product locks on a
generic invalid license. Those checks do not satisfy the rule that every public
claim has a dedicated, outcome-based command in the clean sandbox.

## One-click sample and real-data isolation

- `/` opened at scroll position zero with the job, audience, first action, and
  three facts visible on desktop and phone.
- The one-click action opened `/demo` at scroll position zero. The first receipt
  began at 808 px on desktop and 590 px on phone, inside both opening viewports.
- The sample showed three realistic allowed, denied, and upstream-error
  receipts. Keyboard Enter expanded the first receipt and exposed its purpose,
  endpoint, bounds, fields, redaction policy, digest, and signature.
- **Demo — sample data, nothing is saved** remained visible through expand,
  filter, empty state, and reset. Reset restored all three receipts.
- The demo made no receipt-API or third-party request and left localStorage and
  sessionStorage empty. **Start for real** removed all sample receipts and
  copied nothing into the real desk.

No production receipt, administrator token, or telemetry data was read or
changed.

## Live pages, accessibility, links, and offline behavior

- `/`, `/demo`, `/privacy`, and `/terms` returned 200 with route-specific
  titles, `lang=en`, one `h1`, one `main`, a header, navigation, and a footer.
- `/review-2-missing` deliberately returned HTTP 404 with the designed **Page
  not found** screen and a return action. Its browser resource message is the
  expected result of the intentional 404, not a defect.
- Axe 4.12.1 found zero serious or critical issues on all five routes at phone
  width. The URL verifier found no root-page console or page errors, missing alt
  text, or unlabeled buttons.
- The first Tab focused **Skip to content** with a 3 px cyan outline and 3 px
  offset. Privacy navigation, Back, and Forward focused and announced the new
  `h1`.
- No visible phone control was below 44 × 44 CSS px. Neither viewport had
  horizontal overflow. At 200% page scale the heading remained visible without
  overflow.
- Reduced motion changed transition and animation durations to 0.01 ms and
  scrolling to `auto`.
- After service-worker control, `/demo` reloaded offline with its banner and
  sample. The worker was activated with no waiting or installing update.
- All internal routes, fragment targets, the source link, and the checkout link
  resolved as intended.
- Mobile Lighthouse 13.0.1 scored 100 Performance, 100 Accessibility, 100 Best
  Practices, and 100 SEO. FCP was 1.1 s, LCP 1.3 s, CLS 0, TBT 60 ms, and total
  transfer 75 KiB.

## Paid offer and actual license behavior

- The page states **$49 once** and names the Fleet archive deliverable.
- Live checkout returned 303 to `checkout.dodopayments.com`; Test checkout
  returned 303 to `test.checkout.dodopayments.com`. Both hosts returned 200.
- The live hosted page showed **Telemetry Export Receipts Archive License** and
  a **$49.00** one-time total.
- Actual Live and Test verification endpoints returned `valid:false` and
  `reason:"invalid"` with `Cache-Control: no-store` for a synthetic token.
- A fresh live browser stripped that token from the address, showed **License no
  longer active**, kept the archive locked and buy link visible, and cached no
  license-bearing URL.
- The declared valid-return and restore command passed against its recorded
  valid response. Free receipt functions remained available while locked.

No purchase was attempted and no credential was accessed or recorded. Checkout
availability does not prove entitlement or the refund-revocation claim noted in
the finding.

## Backend, persistence, limits, and recovery

- `/health` returned 200 with build
  `72a83c83554f55c1e10181c4efb6321e6cd0435c`.
- A fresh 160-request fixed-address policy burst returned 45 × 200 and 115 ×
  429. A 429 carried `Retry-After: 1`; another client address and health both
  remained 200.
- Anonymous receipt reads and an identity-only export returned 401.
- The live product has one running replica, minimum and maximum replicas of
  one, and its product Azure Files volume mounted at `/data`.
- A fresh local release runtime generated mode-0600 secrets, created a signed
  policy-denial receipt, stopped, and restarted with the same SQLite directory.
  The same receipt remained readable and valid, second boot reported persisted
  key sources, and the private query marker was absent from product state.
- A local browser showed the bound invalid-token alert. An injected ledger
  failure showed **Try again**; removing the fault and retrying restored the
  empty ledger. The administrator token stayed in sessionStorage, never
  localStorage.
- Rust tests cover normal allowed forwarding, denied and upstream-error
  receipts, malformed JSON, a truncated upstream response, exact-boundary GET,
  header filtering, private-body omission, signatures, shared SQLite,
  administrator rejection, and rate-limited receipts.

The public upstream remains intentionally unconfigured. Successful forwarding
was tested against isolated local upstreams, not production telemetry.

## Candidate and live artifact

Building implementation `a55e7cc` with
`VITE_BUILD_SHA=72a83c83554f55c1e10181c4efb6321e6cd0435c` produced 12 frontend
files byte-identical to live, including HTML, JavaScript, CSS, the service
worker, metadata, and all image assets. JavaScript is 9.68 KB gzip, CSS is 5.27
KB gzip, and the phone hero is 26.3 KB.

## Clean-checkout quality gates

Commands ran from a fresh remote clone detached at `a55e7cc` after
`npm ci --no-audit --no-fund` installed 61 packages.

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

Docker and Podman are unavailable in this worker. The release binary,
Dockerfile contract, live container shape, and byte-identical frontend artifact
were checked instead.

## Declared claim commands

All 13 commands ran exactly as declared and returned success. No command was
left unrun.

| Claim | Command result | Coverage result |
| --- | --- | --- |
| `demo-sandbox` | PASS | Complete |
| `offline-reload` | PASS | Complete |
| `denied-receipt` | PASS | Complete |
| `archive-json` | PASS | Complete |
| `port-only-startup` | PASS | Complete |
| `recorded-exports` | PASS | Complete for its registered text |
| `signed-downloads` | PASS | Complete |
| `privacy-forwarding` | PASS | Complete for its registered text |
| `bounded-get-export` | PASS | Complete |
| `administrator-access` | PASS | **Incomplete: access denial is outside this command** |
| `no-third-party-runtime` | PASS | Complete |
| `api-rate-limit` | PASS | Complete |
| `paid-license-unlock` | PASS | Complete for valid return and restore; no revoked case |

The finding also covers three public promises absent from the register. The
review therefore records **4 untested or incompletely tested public claims**,
not zero.

## Earlier finding disposition

| Earlier finding | Current disposition |
| --- | --- |
| Truncated upstream response or malformed JSON lacked a receipt | Fixed; Rust regressions pass |
| Brand accessible name omitted visible text | Fixed; current axe checks are clean |
| Live receipt state was inconsistent across replicas | Fixed; one replica, durable `/data`, shared-SQLite, and restart checks pass |
| API limits were absent, mis-keyed, or lacked `Retry-After` | Fixed; local claim and fresh live burst pass |
| Exact-boundary GET array fields failed before upstream | Fixed; declared GET claim passes |
| Receipt and requester boundary was public | Fixed at runtime; fresh live anonymous checks return 401 |
| Phone targets were below 44 px | Fixed; fresh phone measurement finds none |
| Vite/Vitest advisories | Fixed; both current audits report zero vulnerabilities |
| Startup provenance or HSTS was incomplete | Fixed; startup claim and live headers pass |
| Identified 429 exports lacked receipts | Fixed; declared rate-limit receipt claim passes |
| Checkout was unavailable | Fixed; current Test and Live checkout reach registered offers |
| Service worker cached license-bearing URLs | Fixed; fresh live invalid return leaves no sensitive cache key |
| Route changes lacked focus and announcements | Fixed; fresh forward/back/forward checks pass |
| Claims register omitted public promises | **Still incomplete; four coverage gaps are listed above** |
| Demo opened above the populated sample | Fixed; first receipt is in both opening viewports |
| Landing omitted How it works and non-goals | Fixed; both sections appear before the paid offer |

## Evidence

Machine-readable browser, persistence, recovery, URL-verifier, and stable
Lighthouse evidence plus desktop and phone screenshots are under
`/work/.evidence/review-2/`. The required report copy is
`/work/.evidence/qa-report.md`.
