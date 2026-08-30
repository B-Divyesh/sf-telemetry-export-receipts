import './styles.css'
import { createArchive } from './archive'

const SLUG = 'telemetry-export-receipts'
const LICENSE_KEY = `sb_license:${SLUG}`
const VERDICT_KEY = `${LICENSE_KEY}:verdict`
const ADMIN_KEY = 'ter:admin-token'
const API = 'https://api.sociobot.in/api/v1'
const isDemo = location.pathname === '/demo' || new URLSearchParams(location.search).get('demo') === '1'
const buildId = (import.meta.env.VITE_BUILD_SHA || 'development').slice(0, 12)

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

const sampleReceipts: Receipt[] = [
  {
    schema: 'telemetry-export-receipt.v1', id: '0198f8b2-7f51-7000-a800-demo00000001', created_at: '2026-08-28T09:42:00Z',
    requester: 'ada@northstar.example', purpose: 'INC-204 checkout latency review', endpoint: '/api/traces/export', method: 'POST',
    time_range: { start: '2026-08-28T08:42:00Z', end: '2026-08-28T09:42:00Z' }, row_limit: 5000,
    fields: ['timestamp', 'service', 'duration_ms', 'trace_id'], redaction_policy: 'pii-basic', query_sha256: '8a31f9305a386c587479887394a488a0a8047534982e03b8a7b0317c12929573',
    policy: { max_range_seconds: 86400, max_rows: 10000, authorization_forwarded: true, result_body_recorded: false }, outcome: 'allowed', upstream_status: 200, denial_reason: null,
    signature: '4fc874eec5effabf714a916735901b842e589a59acadaeef4bff4d00c8f6f11d',
  },
  {
    schema: 'telemetry-export-receipt.v1', id: '0198f8a9-1c20-7000-a800-demo00000002', created_at: '2026-08-28T09:31:00Z',
    requester: 'mina@northstar.example', purpose: 'Weekly error budget review', endpoint: '/api/logs/export', method: 'GET',
    time_range: { start: '2026-08-26T09:31:00Z', end: '2026-08-28T09:31:00Z' }, row_limit: 10000,
    fields: ['timestamp', 'service', 'message'], redaction_policy: 'strict', query_sha256: 'e11025ec4f93b62324d526b971668d5fae01be9b21ebb799d7ddfe831a27cf77',
    policy: { max_range_seconds: 86400, max_rows: 10000, authorization_forwarded: true, result_body_recorded: false }, outcome: 'denied', upstream_status: null, denial_reason: 'Time range exceeds the 24 hour policy cap.',
    signature: 'f1727f24da2913f59324ee71bee3f26ecf93f62b0d2022a84f55c06f7d0a912a',
  },
  {
    schema: 'telemetry-export-receipt.v1', id: '0198f896-ae9f-7000-a800-demo00000003', created_at: '2026-08-28T09:12:00Z',
    requester: 'jo@northstar.example', purpose: 'Customer timeout investigation', endpoint: '/api/metrics/export', method: 'POST',
    time_range: { start: '2026-08-28T08:12:00Z', end: '2026-08-28T09:12:00Z' }, row_limit: 2000,
    fields: ['timestamp', 'metric', 'value'], redaction_policy: 'pii-basic', query_sha256: '901bf11921f6f86cef887253486908298620a117d012cce062388f220a24f07d',
    policy: { max_range_seconds: 86400, max_rows: 10000, authorization_forwarded: true, result_body_recorded: false }, outcome: 'upstream_error', upstream_status: 503, denial_reason: 'The upstream returned an error.',
    signature: '08485ace727cb81dc7aa47e02dfb88472ba834ef54b01060bf9214b0988ef919',
  },
]

