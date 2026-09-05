import { test, expect, chromium, type Page } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'

const adminToken = 'playwright-admin-token-32-characters'

async function expectNoSeriousAxeFindings(page: Page) {
  const results = await new AxeBuilder({ page }).analyze()
  expect(results.violations.filter(item => ['serious', 'critical'].includes(item.impact || ''))).toEqual([])
}

test('public dashboard protects the receipt ledger and supports keyboard access', async ({ page }) => {
  const consoleErrors: string[] = []
  page.on('console', message => { if (message.type() === 'error') consoleErrors.push(message.text()) })
  await page.goto('/')
  await expect(page).toHaveTitle(/Telemetry Export Receipts/)
  await expect(page.getByRole('heading', { level: 1 })).toHaveText(/Record everytelemetry export/)
  await expect(page.getByRole('link', { name: 'TER. — Telemetry Export Receipts home' })).toBeVisible()
  await expect(page.getByRole('heading', { name: 'Administrator access required' })).toBeVisible()
  await page.keyboard.press('Tab')
  await expect(page.getByText('Skip to content')).toBeFocused()
  await expectNoSeriousAxeFindings(page)
  expect(consoleErrors).toEqual([])
})

test('@claim:administrator-access administrator token opens the real isolated ledger without persistent browser storage', async ({ page }) => {
  await page.goto('/')
  await page.getByLabel('Administrator token').fill(adminToken)
  await page.getByRole('button', { name: 'Open receipt desk' }).click()
  await expect(page.getByText('No receipts match')).toBeVisible()
  expect(await page.evaluate(() => localStorage.getItem('ter:admin-token'))).toBeNull()
  expect(await page.evaluate(() => sessionStorage.getItem('ter:admin-token'))).toBe(adminToken)
})

test('route changes move focus to the new page heading and announce it', async ({ page }) => {
  await page.goto('/')
  await page.locator('footer').getByRole('link', { name: 'Privacy' }).click()
  await expect(page).toHaveURL('/privacy')
  const privacyHeading = page.getByRole('heading', { level: 1, name: 'Privacy' })
  await expect(privacyHeading).toBeFocused()
  await expect(page.locator('#announcer')).toHaveText('Privacy loaded.')
  await page.goBack()
  const deskHeading = page.getByRole('heading', { level: 1, name: /Record every telemetry export/ })
  await expect(deskHeading).toBeFocused()
  await expect(page.locator('#announcer')).toHaveText(/Record every telemetry export\. loaded\./)
})

test('@claim:no-third-party-runtime first load has no analytics, ads, remote fonts, or runtime CDN requests', async ({ browser }) => {
  const context = await browser.newContext({ serviceWorkers: 'block' })
  const page = await context.newPage()
  const requests: string[] = []
  page.on('request', request => requests.push(request.url()))
  await page.goto('/')
  await expect(page.getByRole('heading', { level: 1 })).toBeVisible()
  expect(requests.every(url => new URL(url).origin === 'http://127.0.0.1:8080')).toBe(true)
  await context.close()
})

test('@claim:paid-license-unlock a returned or restored valid license unlocks the one-time archive without caching its token URL', async ({ page }) => {
  await page.route('https://api.sociobot.in/api/v1/products/telemetry-export-receipts/verify?license=*', async route => {
    await route.fulfill({ contentType: 'application/json', body: JSON.stringify({ valid: true, reason: 'ok', expires_at: null }) })
  })
  await page.goto('/')
  await page.evaluate(async () => { await navigator.serviceWorker.ready })
  await page.reload()
  await page.waitForFunction(() => navigator.serviceWorker.controller !== null)
  await page.goto('/?license=returned-license-token')
  await expect(page).toHaveURL('/')
  await expect(page.getByRole('button', { name: 'Download JSON archive' })).toBeVisible()
  await expect(page.locator('.license-ticket')).toContainText('$49')
  const cacheKeys = await page.evaluate(async () => {
    const result: string[] = []
    for (const name of await caches.keys()) {
      for (const request of await (await caches.open(name)).keys()) result.push(request.url)
    }
    return result
  })
  expect(cacheKeys.some(url => url.includes('license=') || url.includes('returned-license-token'))).toBe(false)

  await page.evaluate(() => localStorage.clear())
  await page.reload()
  await page.locator('.license-ticket summary').click()
  await page.getByLabel('License token').fill('restored-license-token')
  await page.getByRole('button', { name: 'Verify license' }).click()
  await expect(page.getByRole('button', { name: 'Download JSON archive' })).toBeVisible()
})

test('@claim:demo-sandbox demo loads useful sample data without storage or third-party requests', async ({ page }) => {
  const requests: string[] = []
  page.on('request', request => requests.push(request.url()))
  await page.goto('/demo')
  await expect(page).toHaveTitle('Demo — Telemetry Export Receipts')
  await expect(page.getByText('Demo — sample data, nothing is saved')).toBeVisible()
  await expect(page.getByText('ada@northstar.example')).toBeVisible()
  await page.getByText('ada@northstar.example').click()
  await expect(page.getByText('INC-204 checkout latency review')).toBeVisible()
  expect(await page.evaluate(() => ({ local: localStorage.length, session: sessionStorage.length }))).toEqual({ local: 0, session: 0 })
  expect(requests.every(url => new URL(url).origin === 'http://127.0.0.1:8080')).toBe(true)
})

