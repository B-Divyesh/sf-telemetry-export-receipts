# Telemetry Export Receipts — independent QA handoff

## Status: FAIL

Candidate `89c53799e0091008b6aa94be63e2c47232fd70bc` was independently tested from a fresh clone against <https://telemetry-export-receipts.sociobot.in> on 2026-08-28. Product code was not changed. The full evidence and reproduction details are in `.factory/verification-3.md`.

## Release blockers

- **Critical:** live receipt state is inconsistent. A newly acknowledged receipt returned 200 on only 6/20 immediate reads and 404 on 14/20; ledger reads alternated between 50 and zero entries. The deployment appears to serve isolated SQLite state across replicas, defeating the core audit guarantee.
- **High:** rate limiting does not cover policy/receipt APIs; 200 policy requests all returned 200. Export limiting begins on request 61 but omits `Retry-After` and is keyed by requester identity rather than the first trusted `X-Forwarded-For` hop.
- **High:** valid GET exports fail locally with 502 before the upstream is contacted. Exact-boundary POST works.
- **High:** the public live deployment exposes receipt metadata anonymously and accepts a caller-supplied identity header without the documented SSO/VPN/header-sanitizing perimeter.

Additional findings: several mobile/desktop links are under 44 px high; `npm audit` reports high Vite and critical Vitest development-tool advisories; startup does not log persisted/supplied configuration provenance; live responses omit HSTS.

## What passed

- Clean `npm ci`, `npm test` (1 frontend + 7 Rust), `npm run check`, `npm run build`, `cargo build --release --locked`, and `npm run test:e2e` (2/2).
- Normal bounded POST forwarding, signed JSON/Markdown receipts, HMAC verification, malformed/oversize/denied request receipts, repaired truncated-body receipt handling, credential/result-body non-persistence, restart persistence in one local database, and a 100-request concurrency smoke.
- Live `/health` reports the exact candidate SHA. Rebuilt HTML, JS, CSS, service worker, icons, and images are byte-identical to live.
- Desktop and 390 px browser checks, keyboard operation, visible focus, online error-free load, zero serious/critical axe 4.12.1 findings, reduced motion, error recovery, legal pages, service-worker update, and offline reload.
- Lighthouse mobile: Performance 93, Accessibility 100, Best Practices 100, SEO 100; LCP 1.4 s and CLS 0. Bundles remain well below contract budgets.
- Privacy scan found no analytics/CDN runtime, and SQLite did not contain forwarded credentials, result bodies, or raw query values. Sociobot license verification rate limiting passed with 429 and `Retry-After`.

## Verification commands

```sh
npm ci --no-audit --no-fund
npm test
npm run check
npm run build
cargo build --release --locked
npm run test:e2e
```

Docker and Podman were unavailable in the verifier, so a fresh local image build was not run. The release binary was exercised directly, and the deployed build identity plus byte-for-byte frontend comparison confirms that live serves this candidate.

## Required next steps

1. Use durable shared receipt/key storage across every serving replica and verify reads and signatures through the live balancer and after restart.
2. Implement mandatory forwarded-IP API limiting with `Retry-After` on every 429.
3. Fix GET query serialization and cover it with a real HTTP upstream integration test.
4. Install the documented trusted administrator perimeter on the public deployment.
5. Resolve the medium/low target-size, dependency-audit, startup-log, and HSTS findings, then rerun independent verification.
