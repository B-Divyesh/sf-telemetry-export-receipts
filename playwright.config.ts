import { defineConfig } from '@playwright/test'

export default defineConfig({
  testDir: './tests/e2e',
  use: { baseURL: 'http://127.0.0.1:8080', trace: 'retain-on-failure' },
  webServer: {
    command: 'cargo run',
    url: 'http://127.0.0.1:8080/health',
    reuseExistingServer: true,
    timeout: 120_000,
  },
})
