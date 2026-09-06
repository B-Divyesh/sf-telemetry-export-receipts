# Telemetry Export Receipts — repair 5 handoff

## Outcome

**PASS.** The two strict-review findings are repaired in the deployed product.

- Implementation SHA: `a55e7cc99e55af66617e3979430472cd24aea336`
- Product UI repair: `875d2ea20f62f7a21ccccd43d7d5e155f17b98f1`
- Documentation SHA: current repository `HEAD` at handoff
- Live URL: <https://telemetry-export-receipts.sociobot.in>

## What changed

1. The landing action now says that it loads allowed, denied, and upstream-error sample receipts.
2. `/demo` now opens on a compact sample receipt desk instead of repeating the landing hero. The sample banner, reset action, exit action, filters, and realistic populated receipts remain isolated from real data.
3. The landing page now includes **How it works** with request, policy check, and signed receipt steps. It also includes **What it does not do** before the paid offer: no telemetry or result bodies, no replacement for upstream permissions, and no telemetry dashboards.
4. Browser regressions cover the one-click journey at 1440 × 900 and 390 × 844, the first visible sample receipt, the persistent demo label, reset, isolated browser storage, and the landing section order.
5. The rate-limit claim test now drains a token bucket until it observes the public outcome (429 plus `Retry-After`) instead of assuming a fixed request count. This removes normal refill-timing flakiness without changing production rate limiting.

## Verification

From a detached clean checkout at `a55e7cc` after `npm ci --no-audit --no-fund`:

```sh
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

All commands passed. `npm test` ran Vitest plus 15 Rust tests and the startup test. `npm run test:e2e` passed 16 browser tests. Every one of the 13 commands declared in `.factory/claims.json` also passed exactly as written from the clean checkout. The rate-limit claim passed five additional consecutive local runs.

The deployed image was built from `a55e7cc`. Live `/health` returns that full SHA. Rebuilding with the same `VITE_BUILD_SHA` produced JavaScript and CSS byte-identical to the live assets.

Fresh live browser checks before scrolling found:

- Job: **Record every telemetry export.**
- Audience: observability teams that need bounded downloads and an attributable requester record.
- First action: **Try it with sample data**.

After that action, the first sample receipt begins at 808 px on a 1440 × 900 desktop viewport and 590 px on a 390 × 844 phone viewport. Both views keep the demo banner and had no console errors. The sample reset restored all three records without browser storage or receipt API traffic.

Live `verify-url.sh` passed. Axe 4.12.1 found no serious or critical issues on `/`, `/demo`, `/privacy`, `/terms`, or the designed `/missing-page` 404. The deliberate 404 returned HTTP 404 and is not a defect.

Anonymous receipt reads and forged-identity exports return 401. A 120-request live policy burst returned 104 × 200 and 16 × 429; a captured 429 included `Retry-After: 1`. The deployment has one minimum and one maximum replica and mounts its Azure Files product share at `/data`.

The live checkout returns 303 to the Dodo-hosted checkout. The live verify endpoint returns `valid:false, reason:invalid` and `Cache-Control: no-store` for a synthetic invalid token. A fresh browser strips that token from the URL, keeps Fleet archive locked, and has no license-bearing cache entry. The registered offer metadata is in `/work/.evidence/billing-offer.json`.

## Run and deploy

```sh
npm ci --no-audit --no-fund
npm run build
cargo run
```

The service listens on `PORT` (default 8080). It creates and uses `/data` when mounted, or local `data/` for development. Use `/demo` for the isolated sample. Deploy with the container helper and `WO_DATA_DIR=/data`; keep the product at one replica because its SQLite database and generated keys share that durable boundary.

## Known constraints

- The public upstream remains intentionally unconfigured. Successful export forwarding was verified against isolated local upstreams, not with production telemetry.
- No production administrator token, receipt, purchase, or valid license token was created or read. Live durable receipt continuity was therefore not re-tested with production data after this deploy; the clean Rust restart/persistence coverage and prior verification remain the non-production evidence.
- The $49 one-time offer is registered and reachable. A genuine paid entitlement requires a buyer-returned license token; checkout redirect alone is not treated as entitlement proof.
