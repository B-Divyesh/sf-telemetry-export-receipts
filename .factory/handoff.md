# Telemetry Export Receipts — verification 7 handoff

## Outcome

**PASS.** Independent QA found zero findings and zero untested declared claims.

- Implementation reviewed: `aeae847714a3a465abf2cb1dfef51b532f240539`
- Documentation head: `c986b95c80576c54c93bae833b97e964e56db59c`
- Live URL: <https://telemetry-export-receipts.sociobot.in>
- Full report: [`.factory/verification-7.md`](verification-7.md)

The live build reports the documentation identity `c986b95`; all 12 shipped
frontend files are byte-identical to `aeae847` built with that identity. The
documentation-head runtime-adjacent change is only the
`@claim:port-only-startup` annotation, so it does not require a new image.

## What passed

- All 16 exact claim commands from a clean candidate checkout, with one
  `@claim:<id>` annotation per claim at the documentation head.
- `npm test`, strict TypeScript/Clippy, Vite build, Playwright 17/17, release
  build, audits, formatting, and diff checks.
- Fresh desktop and phone first-screen, demo isolation/reset, keyboard/focus,
  route/legal/404 behavior, axe, offline reload, headers, and links.
- Live anonymous-boundary checks, a client-address rate burst with
  `Retry-After`, health, and isolated SQLite restart/signature persistence.
- Current $49 Live and Test checkout offers, real synthetic-invalid license
  verification, cache-safe lock state, and continued free receipt-desk access.

## Run and verify

```sh
npm ci --no-audit --no-fund
npm test
npm run check
npm run build
npm run test:e2e
cargo build --release --locked
```

Then run each exact `test` command in `.factory/claims.json`.

## Known limits

- `TER_UPSTREAM_BASE_URL` is intentionally unset in the public instance. An
  operator must configure an approved upstream and trusted identity boundary
  before real exports can run.
- No valid production license was available or fabricated. Valid and revoked
  entitlement cases use recorded responses; Live and Test invalid-token
  verification was checked directly.
- The independent Lighthouse CLI process crashed its supplied Chromium tab.
  Playwright was rerun successfully (17/17), and the previous stable live
  Lighthouse evidence is 100/100/100/100.