let loadedReceipts: Receipt[] = []

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
    ${isDemo ? '<aside class="demo-banner" aria-label="Demo mode"><strong>Demo — sample data, nothing is saved</strong><span><a href="/demo">Reset demo</a><a href="/">Start for real</a></span></aside>' : ''}
    <header class="site-header">
      <a class="brand" href="/" aria-label="TER. — Telemetry Export Receipts home"><span class="brand-mark">${icon('gate')}</span><span>TER<span class="brand-dot">.</span></span></a>
      <nav aria-label="Primary"><a ${page === 'desk' && !isDemo ? 'aria-current="page"' : ''} href="/">Receipt desk</a><a ${isDemo ? 'aria-current="page"' : ''} href="/demo">Demo</a><a href="/#integration">Integrate</a><a href="/#license">License</a></nav>
      <span class="boundary"><i></i> Egress boundary</span>
    </header>
    ${content}
    <footer><p><span class="footer-seal">${icon('seal')}</span> Built for operators who need proof, not another telemetry store.</p><nav aria-label="Legal"><a href="/privacy">Privacy</a><a href="/terms">Terms</a><a href="https://github.com/B-Divyesh/sf-telemetry-export-receipts">Source</a></nav><small>Built by Param Factory · Build ${buildId} · Original generated hero art · No analytics</small></footer>
    <div id="announcer" class="sr-only" aria-live="polite"></div>`
}

function legalPage(kind: 'privacy' | 'terms') {
  const privacy = `
    <main id="main" class="legal"><p class="eyebrow">Legal / plain language</p><h1>Privacy</h1><p class="lede">Telemetry Export Receipts is designed to know about the export, not the exported data.</p>
    <h2>What this installation stores</h2><p>For each export attempt, the server stores its requester, purpose, endpoint, bounds, policy, outcome, and signature. It stores selected field names and a query hash. It never stores upstream authorization credentials or result bodies.</p>
    <h2>Where it lives</h2><p>Receipts stay in the SQLite database controlled by the self-hosting operator. This web interface includes no analytics, advertising, tracking pixels, or third-party runtime scripts.</p>
    <h2>Licenses</h2><p>This browser stores a purchased license token and its time-limited verification result. Verification goes to Sociobot, the merchant of record. Never paste an observability access token into the license field.</p>
    <h2>Your controls</h2><p>Administrator access tokens stay in sessionStorage and disappear when the tab closes. Operators control retention by managing the local database. Clear this browser's site data to remove a locally stored license. Contact the operator of your installation for access or deletion requests.</p><p><em>Effective 28 August 2026.</em></p></main>`
  const terms = `
    <main id="main" class="legal"><p class="eyebrow">Legal / plain language</p><h1>Terms</h1><p class="lede">Use this software as one accountable layer in your export path—not as a replacement for upstream permissions.</p>
    <h2>Service</h2><p>The software enforces configured time, row, path, and redaction bounds and creates signed records. You must authenticate users and accept identity headers only from your auth proxy. You must also secure the signing key and test each upstream API integration.</p>
    <h2>One-time license</h2><p>Fleet archive costs US$49 once and adds bulk receipt packaging in the operator UI. The core proxy, safety policy, receipt signing, individual downloads, and accessibility remain free. Sociobot/Dodo is the merchant of record. A refund revokes the license.</p>
    <h2>Warranty</h2><p>The open-source software is provided under the MIT License, without warranty. Review and test configuration before using it for a compliance program.</p><p><em>Effective 28 August 2026.</em></p></main>`
  document.title = `${kind === 'privacy' ? 'Privacy' : 'Terms'} — Telemetry Export Receipts`
  setCanonical(`/${kind}`)
  shell(kind === 'privacy' ? privacy : terms, 'legal')
}

function setCanonical(route: string) {
  document.querySelector<HTMLLinkElement>('#canonical')?.setAttribute('href', `https://telemetry-export-receipts.sociobot.in${route}`)
}

function notFoundPage() {
  document.title = 'Page not found — Telemetry Export Receipts'
  setCanonical('/404')
  shell('<main id="main" class="legal"><p class="eyebrow">404 / Not found</p><h1>Page not found</h1><p class="lede">This address does not match a receipt-desk page.</p><a class="button primary" href="/">Return to receipt desk</a></main>', 'missing')
}

