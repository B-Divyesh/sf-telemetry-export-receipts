import { test, expect } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'

test('dashboard renders its empty, policy, and keyboard states', async ({ page }) => {
  const consoleErrors: string[] = []
  page.on('console', message => { if (message.type() === 'error') consoleErrors.push(message.text()) })
  await page.goto('/')
  await expect(page).toHaveTitle(/Telemetry Export Receipts/)
  await expect(page.getByRole('heading', { level: 1 })).toHaveText(/Every exportleaves proof/)
  await expect(page.getByRole('link', { name: 'TER. — Telemetry Export Receipts home' })).toBeVisible()
  await expect(page.getByText('No crossings match')).toBeVisible()
  await page.keyboard.press('Tab')
  await expect(page.getByText('Skip to content')).toBeFocused()
  const results = await new AxeBuilder({ page }).analyze()
  expect(results.violations.filter(item => ['serious', 'critical'].includes(item.impact || ''))).toEqual([])
  expect(consoleErrors).toEqual([])
})

test('legal routes have one main heading', async ({ page }) => {
  for (const route of ['/privacy', '/terms']) {
    await page.goto(route)
    await expect(page.locator('main')).toHaveCount(1)
    await expect(page.locator('h1')).toHaveCount(1)
  }
})
