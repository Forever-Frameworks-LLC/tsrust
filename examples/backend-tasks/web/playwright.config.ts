import { defineConfig } from '@playwright/test'

export default defineConfig({
  testDir: './tests',
  workers: 1,
  fullyParallel: false,
  timeout: 30000,
  reporter: 'list',
  outputDir: '../../../target/task-web-results',
  use: { baseURL: 'http://127.0.0.1:5173', headless: true, trace: 'retain-on-failure' },
  webServer: {
    command: `node node_modules/vite/bin/vite.js ${process.env.TASK_WEB_PREVIEW === '1' ? 'preview' : ''} --host 127.0.0.1 --port 5173 --strictPort`,
    url: 'http://127.0.0.1:5173',
    reuseExistingServer: false,
  },
})
