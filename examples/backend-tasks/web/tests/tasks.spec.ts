import { test, expect, type Page } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'
import { spawn, type ChildProcess } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { createClient, type Client } from '@tsrust-example/tasks-api'
import { createHttpTransport } from '@tsrust-example/tasks-api/http'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../../..')
const token = 'a'.repeat(64)
let child: ChildProcess, base: string, directory: string, api: Client
let exited: Promise<number | null>
function bounded<T>(promise: Promise<T>, milliseconds = 15000): Promise<T> {
  let timer: ReturnType<typeof setTimeout>
  return Promise.race([promise, new Promise<T>((_, reject) => { timer = setTimeout(() => reject(new Error('Native task fixture timed out')), milliseconds) })]).finally(() => clearTimeout(timer))
}
test.beforeAll(async () => {
  directory = fs.mkdtempSync(path.join(root, 'target/task-web-'))
  child = spawn(path.join(root, 'target/task-http-witness' + (process.platform === 'win32' ? '.exe' : '')), [], { cwd: directory, windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'], env: {
    ...process.env, TASK_TEST_DATABASE_URL: `sqlite:${path.join(directory, 'tasks.sqlite').replaceAll('\\', '/')}?mode=rwc`,
    TASK_TEST_MIGRATIONS: path.join(root, 'examples/backend-tasks/migrations'), TASK_TEST_TOKEN: token,
  } })
  exited = new Promise((resolve, reject) => { child.once('exit', resolve); child.once('error', reject) })
  let output = '', errors = ''
  child.stderr!.on('data', chunk => { errors += chunk })
  base = await bounded(new Promise<string>((resolve, reject) => {
    child.once('error', reject)
    child.once('exit', code => reject(new Error(`Native fixture exited ${code}: ${errors}`)))
    child.stdout!.on('data', chunk => {
      output += chunk
      const line = output.split('\n').slice(0, -1).find(line => line.startsWith('{'))
      if (line) { try { const ready = JSON.parse(line); expect(ready.address).toMatch(/^127\.0\.0\.1:\d+$/); resolve(`http://${ready.address}`) } catch (error) { reject(error) } }
    })
  }))
  api = createClient(createHttpTransport({ url: base + '/api', headers: { authorization: `Bearer ${token}` } }))
})
test.afterAll(async () => {
  try {
    if (child?.exitCode === null) {
      if (base) await fetch(base + '/__fixture_stop', { method: 'POST', headers: { authorization: `Bearer ${token}` }, signal: AbortSignal.timeout(5000) })
      else child.kill()
      expect(await bounded(exited)).toBe(0)
    }
  } finally {
    if (child?.exitCode === null) { child.kill(); await bounded(exited) }
    if (directory) {
      expect(path.dirname(fs.realpathSync(directory))).toBe(fs.realpathSync(path.join(root, 'target')))
      expect(path.basename(directory)).toMatch(/^task-web-/)
      fs.rmSync(directory, { recursive: true, force: true })
    }
  }
})
test.beforeEach(async () => {
  while (true) {
    const result = await api.tasks.list({ after: 0, limit: 100 })
    expect(result.kind).toBe('page')
    if (result.kind !== 'page' || !result.items.length) break
    for (const item of result.items) expect((await api.tasks.delete({ id: item.id })).kind).toBe('deleted')
  }
})
async function connect(page: Page, access = token) {
  await page.goto('/')
  await page.getByLabel('API address', { exact: true }).fill(base + '/api')
  await page.getByLabel('Access token', { exact: true }).fill(access)
  await page.getByRole('button', { name: 'Connect', exact: true }).click()
}
test('native CRUD, keyboard, UTF-8 validation, memory-only connection and accessibility', async ({ page }) => {
  const errors: string[] = []
  page.on('pageerror', error => errors.push(error.message))
  await page.goto('/')
  expect((await new AxeBuilder({ page }).analyze()).violations).toEqual([])
  await connect(page)
  await expect(page.getByText('A clear start', { exact: true })).toBeVisible()
  await page.getByRole('button', { name: 'Add task', exact: true }).click()
  await expect(page.getByRole('alert')).toContainText('Use 1 to 200 UTF-8 bytes')
  await page.getByLabel('New task', { exact: true }).fill('🚀'.repeat(51))
  await page.getByRole('button', { name: 'Add task', exact: true }).click()
  await expect(page.getByRole('alert')).toContainText('UTF-8')
  await page.getByLabel('New task', { exact: true }).fill('Ship the prototype')
  await page.getByLabel('New task', { exact: true }).press('Enter')
  await expect(page.getByRole('list', { name: 'Tasks' })).toContainText('Ship the prototype')
  const checked = page.getByRole('checkbox', { name: 'Mark Ship the prototype complete' })
  await checked.focus()
  await checked.press('Space')
  await expect(page.getByRole('checkbox', { name: 'Mark Ship the prototype incomplete' })).toBeChecked()
  await page.getByRole('button', { name: 'Edit Ship the prototype', exact: true }).click()
  await page.getByLabel('Task title', { exact: true }).fill('Ship native tasks')
  await page.getByLabel('Task title', { exact: true }).press('Enter')
  await expect(page.getByRole('button', { name: 'Edit Ship native tasks', exact: true })).toBeVisible()
  expect((await new AxeBuilder({ page }).analyze()).violations).toEqual([])
  await page.setViewportSize({ width: 375, height: 812 })
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true)
  await page.screenshot({ path: path.join(root, 'target/task-web-mobile.png'), fullPage: true })
  await page.getByRole('button', { name: 'Delete Ship native tasks', exact: true }).click()
  await page.getByRole('button', { name: 'Keep task', exact: true }).click()
  await expect(page.getByRole('list', { name: 'Tasks' })).toContainText('Ship native tasks')
  await page.getByRole('button', { name: 'Delete Ship native tasks', exact: true }).click()
  await page.getByRole('button', { name: 'Confirm delete', exact: true }).click()
  await expect(page.getByText('A clear start', { exact: true })).toBeVisible()
  await page.getByRole('button', { name: 'Disconnect', exact: true }).click()
  await expect(page.getByLabel('Access token', { exact: true })).toHaveValue('')
  expect(await page.evaluate(() => ({ local: { ...localStorage }, session: { ...sessionStorage } }))).toEqual({ local: {}, session: {} })
  expect(errors).toEqual([])
})
test('loading, denial, failed write and retry use actual service boundaries', async ({ page }) => {
  await connect(page, 'b'.repeat(64))
  await expect(page.getByRole('alert')).toContainText('access was denied')
  await page.getByRole('button', { name: 'Disconnect', exact: true }).click()
  let release!: () => void
  const held = new Promise<void>(resolve => { release = resolve })
  await page.route(base + '/api', async route => { if (route.request().method() === 'POST') await held; await route.continue() })
  await connect(page)
  await expect(page.locator('p:not([role])').filter({ hasText: /^Loading tasks…$/ })).toBeVisible()
  release()
  await expect(page.getByText('A clear start', { exact: true })).toBeVisible()
  await page.unroute(base + '/api')
  await page.route(base + '/api', async route => route.request().method() === 'POST' ? route.abort() : route.continue())
  await page.getByLabel('New task', { exact: true }).fill('Unconfirmed write')
  await page.getByRole('button', { name: 'Add task', exact: true }).click()
  await expect(page.getByRole('alert')).toContainText('could not be confirmed')
  await expect(page.getByLabel('New task', { exact: true })).toHaveValue('Unconfirmed write')
  await page.unroute(base + '/api')
  await page.getByRole('button', { name: 'Refresh', exact: true }).click()
  await expect(page.getByText('A clear start', { exact: true })).toBeVisible()
})
test('cursor pages retain isolated native results', async ({ page }) => {
  for (let index = 0; index < 101; index++) expect((await api.tasks.create({ title: `Task ${index}` })).kind).toBe('task')
  await connect(page)
  await expect(page.getByRole('list', { name: 'Tasks' }).getByRole('listitem')).toHaveCount(100)
  await page.getByRole('button', { name: 'Next page', exact: true }).click()
  await expect(page.getByRole('list', { name: 'Tasks' }).getByRole('listitem')).toHaveCount(1)
  await expect(page.getByRole('list', { name: 'Tasks' })).toContainText('Task 100')
  await page.getByRole('button', { name: 'Previous page', exact: true }).click()
  await expect(page.getByRole('list', { name: 'Tasks' }).getByRole('listitem')).toHaveCount(100)
})

test('upstream Vite HMR updates the editable app without a navigation', async ({ page }) => {
  test.skip(process.env.TASK_WEB_PREVIEW === '1', 'HMR belongs to the development server')
  const source = path.join(root, 'examples/backend-tasks/web/src/Tasks.tsx')
  const original = fs.readFileSync(source, 'utf8')
  await page.goto('/')
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Make room for what matters.')
  await page.evaluate(() => { Object.assign(window, { tsrustHmrWitness: 'retained-document' }) })
  try {
    fs.writeFileSync(source, original.replace('Make room for what matters.', 'An editable upstream React app.'))
    await expect(page.getByRole('heading', { level: 1 })).toHaveText('An editable upstream React app.')
    expect(await page.evaluate(() => Reflect.get(window, 'tsrustHmrWitness'))).toBe('retained-document')
  } finally {
    fs.writeFileSync(source, original)
    await expect(page.getByRole('heading', { level: 1 })).toHaveText('Make room for what matters.')
  }
})
