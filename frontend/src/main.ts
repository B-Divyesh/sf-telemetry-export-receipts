import './styles.css'
import { createArchive } from './archive'

const SLUG = 'telemetry-export-receipts'
const LICENSE_KEY = `sb_license:${SLUG}`
const VERDICT_KEY = `${LICENSE_KEY}:verdict`
const API = 'https://api.sociobot.in/api/v1'

type Policy = {
  configured: boolean
  allowed_paths: string[]
  max_range_hours: number
  max_rows: number
  redaction_policies: string[]
  identity_header: string
  signing: string
}

type Receipt = {
  schema: string
  id: string
  created_at: string
  requester: string
  purpose: string
  endpoint: string
  method: string
  time_range: { start: string; end: string }
  row_limit: number
  fields: string[]
  redaction_policy: string
  query_sha256: string
  policy: { max_range_seconds: number; max_rows: number; authorization_forwarded: boolean; result_body_recorded: boolean }
  outcome: 'allowed' | 'denied' | 'upstream_error'
  upstream_status: number | null
  denial_reason: string | null
  signature: string
}

const icon = (name: 'seal' | 'gate' | 'copy' | 'download' | 'refresh') => {
  const paths = {
    seal: '<path d="M12 2l3 2 3.5.5.5 3.5 2 3-2 3-.5 3.5-3.5.5-3 2-3-2-3.5-.5-.5-3.5-2-3 2-3 .5-3.5 3.5-.5z"/><path d="m8.8 12 2.1 2.1 4.5-4.7"/>',
    gate: '<path d="M4 21V5l8-3 8 3v16"/><path d="M8 21V8h8v13M2 21h20"/>',
    copy: '<rect x="8" y="8" width="11" height="11" rx="2"/><path d="M16 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h2"/>',
    download: '<path d="M12 3v12m0 0 4-4m-4 4-4-4M5 21h14"/>',
    refresh: '<path d="M20 6v5h-5M4 18v-5h5"/><path d="M18.5 9A7 7 0 0 0 6.2 6.2L4 8m2 7a7 7 0 0 0 11.8 2.8L20 16"/>',
  }
  return `<svg aria-hidden="true" viewBox="0 0 24 24">${paths[name]}</svg>`
}

const app = document.querySelector<HTMLDivElement>('#app')!
const path = location.pathname

function shell(content: string, page: string) {
  app.innerHTML = `
    <header class="site-header">
      <a class="brand" href="/" aria-label="Telemetry Export Receipts home"><span class="brand-mark">${icon('gate')}</span><span>TER<span class="brand-dot">.</span></span></a>
      <nav aria-label="Primary"><a ${page === 'desk' ? 'aria-current="page"' : ''} href="/">Receipt desk</a><a href="/#integration">Integrate</a><a href="/#license">License</a></nav>
      <span class="boundary"><i></i> Egress boundary</span>
    </header>
    ${content}
    <footer><p><span class="footer-seal">${icon('seal')}</span> Built for operators who need proof, not another telemetry store.</p><nav aria-label="Legal"><a href="/privacy">Privacy</a><a href="/terms">Terms</a><a href="https://github.com/B-Divyesh/sf-telemetry-export-receipts">Source</a></nav><small>Hero imagery generated for this product with Azure OpenAI. No analytics or tracking.</small></footer>
    <div id="announcer" class="sr-only" aria-live="polite"></div>`
}

