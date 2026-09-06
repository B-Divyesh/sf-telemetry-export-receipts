# Telemetry Export Receipts — verification 6 handoff

## Outcome

**PASS.** Independent QA found zero findings and zero untested declared claims.

- Implementation: `a55e7cc99e55af66617e3979430472cd24aea336`
- Documentation and live build identity: `72a83c83554f55c1e10181c4efb6321e6cd0435c`
- Live URL: <https://telemetry-export-receipts.sociobot.in>
- Full report: [`.factory/verification-6.md`](verification-6.md)

## Verification summary

- Fresh clean checkout passed `npm test`, `npm run check`, `npm run build`, `npm run test:e2e` (16/16), release build, both npm audits, formatting, and diff checks.
- All 13 commands in `.factory/claims.json` passed exactly as written.
- Fresh desktop and phone browsers showed the job, audience, and sample action before scrolling. The one-click demo opened directly on three realistic receipts, kept its sample label, reset cleanly, stored nothing, called no real receipt API, and exited without copying data.
- Live axe, keyboard, route focus, phone targets, 200% scale, reduced motion, offline/update, links, legal routes, designed 404, privacy requests, and the factory URL verifier passed.
- Mobile Lighthouse scored 100 in all four categories. FCP was 1.1 s, LCP 1.3 s, CLS 0, TBT 40 ms, and total transfer 75 KiB.
- Live authentication boundaries return 401. A fixed-address policy burst returned 116 rate limits with `Retry-After: 1` while another address and health retained their allowances.
- The deployment has one running replica and a product Azure Files mount at `/data`. Fresh local restart evidence confirms receipt and key persistence.
- Live and Test checkout reach the correct Dodo hosts. The live hosted offer shows the $49 archive. Actual invalid verification locks the archive and leaves no license URL in Cache Storage; the recorded valid-return and restore claim passes.
- Rebuilding the implementation with the documentation build SHA produces frontend files byte-identical to live.

## Run and verify

```sh
npm ci --no-audit --no-fund
npm test
npm run check
npm run build
npm run test:e2e
cargo build --release --locked
```

Use `/demo` for the isolated sample. The service runs on `PORT` (default 8080), uses `/data` when mounted, and falls back to local `data/` for development.

## Known constraints

- The public upstream remains unconfigured. Successful forwarding was verified against isolated local upstreams, not production telemetry.
- No production receipt, administrator token, purchase, or valid license was created or accessed. Checkout availability and invalid-license reconciliation are live evidence; the valid entitlement path uses the declared recorded-response browser test.
- No live restart was performed. The live one-replica durable mount, shared-SQLite test, and fresh local release restart are the persistence evidence.
- Docker and Podman were unavailable. The release binary, Dockerfile, live deployment, and byte-identical frontend artifact were checked.

## Next steps

No release-blocking work remains. Configure the documented upstream and trusted outer identity proxy only when an operator is ready to connect real telemetry.