test('@claim:offline-reload demo works offline after its first visit', async () => {
  const browser = await chromium.launch()
  const context = await browser.newContext()
  const page = await context.newPage()
  await page.goto('http://127.0.0.1:8080/demo')
  await page.evaluate(async () => { await navigator.serviceWorker.ready })
  await page.evaluate(async () => { const registration = await navigator.serviceWorker.ready; await registration.update() })
  await page.reload()
  await page.waitForFunction(() => navigator.serviceWorker.controller !== null)
  await context.setOffline(true)
  await page.reload()
  await expect(page.getByText('Demo — sample data, nothing is saved')).toBeVisible()
  await expect(page.getByText('ada@northstar.example')).toBeVisible()
  await context.close()
  await browser.close()
})

test('reduced motion disables transitions and content survives 200% text sizing', async ({ browser }) => {
  const context = await browser.newContext({ reducedMotion: 'reduce', viewport: { width: 390, height: 844 } })
  const page = await context.newPage()
  await page.goto('/demo')
  const duration = await page.getByRole('link', { name: 'Try it with sample data' }).evaluate(element => getComputedStyle(element).transitionDuration)
  expect(Number.parseFloat(duration)).toBeLessThanOrEqual(0.00001)
  const cdp = await context.newCDPSession(page)
  await cdp.send('Emulation.setPageScaleFactor', { pageScaleFactor: 2 })
  await expect(page.getByRole('heading', { level: 1 })).toBeVisible()
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth)).toBe(true)
  await context.close()
})

test('desktop, mobile, legal, and demo pages pass serious accessibility checks', async ({ browser }) => {
  const context = await browser.newContext({ serviceWorkers: 'block' })
  for (const viewport of [{ width: 1440, height: 900 }, { width: 390, height: 844 }]) {
    for (const route of ['/', '/demo', '/privacy', '/terms']) {
      const page = await context.newPage()
      await page.setViewportSize(viewport)
      await page.goto(route)
      await expect(page.locator('main')).toHaveCount(1)
      await expect(page.locator('h1')).toHaveCount(1)
      await expectNoSeriousAxeFindings(page)
      await page.close()
    }
  }
  await context.close()
})

test('all visible controls meet the 44px touch target minimum at 390px', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 })
  for (const route of ['/demo', '/privacy', '/terms']) {
    await page.goto(route)
    const undersized = await page.locator('a:visible, button:visible, input:visible, select:visible, summary:visible').evaluateAll(elements => elements.flatMap(element => {
      const box = element.getBoundingClientRect()
      return box.width < 44 || box.height < 44 ? [{ text: element.textContent?.trim(), width: box.width, height: box.height }] : []
    }))
    expect(undersized).toEqual([])
  }
})

test('protected APIs reject anonymous access and rate limits include Retry-After', async ({ request }) => {
  const anonymous = await request.get('/api/v1/receipts')
  expect(anonymous.status()).toBe(401)
  const forged = await request.post('/api/v1/exports', {
    headers: { 'X-Export-User': 'forged@example.com' },
    data: { endpoint: '/api/logs/export' },
  })
  expect(forged.status()).toBe(401)

  let limited
  for (let index = 0; index < 45; index += 1) {
    limited = await request.get('/api/v1/policy', { headers: { 'X-Forwarded-For': '198.51.100.44' } })
    if (limited.status() === 429) break
  }
  expect(limited?.status()).toBe(429)
  expect(limited?.headers()['retry-after']).toBeTruthy()
})

test('@claim:denied-receipt identified policy denials return a verifiable receipt without raw query data', async ({ request }) => {
  const response = await request.post('/api/v1/exports', {
    headers: {
      'X-TER-Admin-Token': adminToken,
      'X-Export-User': 'claim@example.com',
      'X-Forwarded-For': '203.0.113.81',
    },
    data: {
      endpoint: '/api/logs/export',
      start: '2026-01-01T00:00:00Z',
      end: '2026-01-03T00:00:00Z',
      row_limit: 10,
      fields: ['message'],
      redaction_policy: 'pii-basic',
      purpose: 'audit review',
      query: { secret_filter: 'private-query-marker' },
    },
  })
  expect(response.status()).toBe(403)
  const body = await response.json() as { receipt_id: string }
  expect(body.receipt_id).toBeTruthy()
  const receipt = await request.get(`/api/v1/receipts/${body.receipt_id}`, { headers: { 'X-TER-Admin-Token': adminToken, 'X-Forwarded-For': '203.0.113.81' } })
  expect(receipt.status()).toBe(200)
  expect(await receipt.text()).not.toContain('private-query-marker')
  const verification = await request.get(`/api/v1/receipts/${body.receipt_id}/verify`, { headers: { 'X-TER-Admin-Token': adminToken, 'X-Forwarded-For': '203.0.113.81' } })
  expect(verification.status()).toBe(200)
  expect(await verification.json()).toMatchObject({ valid: true, algorithm: 'HMAC-SHA256' })
})

test('responses include the required security policy headers', async ({ request }) => {
  const response = await request.get('/')
  expect(response.headers()['strict-transport-security']).toContain('max-age=31536000')
  expect(response.headers()['content-security-policy']).toContain("frame-ancestors 'none'")
  expect(response.headers()['x-content-type-options']).toBe('nosniff')
})

test('unknown routes return the designed 404 page', async ({ page }) => {
  const response = await page.goto('/missing-page')
  expect(response?.status()).toBe(404)
  await expect(page).toHaveTitle('Page not found — Telemetry Export Receipts')
  await expect(page.getByRole('heading', { level: 1, name: 'Page not found' })).toBeVisible()
})