if (path === '/privacy' || path === '/terms') {
  legalPage(path.slice(1) as 'privacy' | 'terms')
} else if (path === '/' || isDemo) {
  if (isDemo) {
    document.title = 'Demo — Telemetry Export Receipts'
    setCanonical('/demo')
  }
  shell(`
  <main id="main">
    <section class="hero" aria-labelledby="hero-title">
      <div class="hero-copy"><p class="eyebrow"><span>01</span> Signed at the boundary</p><h1 id="hero-title">Record every<br><em>telemetry export.</em></h1><p class="lede">For observability teams, this proxy limits downloads and records who requested each one.</p><div class="hero-actions"><a class="button primary" href="/demo">Try it with sample data</a><a class="button quiet" href="#desk">Use your installation <span>↓</span></a></div><ul class="proof-points"><li>${icon('seal')} Signed JSON and Markdown</li><li>${icon('gate')} Result bodies never stored</li><li>${icon('download')} Optional archive costs $49 once</li></ul></div>
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
    <section id="integration" class="integration" aria-labelledby="integration-title"><div><p class="eyebrow"><span>03</span> One guarded route</p><h2 id="integration-title">Keep permissions.<br>Bound the query.</h2><p>The proxy forwards your existing <code>Authorization</code> and <code>Cookie</code> headers only to the configured upstream. Your trusted auth proxy supplies requester identity. The upstream body and status return with receipt ID and signature headers.</p></div><div class="code-panel"><div class="code-head"><span><i></i><i></i><i></i></span><button id="copy-curl" type="button">${icon('copy')} Copy request</button></div><pre><code id="curl-example">curl -X POST https://your-host/api/v1/exports \\
  -H 'Authorization: Bearer …' \\
  -H 'X-TER-Admin-Token: …' \\
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
    <section id="license" class="license-section" aria-labelledby="license-title"><div class="license-copy"><p class="eyebrow"><span>04</span> Optional paid archive</p><h2 id="license-title">Export a receipt archive.</h2><p>The core proxy and individual signed receipts stay free. Fleet archive packages the loaded audit set for an offline review or handoff.</p><ul><li>Bulk JSON archive from current filters</li><li>Portable, no recurring data-volume fee</li><li>One installation license</li></ul></div><div class="license-ticket"><p class="ticket-kicker">Fleet archive</p><p class="price"><strong>$49</strong> <span>once</span></p><p id="license-status">License not installed</p><a id="buy-license" class="button primary" href="https://api.sociobot.in/api/v1/products/telemetry-export-receipts/checkout">Buy one-time license <span>↗</span></a><button id="download-archive" class="button primary" type="button" aria-label="Download JSON archive" hidden>${icon('download')} Download JSON archive</button><details><summary>Have a license? Restore it</summary><form id="license-form"><label for="license-token">License token</label><input id="license-token" name="license" autocomplete="off" spellcheck="false"><button type="submit" class="button quiet" aria-label="Verify license">Verify license</button></form></details><p class="legal-note">Sociobot/Dodo is merchant of record. Refunds are handled there. <a href="/terms">Terms</a> · <a href="/privacy">Privacy</a></p></div></section>
  </main>`, 'desk')
  void initDesk()
} else {
  notFoundPage()
}

