# Telemetry Export Receipts — repair handoff

## Status

Release-blocking source defects from independent report `14c40424c83e619a6718602fa0480d2792a7d204` are repaired and pushed to `main` in commit `7ab3a56`. Candidate `89c53799e0091008b6aa94be63e2c47232fd70bc` was the failing live revision.

Deployment was not run from this worker. The only installed container deployment helper accesses shared Azure resources whose names are not `sf-telemetry-export-receipts`; the work order explicitly forbids reading or changing them. The repository has no permitted deployment workflow. After the push, the public `/health` endpoint still reported `89c53799e0091008b6aa94be63e2c47232fd70bc`, so live verification of the repair remains pending.

## Findings repaired

- **Durable receipt boundary:** runtime state now defaults to `/data/receipts.db`; first boot generates mode-0600 signing and administrator secrets under `/data`; restart reuses them. The image declares `/data` as its volume and runs non-root. A regression opens two application instances on the same SQLite file and proves create/read/signature visibility across them. Deployment must mount the work order's `/data` share and set both minimum and maximum replicas to one.
- **API rate limiting:** every API except `/health` uses a server-side token bucket keyed by the first parsed `X-Forwarded-For` address. Exports have a stricter bucket. Every 429 includes `Retry-After`. Tests cover policy limiting and prove that changing requester identities cannot bypass export limits.
- **GET forwarding:** arrays now become repeated query keys while scalar values remain scalar and object values use compact JSON. A real isolated HTTP upstream regression proves an exact 24-hour, 10,000-row GET reaches upstream with the expected bounded parameters.
- **Public trust boundary:** exports and every receipt endpoint require `X-TER-Admin-Token`, compared in constant time. The token is generated at first boot unless supplied. The UI stores an entered token only in `sessionStorage`. Tests reject anonymous receipt access and a forged `X-Export-User` without administrator access. The outer administrator proxy must still strip and inject requester identity.
- **Touch targets:** visible links, buttons, fields, and summaries meet 44 by 44 CSS pixels at a 390-pixel viewport; a browser regression measures them.
- **Toolchain advisories:** Vite is `7.3.6`, Vitest is `3.2.7`, and Playwright is pinned to `1.58.2`. Both full and production-only npm audits report zero vulnerabilities.
- **Startup provenance:** startup emits one structured configuration-source record without secret values. A process test clears the environment, supplies only `PORT`, checks `/health`, checks generated mode-0600 secrets, and asserts the provenance fields.
- **HSTS:** all responses include `Strict-Transport-Security: max-age=31536000; includeSubDomains`; browser coverage also checks CSP and nosniff.

The repair also adds the required isolated `/demo`, claim manifest/tests, copy audit, route metadata, sitemap, robots file, social card, legal-route titles, designed 404, service-worker shell discovery, reduced-motion/zoom coverage, and current build identity in the footer.

## Verification evidence

Run from `/work/repo` on 2026-08-30:

```sh
npm ci --no-audit --no-fund
npm test
npm run check
npm run build
cargo build --release --locked
npm run test:e2e
npm audit --audit-level=high
npm audit --omit=dev --audit-level=high
```

Results:

- Vitest: 1/1 passed. Rust: 14/14 passed, including the isolated process startup test.
- TypeScript and Clippy with warnings denied: passed.
- Production build: `dist/` produced; initial JS 23.94 kB raw / 8.69 kB gzip, CSS 16.14 kB raw / 4.60 kB gzip, mobile hero WebP 26.3 kB.
- Release build: passed with `--locked`.
- Playwright: 11/11 passed across desktop and 390-pixel mobile, keyboard, serious/critical axe, 200% page scale, reduced motion, privacy, offline reload/update, protected APIs, response headers, and 404 behavior.
- All five commands in `.factory/claims.json` passed from the demo/test entry points.
- npm audit: zero vulnerabilities in both modes.
- `cargo fmt --all -- --check`, `git diff --check`: passed.
- `/opt/fleet/lib/verify-url.sh` against the local release: HTTP 200, one H1, `lang`, `main`, zero missing alt text, zero unlabeled buttons, and zero console errors.
- Lighthouse 13 mobile on `/demo`: Performance 100, Accessibility 100, Best Practices 100, SEO 100; FCP 1.1 s, LCP 1.3 s, CLS 0, TBT 0 ms.
- Local only-`PORT` start: healthy; generated secrets were mode 0600; restart reported persisted sources and retained receipt verification.
- Local 100-request concurrent `/health` smoke: 100 HTTP 200 responses.

Docker and Podman are unavailable in this worker, so the image itself was not rebuilt locally. The production frontend and locked release binary were built directly, and the Dockerfile contract was inspected and regression-tested through the runtime binary.

## Demo and operations

- Demo URL: `/demo`; it holds three realistic receipts in memory, writes no browser/server storage, and offers Reset demo and Start for real. Details are in `.factory/demo.md`.
- Real receipt/export APIs require the administrator token from `/data/admin-access.key` or the configured override.
- Configure only an exact trusted `TER_UPSTREAM_BASE_URL`; the public live service currently has no upstream configured.
- Back up the database and receipt-signing key together. Run exactly one replica with the durable `/data` mount.

## Remaining release step

The factory deployment authority must deploy `7ab3a56` using the work order's container configuration, mount `/data`, and enforce one replica. Then verify the public health SHA, create/read/verify one receipt repeatedly through the public balancer, restart the revision and verify it again, exercise 429 plus `Retry-After`, confirm anonymous receipt access is 401, and rerun the URL/axe/Lighthouse checks. No product-code gap is known.