function legalPage(kind: 'privacy' | 'terms') {
  const privacy = `
    <main id="main" class="legal"><p class="eyebrow">Legal / plain language</p><h1>Privacy</h1><p class="lede">Telemetry Export Receipts is designed to know about the export, not the exported data.</p>
    <h2>What this installation stores</h2><p>For every permitted, denied, or failed export attempt, the server stores requester identity, purpose, endpoint, bounded time range, row cap, selected field names, redaction policy, a query hash, outcome, and signed policy snapshot. It does not store upstream authorization credentials or result bodies.</p>
    <h2>Where it lives</h2><p>Receipts stay in the SQLite database controlled by the self-hosting operator. This web interface includes no analytics, advertising, tracking pixels, or third-party runtime scripts.</p>
    <h2>Licenses</h2><p>If you buy or restore a license, its token and a time-limited verification result are stored in this browser. Verification is sent to Sociobot, the merchant of record. Do not paste an observability access token into the license field.</p>
    <h2>Your controls</h2><p>Operators control retention by managing the local database. Clear this browser's site data to remove a locally stored license. Contact the operator of your installation for access or deletion requests.</p><p><em>Effective 28 August 2026.</em></p></main>`
  const terms = `
    <main id="main" class="legal"><p class="eyebrow">Legal / plain language</p><h1>Terms</h1><p class="lede">Use this software as one accountable layer in your export path—not as a replacement for upstream permissions.</p>
    <h2>Service</h2><p>The software enforces configured time, row, path, and redaction bounds and creates signed records. You remain responsible for authenticating users, trusting the configured identity header only from your auth proxy, securing the signing key, and validating each upstream API integration.</p>
    <h2>One-time license</h2><p>The optional Fleet archive unlock costs US$49 once and adds bulk local receipt packaging in the operator UI. The core proxy, safety policy, receipt signing, individual JSON/Markdown access, and accessibility remain available without purchase. Sociobot/Dodo is the merchant of record; refunds are handled there and revoke the license.</p>
    <h2>Warranty</h2><p>The open-source software is provided under the MIT License, without warranty. Review and test configuration before using it for a compliance program.</p><p><em>Effective 28 August 2026.</em></p></main>`
  document.title = `${kind === 'privacy' ? 'Privacy' : 'Terms'} — Telemetry Export Receipts`
  shell(kind === 'privacy' ? privacy : terms, 'legal')
}

if (path === '/privacy' || path === '/terms') {
  legalPage(path.slice(1) as 'privacy' | 'terms')
} else {
  shell(`
  <main id="main">
    <section class="hero" aria-labelledby="hero-title">
      <div class="hero-copy"><p class="eyebrow"><span>01</span> Signed at the boundary</p><h1 id="hero-title">Every export<br><em>leaves proof.</em></h1><p class="lede">Put a hard limit in front of telemetry downloads. Preserve upstream permissions. Issue a signed receipt that says who exported what—and under which policy.</p><div class="hero-actions"><a class="button primary" href="#desk">Inspect receipts <span>↓</span></a><a class="button quiet" href="#integration">Proxy an export <span>↗</span></a></div><ul class="proof-points"><li>${icon('seal')} Signed JSON + Markdown</li><li>${icon('gate')} Result bodies never stored</li></ul></div>
      <figure class="hero-art"><picture><source media="(max-width: 600px)" srcset="/assets/receipt-gate-mobile.webp"><source srcset="/assets/receipt-gate.webp" type="image/webp"><img src="/assets/receipt-gate.jpg" width="960" height="640" fetchpriority="high" decoding="async" alt="An illustrated night-market gate turns abstract telemetry streams into a sealed paper receipt."></picture><figcaption><span>Policy gate</span><span>Bounded egress → signed proof</span></figcaption></figure>
    </section>
    <section id="desk" class="desk" aria-labelledby="desk-title">
      <div class="section-heading"><div><p class="eyebrow"><span>02</span> Live ledger</p><h2 id="desk-title">Receipt desk</h2></div><p>Machine-signed records from this installation. The newest crossing appears first.</p></div>
      <div class="desk-grid">
        <aside class="policy-board" aria-labelledby="policy-title"><div class="board-head"><span>${icon('gate')}</span><div><p>Active policy</p><h3 id="policy-title">Export boundary</h3></div></div><div id="policy-state" class="policy-loading" aria-live="polite"><span class="pulse"></span> Reading server policy…</div></aside>
        <div class="ledger">
          <form id="filters" class="filters" role="search"><div class="field"><label for="requester">Requester</label><input id="requester" name="requester" type="search" autocomplete="off" placeholder="name@example.com"></div><div class="field"><label for="outcome">Outcome</label><select id="outcome" name="outcome"><option value="">All outcomes</option><option value="allowed">Allowed</option><option value="denied">Denied</option><option value="upstream_error">Upstream error</option></select></div><button class="icon-button" type="submit">${icon('refresh')} Refresh</button></form>
          <div id="network-state" class="network-state" hidden role="status">You’re offline. Showing the most recently loaded receipt list.</div>
          <div id="receipt-list" class="receipt-list" aria-live="polite" aria-busy="true"><div class="loading-receipt"><span></span><span></span><span></span></div><p>Opening the ledger…</p></div>
        </div>
      </div>
    </section>
    <section id="integration" class="integration" aria-labelledby="integration-title"><div><p class="eyebrow"><span>03</span> One guarded route</p><h2 id="integration-title">Keep permissions.<br>Bound the query.</h2><p>The proxy forwards your existing <code>Authorization</code> and <code>Cookie</code> headers only to the configured upstream. Your trusted auth proxy supplies requester identity. The response comes back unchanged with receipt ID and signature headers.</p></div><div class="code-panel"><div class="code-head"><span><i></i><i></i><i></i></span><button id="copy-curl" type="button">${icon('copy')} Copy request</button></div><pre><code id="curl-example">curl -X POST https://your-host/api/v1/exports \\
  -H 'Authorization: Bearer …' \\
  -H 'X-Export-User: ada@example.com' \\
  -H 'Content-Type: application/json' \\
  -d '{
    "endpoint": "/api/logs/export",
    "start": "2026-08-28T09:00:00Z",
    "end": "2026-08-28T10:00:00Z",
    "row_limit": 5000,
    "fields": ["timestamp", "service", "message"],
    "redaction_policy": "pii-basic",
    "purpose": "INC-204 response"
  }'</code></pre><p class="code-note"><span>Response headers</span> X-Export-Receipt-Id · X-Export-Receipt-Signature</p></div></section>
    <section id="license" class="license-section" aria-labelledby="license-title"><div class="license-copy"><p class="eyebrow"><span>04</span> Optional operator unlock</p><h2 id="license-title">Take the audit with you.</h2><p>The core proxy and individual signed receipts stay free. Fleet archive packages the loaded audit set for an offline review or handoff.</p><ul><li>Bulk JSON archive from current filters</li><li>Portable, no recurring data-volume fee</li><li>One installation license</li></ul></div><div class="license-ticket"><p class="ticket-kicker">Fleet archive</p><p class="price"><strong>$49</strong> <span>once</span></p><p id="license-status">License not installed</p><a id="buy-license" class="button primary" href="https://api.sociobot.in/api/v1/products/telemetry-export-receipts/checkout">Buy one-time license <span>↗</span></a><button id="download-archive" class="button primary" type="button" aria-label="Download JSON archive" hidden>${icon('download')} Download JSON archive</button><details><summary>Have a license? Restore it</summary><form id="license-form"><label for="license-token">License token</label><input id="license-token" name="license" autocomplete="off" spellcheck="false"><button type="submit" class="button quiet" aria-label="Verify license">Verify license</button></form></details><p class="legal-note">Sociobot/Dodo is merchant of record. Refunds are handled there. <a href="/terms">Terms</a> · <a href="/privacy">Privacy</a></p></div></section>
  </main>`, 'desk')
  void initDesk()
}

