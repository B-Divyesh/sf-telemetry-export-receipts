# Telemetry Export Receipts — review 2 handoff

## Outcome

**FAIL.** The product runtime passed, but strict review found one medium
claims-contract finding covering four untested or incompletely tested public
claims.

- Implementation: `a55e7cc99e55af66617e3979430472cd24aea336`
- Documentation head reviewed: `ab97454d2918f7d89b35e06dfa2949c015f93c5e`
- Live build identity: `72a83c83554f55c1e10181c4efb6321e6cd0435c`
- Live URL: <https://telemetry-export-receipts.sociobot.in>
- Full report: [`.factory/review-2.md`](review-2.md)

## What was reviewed

- Fresh desktop and phone browsers, the one-click sample, reset and exit,
  keyboard and focus, 200% scale, reduced motion, offline reload, route titles,
  legal pages, links, expected 404, privacy requests, and recovery states.
- Live and Test checkout redirects, the live $49 offer, actual invalid-license
  verification, cache safety, and the recorded valid return/restore path.
- Live health, anonymous access, client-address rate limiting, `Retry-After`,
  one-replica deployment, and the product `/data` mount.
- Fresh local receipt creation and restart against persistent SQLite and keys.
- Every declared claim command plus the documented quality gates from a clean
  clone of the implementation candidate.
- Byte comparison of all 12 built frontend files against live.

## What passed

- All 13 declared claim commands returned success; full Playwright passed
  16/16 and Rust passed 15 library tests plus startup.
- The live demo is realistic, visible in the opening desktop and phone
  viewports, persistent in its sample label, resettable, and isolated from real
  data and storage.
- Axe found no serious or critical issue. Stable mobile Lighthouse scored 100
  in all four categories.
- Checkout, actual invalid-license locking, free functions, API boundaries,
  rate limits, persistence, security headers, offline behavior, and all earlier
  runtime/page findings passed.

## Finding to address

Complete the claims register and dedicated tests for:

1. Anonymous receipt and export rejection inside the
   `administrator-access` claim command.
2. Withholding an upstream result when its mandatory receipt write fails.
3. Omitting a marked request body from application logs.
4. Locking the archive for a recorded revoked/refunded license response.

Do not remove the intended paid archive. The current $49 Test and Live offers
and free core behavior are correct.

## Run and verify

```sh
npm ci --no-audit --no-fund
npm test
npm run check
npm run build
npm run test:e2e
cargo build --release --locked
```

Then run every command in `.factory/claims.json` from a clean checkout and
cross-check all live and README promises against the register.

## Constraints

- The public upstream is unconfigured. Successful forwarding was verified with
  isolated local upstreams only.
- No production receipt, administrator token, purchase, valid license, or
  telemetry data was accessed or created.
- Docker and Podman were unavailable; the release binary, Dockerfile, live
  container shape, and byte-identical frontend output were checked instead.
