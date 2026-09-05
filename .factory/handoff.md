# Telemetry Export Receipts — review 1 handoff

## Outcome

**FAIL — 2 medium findings, 0 untested declared claims.**

The strict review is in `.factory/review-1.md`. No product code was changed.
The reviewed implementation is
`8382bce957f7aa66257b10da53c53c101d6bd71e`; the documentation head at review
start was `ea1e53b90ea1dfb5bc5c9fed07405176d441d573`. Live reports build
`2d164960d093c6a42deb89416d01e5cd73137d34`, and its HTML, JS, and CSS are
byte-identical to the implementation candidate built with that build ID.

## Findings to repair

1. `/demo` opens on the repeated landing hero. At 1440 × 900 the first receipt
   starts at 1,257 px; at 390 × 844 it starts at 2,294 px. Put the populated
   receipt desk in the first post-click viewport and state beside the action
   what sample will load.
2. Add the required three-step **How it works** section and a dedicated
   privacy/non-goals section before the paid offer on the landing page.

## What passed

- Every one of the 13 declared claim commands passed from a clean detached
  checkout. There are no untested declared or public claims.
- `npm test`, `npm run check`, `npm run build`, `npm run test:e2e`, release
  build, audits, formatting, and diff checks passed.
- Fresh desktop and phone checks passed for plain first-screen copy, sample
  content, reset, isolation, Start for real, keyboard, focus, 200% scale,
  44 px targets, reduced motion, legal routes, designed 404, links, and offline
  reload.
- Axe 4.12.1 found no serious or critical issues. The factory URL verifier
  passed.
- Live health, anonymous protection, first-hop rate limiting, 429
  `Retry-After`, security headers, local restart persistence, boundary cases,
  and recovery paths passed.
- Checkout reaches the registered Dodo-hosted **Telemetry Export Receipts
  Archive License** at US$49 once. Real invalid-token verification, URL
  stripping, cache exclusion, and locked-state recovery passed. The recorded
  valid-return and restore claim test passed.
- Mobile Lighthouse: 100 Performance, 100 Accessibility, 100 Best Practices,
  and 100 SEO. FCP 1.2 s, LCP 1.5 s, CLS 0, TBT 50 ms.

## Verification commands

```sh
npm ci --no-audit --no-fund
npm test
npm run check
npm run build
npm run test:e2e
cargo build --release --locked
npm audit --audit-level=high
npm audit --omit=dev --audit-level=high
cargo fmt --all -- --check
git diff --check
```

Run each command in `.factory/claims.json` exactly as written. Live browser and
backend evidence is under `/work/.evidence/` for this review.

## How to run

```sh
npm ci --no-audit --no-fund
npm run build
cargo run
```

The service listens on `PORT` (default 8080) without required configuration.
It uses `/data` when that mount exists and a local `data/` directory otherwise.
Use `/demo` for the isolated sample.

## Known constraints

The public upstream is intentionally unconfigured, so normal upstream behavior
was exercised against isolated test servers rather than live telemetry. No
production receipt, administrator credential, or payment was created or read.
Docker and Podman were unavailable; the release binary and deployed artifact
were verified directly.

Keep the deployed SQLite service at one replica with its durable `/data` mount.
Horizontal writers require a different database design.