let loadedReceipts: Receipt[] = []

async function initDesk() {
  captureReturnedLicense()
  bindInteractions()
  updateNetworkState()
  addEventListener('online', updateNetworkState)
  addEventListener('offline', updateNetworkState)
  await Promise.all([loadPolicy(), loadReceipts(), checkLicense()])
  if ('serviceWorker' in navigator) navigator.serviceWorker.register('/sw.js').catch(() => undefined)
}

function bindInteractions() {
  document.querySelector<HTMLFormElement>('#filters')!.addEventListener('submit', event => { event.preventDefault(); void loadReceipts() })
  document.querySelector('#copy-curl')!.addEventListener('click', async () => {
    const value = document.querySelector('#curl-example')!.textContent || ''
    try { await navigator.clipboard.writeText(value); announce('Request copied to clipboard.'); setButtonText('#copy-curl', 'Copied') }
    catch { announce('Copy failed. Select the request text and copy it manually.') }
  })
  document.querySelector<HTMLFormElement>('#license-form')!.addEventListener('submit', event => {
    event.preventDefault()
    const token = new FormData(event.currentTarget as HTMLFormElement).get('license')?.toString().trim()
    if (token) { localStorage.setItem(LICENSE_KEY, token); localStorage.removeItem(VERDICT_KEY); void checkLicense(true) }
  })
  document.querySelector('#download-archive')!.addEventListener('click', downloadArchive)
}

