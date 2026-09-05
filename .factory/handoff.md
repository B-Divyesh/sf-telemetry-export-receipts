# Telemetry Export Receipts — repair 4 handoff

## Outcome

The repaired service is deployed and healthy. The deployed implementation is
`8382bce957f7aa66257b10da53c53c101d6bd71e`; it is intentionally separate from
this later documentation-only handoff commit.

It records bounded telemetry exports as signed receipts for self-hosted
observability administrators. The first action is **Try it with sample data**.

## Repairs completed

- The live Container App now has one active, healthy replica and an Azure Files
  volume mounted at `/data`. Its minimum and maximum replica counts are both
  one. SQLite, the generated signing key, and the generated administrator token
  persist there.
- SQLite uses one connection and the Azure Files-compatible `unix-none` VFS.
  This is safe only for this deliberately single-serving-replica deployment;
  do not scale it horizontally or overlap writers.
- Identified, authenticated exports that hit the export limit now receive a
  signed denied receipt as well as `429 Retry-After`. Unauthenticated requests
  remain rate-limited, but do not create an unattributable receipt.
- The registered USD 49 one-time Archive License remains advertised. The live
  checkout endpoint redirects to the hosted Dodo checkout, and license
  verification handles an invalid token correctly. Returned licenses are
  stored locally, verified in the background, restorable, and unlock the paid
  archive UI without gating the free export safeguards.
- Service-worker cache version 4 deletes previous caches on activation and
  never caches URLs containing `license`.
- Internal navigation now changes route titles, moves focus to the new `h1`,
  and announces the new page. The legal and 404 routes retain the standard
  structure.
- The claim register now has 13 outcome-based checks covering demo isolation,
  offline reload, denied/allowed/upstream-failed receipts, signed downloads,
  privacy-preserving forwarding, GET bounds, administrator access, runtime
  origins, API limits, startup, and the paid license flow.
- Landing and application labels were rewritten in direct operational language.
  The catalog description is verb-first and is copied to
  `/work/.evidence/catalog-description.txt`.

## Live deployment evidence

- URL: `https://telemetry-export-receipts.sociobot.in`
- Revision: `sf-telemetry-export-receipts--0000011`
- Image: `sociobotregistry.azurecr.io/sf-telemetry-export-receipts:8382bce957f7`
- Cold `/health`: `200` with build SHA `8382bce957f7aa66257b10da53c53c101d6bd71e`.
- A fresh desktop browser and a 390 px phone browser both showed the job,
  audience, and **Try it with sample data** before scrolling. The one-click
  sample showed Ada's populated receipt records, the persistent demo label,
  reset action, and no real-data writes.
- Fresh live navigation to Privacy focused its `h1` and announced “Privacy
  loaded.” A fresh license-bearing return URL was stripped and its token was
  absent from Cache Storage. Normal-page browser-console checks were clean.
- A concurrent live rate-limit check produced 200 and 429 responses; a sampled
  429 included `Retry-After: 1`.
- Checkout followed one redirect to the hosted Dodo checkout and ended at 200.
  A synthetic invalid token verified as `{ "valid": false, "reason": "invalid" }`.
- `verify-url.sh` passed title, language, one `h1`, `main`, image-alt, button,
  and console checks. Axe found zero serious or critical issues on the live
  demo. Current Lighthouse scores were Performance 98, Accessibility 100,
  Best Practices 100, and SEO 100. Lighthouse's final screenshot collection
  crashed after producing these audit results; the result JSON is retained in
  `/work/.evidence/ter-final/lighthouse.json`.

## Verification from a documented clean setup

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

All commands passed. `dist/` is produced; the initial application JavaScript is
9.02 KB gzip and CSS is 4.60 KB gzip. Both dependency audits report zero
vulnerabilities.

Every declared command in `.factory/claims.json` was also run independently
and passed (13/13):

```sh
npx playwright test -g '@claim:demo-sandbox'
npx playwright test -g '@claim:offline-reload'
npx playwright test -g '@claim:denied-receipt'
npx vitest run -t '@claim:archive-json'
cargo test --test startup port_only_startup_generates_and_reports_configuration_sources
cargo test --lib app::tests::claim_allowed_denied_and_upstream_failed_exports_have_signed_receipts -- --exact
cargo test --lib app::tests::claim_receipts_are_signed_and_downloadable_as_json_and_markdown -- --exact
cargo test --lib app::tests::claim_allowed_exports_forward_only_permitted_headers_and_store_no_result_data -- --exact
cargo test --lib app::tests::claim_get_export_repeats_array_fields_and_reaches_upstream -- --exact
npx playwright test -g '@claim:administrator-access'
npx playwright test -g '@claim:no-third-party-runtime'
cargo test --lib app::tests::claim_api_rate_limit_uses_client_address_and_receipts_for_exports -- --exact
npx playwright test -g '@claim:paid-license-unlock'
```

## How to run

```sh
npm ci --no-audit --no-fund
npm run build
cargo run
```

The service listens on `PORT` (default `8080`) with no required environment
variables. It creates durable state in `/data` when that mount exists, or next
to the binary for local development. Use `/demo` for the isolated sample.

## Known limits and next steps

- The public upstream target is intentionally not configured, and this repair
  worker was not authorized to read the generated administrator token. It
  therefore did not create or read a real public receipt across a controlled
  restart. Isolated integration tests cover creation, retrieval, signatures,
  persistence, error recovery, and rate-limit receipt behavior.
- No real purchase was made to avoid a charge. The live hosted checkout and
  invalid-token verification were checked; the valid-return and restoration
  behavior use a recorded verification fixture in the browser claim.
- Preserve the one-replica bound while SQLite is on Azure Files. A future
  horizontally scaled version needs a database designed for concurrent writers.
