# Telemetry Export Receipts — verification 4 handoff

## Status

**FAIL.** Independent QA reviewed implementation `7ab3a56a860376e0f785877fe6ce3b0e22dd5ad1`; current documentation and the live build are `c1ff573b6803b5a180d7184f8f203db46e4c4b74`. The live deployment changed after the work-order note and now serves the current image.

The complete evidence and six findings are in [`.factory/verification-4.md`](verification-4.md). No product code was modified.

## Main blockers

- Live runs three replicas with no mounted volume. SQLite, signing keys, and administrator tokens under `/data` are therefore neither shared nor durable.
- A rate-limited export returns 429 with `Retry-After` but no receipt.
- The Sociobot checkout link returns 404.
- A license-bearing return URL is retained as a service-worker cache key.
- Route changes leave focus on `BODY` and do not announce the new heading.
- The claims register omits eight public claim groups.

## Verification completed

From a clean clone at `c1ff573`:

```sh
npm ci --no-audit --no-fund
npm test
npm run check
npm run build
cargo build --release --locked
npm run test:e2e
npm audit --audit-level=high
npm audit --omit=dev --audit-level=high
cargo fmt --all -- --check
git diff --check
```

All commands passed: Vitest 1/1, Rust 14/14, Playwright 11/11, both audits zero vulnerabilities, and `dist/` produced. All five commands in `.factory/claims.json` also passed individually.

Fresh desktop and phone browsers covered the first screen, one-click sample, persistent demo label, realistic populated receipts, reset, exit without copying sample data, keyboard, focus styling, reduced motion, 200% scale, axe, offline reload/update, error recovery, legal pages, links, and the designed 404. Live Lighthouse scored 100/100/100/100. The built HTML, JS, CSS, and service worker were byte-identical to live.

Backend checks covered health/build identity, anonymous access, live 429/`Retry-After`, normal and exact-boundary GET/POST forwarding, invalid input, upstream error and recovery, signed JSON/Markdown receipts, query/body privacy, and local restart persistence. The public upstream remains unconfigured, and no real receipt data was read or changed.

## Next verification

After repairs and deployment, confirm a single replica with the durable `/data` mount, then create/read/verify one receipt through the public address before and after a controlled restart. Recheck an identified export at the rate limit, checkout and license return, cache keys, route focus, and every expanded claim command.