async function loadPolicy() {
  const target = document.querySelector<HTMLDivElement>('#policy-state')!
  try {
    const response = await fetch('/api/v1/policy', { headers: { Accept: 'application/json' } })
    if (!response.ok) throw new Error()
    const policy = await response.json() as Policy
    target.className = ''
    target.innerHTML = ''
    const state = document.createElement('p')
    state.className = policy.configured ? 'policy-ready' : 'policy-warning'
    state.textContent = policy.configured ? '● Ready to issue receipts' : '▲ Upstream needs configuration'
    target.append(state, policyRows(policy))
  } catch {
    target.className = 'policy-error'
    target.textContent = 'Policy unavailable. Check the server, then refresh.'
  }
}

function policyRows(policy: Policy) {
  const dl = document.createElement('dl')
  const entries = [
    ['Time window', `≤ ${policy.max_range_hours} hours`],
    ['Row cap', `≤ ${policy.max_rows.toLocaleString()}`],
    ['Approved routes', String(policy.allowed_paths.length)],
    ['Identity', policy.identity_header],
    ['Signature', policy.signing],
    ['Result body', 'Never recorded'],
  ]
  for (const [term, detail] of entries) { const dt = document.createElement('dt'); dt.textContent = term!; const dd = document.createElement('dd'); dd.textContent = detail!; dl.append(dt, dd) }
  return dl
}

async function loadReceipts() {
  const list = document.querySelector<HTMLDivElement>('#receipt-list')!
  list.setAttribute('aria-busy', 'true')
  const requester = document.querySelector<HTMLInputElement>('#requester')!.value.trim()
  const outcome = document.querySelector<HTMLSelectElement>('#outcome')!.value
  const params = new URLSearchParams({ limit: '50' })
  if (requester) params.set('requester', requester)
  if (outcome) params.set('outcome', outcome)
  try {
    const response = await fetch(`/api/v1/receipts?${params}`, { headers: { Accept: 'application/json' }, cache: 'no-store' })
    if (!response.ok) throw new Error()
    const data = await response.json() as { receipts: Receipt[] }
    loadedReceipts = data.receipts
    renderReceipts(data.receipts)
  } catch {
    list.innerHTML = '<div class="empty-seal" aria-hidden="true">!</div><h3>The ledger could not be reached</h3><p>Check the server connection, then use Refresh. Existing exports remain in SQLite.</p><button class="button quiet retry" type="button">Try again</button>'
    list.querySelector('button')?.addEventListener('click', () => void loadReceipts())
  } finally { list.setAttribute('aria-busy', 'false') }
}

function renderReceipts(receipts: Receipt[]) {
  const list = document.querySelector<HTMLDivElement>('#receipt-list')!
  list.innerHTML = ''
  if (!receipts.length) {
    list.classList.add('empty')
    list.innerHTML = `<div class="empty-seal" aria-hidden="true">${icon('seal')}</div><h3>No crossings match</h3><p>Send a bounded request through <code>/api/v1/exports</code>, or clear the filters to see all receipts.</p><a class="button quiet" href="#integration">View integration request</a>`
    return
  }
  list.classList.remove('empty')
  for (const receipt of receipts) list.append(receiptRow(receipt))
}

function receiptRow(receipt: Receipt) {
  const article = document.createElement('article')
  article.className = `receipt receipt-${receipt.outcome}`
  const heading = document.createElement('h3')
  const button = document.createElement('button')
  button.type = 'button'; button.className = 'receipt-summary'; button.setAttribute('aria-expanded', 'false')
  const date = new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(receipt.created_at))
  button.innerHTML = `<span class="status-stamp" aria-hidden="true">${receipt.outcome === 'allowed' ? '✓' : receipt.outcome === 'denied' ? '×' : '!'}</span><span class="receipt-who"></span><span class="receipt-time"></span><span class="receipt-cap"></span><span class="chevron">⌄</span>`
  button.querySelector('.receipt-who')!.textContent = receipt.requester
  button.querySelector('.receipt-time')!.textContent = date
  button.querySelector('.receipt-cap')!.textContent = `${receipt.outcome.replace('_', ' ')} · ${receipt.row_limit.toLocaleString()} rows`
  heading.append(button)
  const details = document.createElement('div'); details.className = 'receipt-detail'; details.hidden = true
  const grid = document.createElement('dl')
  const values = [
    ['Receipt ID', receipt.id], ['Purpose', receipt.purpose], ['Endpoint', `${receipt.method} ${receipt.endpoint}`],
    ['Range', `${formatTime(receipt.time_range.start)} → ${formatTime(receipt.time_range.end)}`], ['Fields', receipt.fields.join(', ')],
    ['Redaction', receipt.redaction_policy], ['Query SHA-256', receipt.query_sha256], ['Signature', receipt.signature],
  ]
  for (const [term, value] of values) { const wrap = document.createElement('div'); const dt = document.createElement('dt'); dt.textContent = term!; const dd = document.createElement('dd'); dd.textContent = value!; if (term === 'Receipt ID' || term === 'Query SHA-256' || term === 'Signature') dd.className = 'mono'; wrap.append(dt, dd); grid.append(wrap) }
  const actions = document.createElement('div'); actions.className = 'receipt-actions'
  const jsonLink = document.createElement('a'); jsonLink.className = 'button mini'; jsonLink.href = `/api/v1/receipts/${encodeURIComponent(receipt.id)}`; jsonLink.textContent = 'Open JSON'
  const mdLink = document.createElement('a'); mdLink.className = 'button mini'; mdLink.href = `/api/v1/receipts/${encodeURIComponent(receipt.id)}/markdown`; mdLink.textContent = 'Download Markdown'
  const verifyLink = document.createElement('a'); verifyLink.className = 'button mini'; verifyLink.href = `/api/v1/receipts/${encodeURIComponent(receipt.id)}/verify`; verifyLink.textContent = 'Verify signature'
  actions.append(jsonLink, mdLink, verifyLink); details.append(grid, actions)
  button.addEventListener('click', () => { const open = button.getAttribute('aria-expanded') === 'true'; button.setAttribute('aria-expanded', String(!open)); details.hidden = open })
  article.append(heading, details)
  return article
}