async function initDesk() {
  captureReturnedLicense()
  bindInteractions()
  updateNetworkState()
  addEventListener('online', updateNetworkState)
  addEventListener('offline', updateNetworkState)
  if (isDemo) {
    renderDemoPolicy()
    loadReceipts()
    setLicense(false, 'License not used in demo')
    document.querySelector<HTMLAnchorElement>('#buy-license')!.hidden = true
    document.querySelector<HTMLDetailsElement>('.license-ticket details')!.hidden = true
  } else {
    await Promise.all([loadPolicy(), loadReceipts(), checkLicense()])
  }
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

function renderDemoPolicy() {
  const target = document.querySelector<HTMLDivElement>('#policy-state')!
  target.className = ''
  target.innerHTML = ''
  const state = document.createElement('p')
  state.className = 'policy-ready'
  state.textContent = '● Sample policy active'
  target.append(state, policyRows({ configured: true, allowed_paths: ['/api/logs/export', '/api/traces/export', '/api/metrics/export'], max_range_hours: 24, max_rows: 10000, redaction_policies: ['pii-basic', 'strict'], identity_header: 'x-export-user', signing: 'HMAC-SHA256' }))
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
  if (isDemo) {
    const matches = sampleReceipts.filter(receipt => (!requester || receipt.requester.toLowerCase().includes(requester.toLowerCase())) && (!outcome || receipt.outcome === outcome))
    loadedReceipts = matches
    renderReceipts(matches)
    list.setAttribute('aria-busy', 'false')
    return
  }
  if (!sessionStorage.getItem(ADMIN_KEY)) return renderAdminAccess(list)
  try {
    const response = await fetch(`/api/v1/receipts?${params}`, { headers: adminHeaders(), cache: 'no-store' })
    if (response.status === 401) return renderAdminAccess(list, true)
    if (!response.ok) throw new Error()
    const data = await response.json() as { receipts: Receipt[] }
    loadedReceipts = data.receipts
    renderReceipts(data.receipts)
  } catch {
    list.innerHTML = '<div class="empty-seal" aria-hidden="true">!</div><h3>The ledger could not be reached</h3><p>Check the server connection, then use Refresh. Existing exports remain in SQLite.</p><button class="button quiet retry" type="button">Try again</button>'
    list.querySelector('button')?.addEventListener('click', () => void loadReceipts())
  } finally { list.setAttribute('aria-busy', 'false') }
}

function adminHeaders() {
  const headers: Record<string, string> = { Accept: 'application/json' }
  const token = sessionStorage.getItem(ADMIN_KEY)
  if (token) headers['X-TER-Admin-Token'] = token
  return headers
}

function renderAdminAccess(list: HTMLDivElement, rejected = false) {
  loadedReceipts = []
  list.classList.add('empty')
  list.innerHTML = `<div class="empty-seal" aria-hidden="true">⌁</div><h3>Administrator access required</h3><p id="admin-help">Enter the token from the server’s admin-access.key file. It stays in this browser tab.</p>${rejected ? '<p id="admin-error" class="form-error" role="alert">That token was not accepted. Check the file and try again.</p>' : ''}<form id="admin-access" class="admin-access"><label for="admin-token">Administrator token</label><input id="admin-token" name="token" type="password" autocomplete="off" aria-describedby="admin-help${rejected ? ' admin-error' : ''}" required><button class="button primary" type="submit">Open receipt desk</button></form>`
  list.querySelector<HTMLFormElement>('#admin-access')?.addEventListener('submit', event => {
    event.preventDefault()
    const token = new FormData(event.currentTarget as HTMLFormElement).get('token')?.toString().trim()
    if (!token) return
    sessionStorage.setItem(ADMIN_KEY, token)
    announce('Administrator token saved for this tab.')
    void loadReceipts()
  })
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
  if (!isDemo) {
    const jsonButton = document.createElement('button'); jsonButton.type = 'button'; jsonButton.className = 'button mini'; jsonButton.textContent = 'Download JSON'; jsonButton.addEventListener('click', () => void downloadProtectedReceipt(`/api/v1/receipts/${encodeURIComponent(receipt.id)}`, `${receipt.id}.json`))
    const mdButton = document.createElement('button'); mdButton.type = 'button'; mdButton.className = 'button mini'; mdButton.textContent = 'Download Markdown'; mdButton.addEventListener('click', () => void downloadProtectedReceipt(`/api/v1/receipts/${encodeURIComponent(receipt.id)}/markdown`, `${receipt.id}.md`))
    const verifyButton = document.createElement('button'); verifyButton.type = 'button'; verifyButton.className = 'button mini'; verifyButton.textContent = 'Verify signature'; verifyButton.addEventListener('click', () => void verifyProtectedReceipt(receipt.id))
    actions.append(jsonButton, mdButton, verifyButton)
  } else {
    const note = document.createElement('p'); note.className = 'demo-note'; note.textContent = 'Sample receipt — no server record was created.'; actions.append(note)
  }
  details.append(grid, actions)
  button.addEventListener('click', () => { const open = button.getAttribute('aria-expanded') === 'true'; button.setAttribute('aria-expanded', String(!open)); details.hidden = open })
  article.append(heading, details)
  return article
}

async function downloadProtectedReceipt(url: string, filename: string) {
  try {
    const response = await fetch(url, { headers: adminHeaders(), cache: 'no-store' })
    if (!response.ok) throw new Error()
    const blob = await response.blob()
    const link = document.createElement('a'); link.href = URL.createObjectURL(blob); link.download = filename; link.click(); URL.revokeObjectURL(link.href)
    announce(`Downloaded ${filename}.`)
  } catch { announce('The receipt could not be downloaded. Check administrator access, then try again.') }
}

async function verifyProtectedReceipt(id: string) {
  try {
    const response = await fetch(`/api/v1/receipts/${encodeURIComponent(id)}/verify`, { headers: adminHeaders(), cache: 'no-store' })
    if (!response.ok) throw new Error()
    const result = await response.json() as { valid: boolean }
    announce(result.valid ? 'The receipt signature is valid.' : 'The receipt signature is not valid.')
  } catch { announce('The signature could not be checked. Check administrator access, then try again.') }
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
