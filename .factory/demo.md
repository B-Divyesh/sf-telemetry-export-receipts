# Demo sandbox

- URL: `https://telemetry-export-receipts.sociobot.in/demo`
- Entry: choose **Try it with sample data** on the first screen. The action says it loads allowed, denied, and upstream-error receipts.
- First view: `/demo` opens on a compact sample heading and populated receipt desk. At desktop and phone widths, the first sample receipt is visible before scrolling.
- Sample: three realistic export receipts covering allowed, denied, and upstream-error outcomes.
- Isolation: demo records live only in the page bundle. The demo does not read the API, localStorage, sessionStorage, SQLite, or real receipts.
- Reset: choose **Reset demo** in the persistent banner.
- Exit: choose **Start for real**. Nothing is copied into the real receipt desk.
- Offline: after the service worker finishes installing, `/demo` and its sample data reload without a network connection.
