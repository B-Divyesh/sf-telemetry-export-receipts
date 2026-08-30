import { defineConfig } from '@playwright/test'

export default defineConfig({
  testDir: './tests/e2e',
  use: { baseURL: 'http://127.0.0.1:8080', trace: 'retain-on-failure' },
  webServer: {
    command: 'npm run build && TER_ADMIN_TOKEN=playwright-admin-token-32-characters TER_RECEIPT_SIGNING_KEY=playwright-signing-key-32-characters DATABASE_URL=sqlite::memory: cargo run',
    url: 'http://127.0.0.1:8080/health',
    reuseExistingServer: true,
    timeout: 120_000,
  },
})
