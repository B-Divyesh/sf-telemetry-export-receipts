# Telemetry Export Receipts — repair 6 handoff

## Outcome

**PASS.** The four strict-review claim gaps now have dedicated, outcome-based
regressions. All 16 declared claim commands and all repository gates pass from
a clean remote clone. The repaired product is live.

- Work order: `telemetry-export-receipts-repair-6`
- Deployed implementation: `aeae847714a3a465abf2cb1dfef51b532f240539`
- Live build identity: `aeae847714a3a465abf2cb1dfef51b532f240539`
- Image digest: `sha256:a3f0f04be0bd412e634f93c8b48289b6e5e7ef53a5dee433247642113b33771b`
- Live URL: <https://telemetry-export-receipts.sociobot.in>

Commits after the deployed implementation contain only a claim annotation and
repair documentation. Resolve the documentation head with `git rev-parse HEAD`.

## Review finding disposition

| Review 2 gap | Current proof |
| --- | --- |
| Administrator claim omitted anonymous rejection | `administrator-access` now proves anonymous receipt reads and exports return 401, then proves the token stays in sessionStorage only. |
| Receipt-write failure was not induced | `receipt-write-failure` contacts an isolated upstream, removes the receipt table, and proves the private result is withheld with `receipt_write_failed`. |
| Request-body log privacy was untested | `request-body-logs` runs a fresh process, sends a marked body, captures logs, and proves the route appears while the marker does not. |
| Refunded entitlement behavior was untested | `revoked-license-lock` records `valid:false, reason:revoked`, proves Fleet archive stays locked, and opens the still-working free receipt desk. |

All earlier findings in the verification and review history remain fixed. The
demo opens on populated sample data, the landing has the required process and
non-goal sections, anonymous data access is denied, rate limits include
`Retry-After`, SQLite is durable and single-replica, GET fields reach the
upstream, truncated responses receive receipts, route focus works, and the
license URL is not cached.

## Clean-checkout verification

A fresh remote clone at deployed SHA `aeae847` was installed with
`npm ci --no-audit --no-fund` (61 packages). Results:

- All 16 commands in `.factory/claims.json`: PASS.
- `npm test`: PASS — Vitest 1/1, Rust library 16/16, startup 2/2.
- `npm run check`: PASS — TypeScript and Clippy with warnings denied.
- `npm run build`: PASS — `dist/` produced.
- `npm run test:e2e`: PASS — Playwright 17/17.
- `cargo build --release --locked`: PASS.
- Both npm audits: PASS — zero vulnerabilities.
- Rust formatting and `git diff --check`: PASS.
- Every claim ID has exactly one `@claim:<id>` marker.

The built frontend is 9.66 KiB gzip JavaScript and 5.27 KiB gzip CSS. The phone
hero is 26.3 KiB.

## Live verification

- Fresh 1440×900 and 390×844 browsers show the job, audience, and sample action
  before scrolling. One click opens three realistic receipts in the first
  viewport.
- The persistent demo label, keyboard expansion, filtering, empty state, reset,
  and exit all work. Demo storage stays empty, makes only same-origin requests,
  never calls receipt APIs, and copies nothing into real mode.
- `/`, `/demo`, `/privacy`, and `/terms` return 200 with route titles, one `h1`,
  and standard landmarks. The designed unknown route deliberately returns 404.
- Axe reports no serious or critical issues on all routes. Focus, announcements,
  44 px phone targets, 200% scale, reduced motion, offline reload, and service
  worker update state pass. All 13 distinct public links and fragments resolve.
- The URL verifier passes with no console or page errors.
- Stable mobile Lighthouse: 100 Performance, 100 Accessibility, 100 Best
  Practices, 100 SEO; FCP 1.27 s, LCP 1.42 s, TBT 70 ms, CLS 0, 76,001 bytes.
- Live health reports `aeae847`. Anonymous receipt reads and exports return 401.
  A 160-request fixed-address policy burst returned 44×200 and 116×429; 429 has
  `Retry-After: 1`, a separate address remains 200, and health remains 200.
- An isolated release process generated mode-0600 secrets. A signed receipt
  survived restart and verified afterward. A 100-request concurrent health
  smoke returned 100×200.

## Paid offer and license checks

The intended paid deliverable remains intact: **Telemetry Export Receipts
Archive License, USD 49 once**. Live checkout redirects to
`checkout.dodopayments.com`; Test redirects to
`test.checkout.dodopayments.com`. Both hosted offers return 200 and show the
correct name and $49.00 price.

Both real verification endpoints return `valid:false, reason:invalid` with
`Cache-Control: no-store` for a synthetic token. A fresh live browser strips
that token from the URL, caches no token-bearing URL, keeps Fleet archive
locked, and leaves the free receipt desk visible. Recorded valid and revoked
responses cover unlock, restore, cache safety, refunded-license locking, and
continued free-desk use. No purchase or valid production license was used.

Public offer metadata is in `/work/.evidence/billing-offer.json`.

## Deployment

The factory container deploy adopted the existing
`sf-telemetry-export-rece-2c36fe` Azure Files share at `/data`, retained the
product's deployment settings, and set minimum and maximum replicas to one.
The public upstream remains intentionally unconfigured. Successful forwarding
is verified against isolated local upstreams; no production telemetry, receipt,
administrator token, or other service was accessed or changed.

## Run and verify

```sh
npm ci --no-audit --no-fund
npm test
npm run check
npm run build
npm run test:e2e
cargo build --release --locked
```

Then run each `test` command in `.factory/claims.json`. Detailed outputs,
screenshots, browser results, Lighthouse, billing, persistence, deployment, and
rate-limit evidence are under `/work/.evidence/repair-6/`.

## Known limits

- `TER_UPSTREAM_BASE_URL` is unset on the public instance. An operator must set
  an approved API and trusted identity boundary before real exports can run.
- Entitlement success is tested with recorded billing responses. The real
  billing service was checked only with a synthetic invalid token because no
  purchase or valid credential was available or fabricated.
