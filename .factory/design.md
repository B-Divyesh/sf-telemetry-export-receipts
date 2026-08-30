# Telemetry Export Receipts — visual thesis

## Direction: night-market proof desk

The interface borrows the useful visual grammar of a night market after rain: a dark, quiet street; small pools of colored light; stamped paper receipts; and a bright service window that makes the next action unmistakable. This fits an egress-control product because each export is treated like a declared parcel crossing a staffed boundary. The look must feel operational and accountable, never cyberpunk for its own sake.

The product is intentionally single-mode. Its near-black canvas is part of the night-market metaphor and lets receipt paper, status seals, and policy warnings retain a stable meaning. Every status also has a word and icon; color is never the only signal.

## Tokens

- `ink-950 #090B12` — painted night background.
- `ink-900 #101520` — work surface.
- `ink-850 #171D2A` — raised controls.
- `paper #F5F0DF` — receipt paper and primary text on dark.
- `paper-muted #B9B5A7` — supporting text (7.3:1 on `ink-950`).
- `cyan #69E6E1` — primary actions and trusted boundaries (12.1:1 dark text on cyan).
- `coral #FF7B72` — denial and destructive states.
- `marigold #F8C45A` — policy warnings and pending states.
- `jade #59D38C` — allowed/success states.
- `violet #C4A7FF` — signatures and cryptographic details.
- Focus is a 3 px cyan outer ring with a 2 px dark separator.

## Typography

No network fonts. Headings use `Arial Narrow`, `Aptos Narrow`, and system sans fallbacks: condensed lettering recalls hand-painted stall signs without adding font weight to the first load. Body and controls use `Inter` if locally available, then `ui-sans-serif`; technical data uses `ui-monospace`. Numerals are tabular in time ranges, counts, signatures, and receipt IDs. Scale: 14 / 16 / 20 / 28 / 44 / 64 px, with body never below 16 px.

## Layout and spacing

An 8 px rhythm with 4 px for optical nudges. Desktop is a wide counter: 24–40 px gutters, a 12-column grid, and a 1240 px maximum. The policy board occupies four columns while the receipt ledger occupies eight. On 390 px screens the board becomes a compact strip, secondary receipt fields fold below the summary, and the filter row stacks. Targets are at least 44 px; readable text stays within 72 characters.

Receipt entries are perforated horizontal strips rather than generic cards. Dashed rules mean machine-verifiable boundaries; rounded solid panels are reserved for controls. Tiny offset cyan/coral shadows evoke misregistered neon tubes and only appear on the wordmark and primary action.

## Interaction grammar

- A receipt opens from its row, expanding downward from the selected origin.
- Copy/download actions acknowledge immediately with a short verb change and live-region message.
- Status seals use both a stamped glyph and a written outcome.
- Empty, loading, error, and offline states occupy the ledger itself and always name a next step.
- Keyboard order follows header, filters, ledger, then supporting material. No modal is required for core work.

## Motion policy

UI transitions are 180–240 ms and affect only opacity and transform. Newly arrived receipts slide 8 px from the proxy boundary; an expanded receipt unfolds from its row. Nothing loops. Under `prefers-reduced-motion: reduce`, transforms and smooth scrolling are removed and state changes are immediate.

## Original asset plan and prompt sheet

One generated hero illustration shows a small night-market inspection window where abstract glowing telemetry ribbons (log lines, trace spans, and metric dots) pass through a policy gate and leave as a single paper receipt with a wax-like verification seal. Materials: wet asphalt, enamel signs, receipt paper, brushed steel, translucent acrylic. Lighting: cyan service-window light, coral edge light, marigold paper glow, deep navy shadows. Lens: slightly elevated orthographic editorial view, crisp silhouette, generous negative space. No people, text, letters, numbers, logos, brands, watermarks, UI screenshots, gradients-as-background, or misleading dashboards.

Generation command/model: `/opt/fleet/lib/gen-image.sh`, Azure OpenAI image deployment `factory-image`, 2026-08-28. Generated work is original for this product. The final optimized WebP and its source/prompt sidecar live under `frontend/public/assets/`; generated imagery is disclosed in the footer.

The 1200×630 social card and 180×180 touch icon are crops derived from that same original image on 2026-08-30. No new third-party material was introduced.

Authored SVG icons (gate, seal, copy, download, denial) use simple geometric strokes and are product-owned. No third-party visual assets are used.