function captureReturnedLicense() {
  const params = new URLSearchParams(location.search)
  const token = params.get('license')
  if (!token) return
  localStorage.setItem(LICENSE_KEY, token)
  localStorage.removeItem(VERDICT_KEY)
  params.delete('license')
  history.replaceState({}, '', `${location.pathname}${params.size ? `?${params}` : ''}${location.hash}`)
}

async function checkLicense(force = false) {
  const token = localStorage.getItem(LICENSE_KEY)
  if (!token) return setLicense(false, 'License not installed')
  const cached = safeJson(localStorage.getItem(VERDICT_KEY)) as { valid?: boolean; checked?: number } | null
  if (!force && cached?.checked && Date.now() - cached.checked < 86_400_000) {
    setLicense(Boolean(cached.valid), cached.valid ? 'Fleet archive unlocked' : 'License no longer active')
    return
  }
  try {
    const response = await fetch(`${API}/products/${SLUG}/verify?license=${encodeURIComponent(token)}`)
    const verdict = await response.json() as { valid: boolean; reason: string }
    localStorage.setItem(VERDICT_KEY, JSON.stringify({ valid: verdict.valid, checked: Date.now() }))
    setLicense(verdict.valid, verdict.valid ? 'Fleet archive unlocked' : 'License no longer active')
    if (!verdict.valid) announce('That license is not active for this product.')
  } catch {
    if (!cached?.valid) setLicense(false, 'Verification unavailable — free tools still work')
  }
}

function setLicense(valid: boolean, text: string) {
  const status = document.querySelector('#license-status'); if (!status) return
  status.textContent = text; status.className = valid ? 'unlocked' : ''
  document.querySelector<HTMLAnchorElement>('#buy-license')!.hidden = valid
  document.querySelector<HTMLButtonElement>('#download-archive')!.hidden = !valid
}

function downloadArchive() {
  const blob = new Blob([JSON.stringify(createArchive(loadedReceipts), null, 2)], { type: 'application/json' })
  const link = document.createElement('a'); link.href = URL.createObjectURL(blob); link.download = `telemetry-export-receipts-${new Date().toISOString().slice(0, 10)}.json`; link.click(); URL.revokeObjectURL(link.href)
  announce(`Downloaded ${loadedReceipts.length} receipts.`)
}

function updateNetworkState() { const el = document.querySelector<HTMLElement>('#network-state'); if (el) el.hidden = navigator.onLine }
function formatTime(value: string) { return new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(value)) }
function announce(message: string) { const el = document.querySelector('#announcer'); if (el) el.textContent = message }
function setButtonText(selector: string, text: string) { const button = document.querySelector<HTMLButtonElement>(selector); if (!button) return; const original = button.innerHTML; button.textContent = text; setTimeout(() => { button.innerHTML = original }, 1800) }
function safeJson(value: string | null) { try { return value ? JSON.parse(value) : null } catch { return null } }
