import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import net from 'node:net';
import { spawn, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { fileURLToPath, pathToFileURL } from 'node:url';

assert.equal(process.platform, 'win32', 'This witness proves the Windows package only');
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const example = path.join(root, 'examples/backend-tasks');
const packaged = path.join(example, 'native/out/Tsrust Tasks-win32-x64');
const temporary = fs.mkdtempSync(path.join(os.tmpdir(), 'tsrust-desktop-package-'));
const application = path.join(temporary, 'app');
const { chromium } = await import(pathToFileURL(path.join(example, 'web/node_modules/@playwright/test/index.mjs')));
const { deadline } = await import(pathToFileURL(path.join(example, 'native/src/backend.ts')));
const { getCurrentFuseWire, FuseV1Options, FuseState } = await import(pathToFileURL(path.join(example, 'native/node_modules/@electron/fuses/dist/index.js')));
fs.cpSync(packaged, application, { recursive: true });
const executable = path.join(application, 'tsrust-tasks.exe');
const backendExecutable = path.join(application, 'resources/backend/tasks-backend.exe');
const sessions = [];
async function port() {
  const server = net.createServer(); await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const result = server.address().port; await new Promise(resolve => server.close(resolve)); return result;
}
async function waitFor(check, message) {
  for (let attempt = 0; attempt < 200; attempt++) { const value = await check(); if (value) return value; await new Promise(resolve => setTimeout(resolve, 100)); }
  throw new Error(message);
}
function alive(pid) { try { process.kill(pid, 0); return true; } catch (error) { if (error.code === 'ESRCH') return false; throw error; } }
async function launch(name) {
  const debugPort = await port(), data = path.join(temporary, name); fs.mkdirSync(data, { recursive: true });
  const environment = { ...process.env, TSRUST_HEADLESS: '1', TSRUST_DATA_DIR: data,
    PATH: path.join(process.env.SystemRoot, 'System32') };
  delete environment.TSRUST; delete environment.CARGO_HOME; delete environment.RUSTUP_HOME;
  const child = spawn(executable, [`--remote-debugging-port=${debugPort}`, '--remote-debugging-address=127.0.0.1', '--disable-gpu'], {
    cwd: application, env: environment, windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'],
  });
  const record = { child, output: '', errors: '', browser: undefined, backendPid: undefined };
  sessions.push(record);
  record.exited = new Promise((resolve, reject) => { child.once('error', reject); child.once('close', resolve); });
  child.stdout.on('data', chunk => { record.output += chunk; for (const line of record.output.split('\n').slice(0, -1)) { try { const value = JSON.parse(line); if (value.kind === 'desktop-backend-ready') record.backendPid = value.pid; } catch {} } });
  child.stderr.on('data', chunk => { record.errors += chunk; });
  await waitFor(async () => {
    if (child.exitCode !== null) throw new Error(`Packaged app exited ${child.exitCode}: ${record.errors}`);
    try { const response = await fetch(`http://127.0.0.1:${debugPort}/json/version`, { signal: AbortSignal.timeout(500) }); return response.ok; } catch { return false; }
  }, 'Packaged app did not expose its test-only Chromium debug connection');
  record.browser = await chromium.connectOverCDP(`http://127.0.0.1:${debugPort}`);
  record.page = await waitFor(() => record.browser.contexts()[0]?.pages()[0], 'Packaged app did not create a hidden window');
  return record;
}
async function close(record) {
  await record.page.close(); assert.equal(await deadline(record.exited, 15000, 'Packaged app did not quit'), 0, record.errors);
  await record.browser.close().catch(() => {});
  if (record.backendPid) await waitFor(() => !alive(record.backendPid), 'Native backend outlived normal quit');
}
try {
  assert.ok(fs.existsSync(path.join(application, 'resources/app.asar')));
  const fuses = await getCurrentFuseWire(executable);
  for (const flag of [FuseV1Options.RunAsNode, FuseV1Options.EnableNodeOptionsEnvironmentVariable, FuseV1Options.EnableNodeCliInspectArguments, FuseV1Options.GrantFileProtocolExtraPrivileges]) assert.equal(fuses[flag], FuseState.DISABLE);
  for (const flag of [FuseV1Options.EnableCookieEncryption, FuseV1Options.EnableEmbeddedAsarIntegrityValidation, FuseV1Options.OnlyLoadAppFromAsar]) assert.equal(fuses[flag], FuseState.ENABLE);
  const runtime = JSON.parse(fs.readFileSync(path.join(application, 'resources/backend/runtime-provenance.json'), 'utf8').replace(/^\uFEFF/, ''));
  assert.ok(runtime.files.some(file => file.name.toLowerCase() === 'vcruntime140.dll'));
  assert.ok(fs.existsSync(path.join(application, 'resources/backend/vcruntime140.dll')));
  for (const file of runtime.files) {
    assert.equal(path.basename(file.name), file.name);
    assert.equal(createHash('sha256').update(fs.readFileSync(path.join(application, 'resources/backend', file.name))).digest('hex'), file.sha256);
  }
  const normal = await launch('normal-data'); const page = normal.page;
  await page.getByText('Connected to your desktop service', { exact: true }).waitFor();
  const titlebar = page.getByRole('banner', { name: 'Application title bar' });
  await titlebar.getByText('Tsrust Tasks', { exact: true }).waitFor();
  assert.deepEqual(await titlebar.evaluate(element => ({ height: element.getBoundingClientRect().height, drag: getComputedStyle(element).getPropertyValue('app-region') })), { height: 44, drag: 'drag' });
  assert.equal(await titlebar.evaluate(element => element.getBoundingClientRect().width === innerWidth), true, 'header spans the window, including beneath caption controls');
  assert.equal(await page.evaluate(() => navigator.windowControlsOverlay.getTitlebarAreaRect().height), 43, 'native overlay leaves the full-width bottom border exposed');
  assert.ok(normal.backendPid);
  const loaded = spawnSync('powershell.exe', ['-NoProfile', '-NonInteractive', '-Command', "$module=(Get-Process -Id ([int]$env:TSRUST_BACKEND_PROOF_PID)).Modules | Where-Object ModuleName -eq 'vcruntime140.dll'; if (-not $module) { throw 'Loaded CRT not found' }; $module.FileName"], {
    env: { ...process.env, TSRUST_BACKEND_PROOF_PID: String(normal.backendPid) }, encoding: 'utf8', windowsHide: true, timeout: 10000,
  });
  assert.ifError(loaded.error); assert.equal(loaded.status, 0, loaded.stderr);
  assert.equal(fs.realpathSync(loaded.stdout.trim()).toLowerCase(), fs.realpathSync(path.join(application, 'resources/backend/vcruntime140.dll')).toLowerCase(), 'native process loads its packaged CRT, not the developer machine runtime');
  assert.deepEqual(await page.evaluate(() => ({ process: typeof globalThis.process, require: typeof globalThis.require,
    namespaces: Object.keys(window.tsrust), commands: Object.keys(window.tsrust.tasks).sort(), rawIpc: 'ipcRenderer' in window.tsrust,
  })), { process: 'undefined', require: 'undefined', namespaces: ['tasks'], commands: ['create', 'delete', 'list', 'update'], rawIpc: false });
  assert.equal(await page.getByLabel('Access token', { exact: true }).count(), 0);
  const invalid = await page.evaluate(() => window.tsrust.tasks.create({ title: 1 })); assert.equal(invalid.error.code, 'api-validation');
  await page.getByLabel('New task', { exact: true }).fill('Packaged native task'); await page.getByRole('button', { name: 'Add task', exact: true }).click();
  await page.getByRole('button', { name: 'Edit Packaged native task', exact: true }).waitFor();
  await page.getByRole('button', { name: 'Edit Packaged native task', exact: true }).click();
  await page.getByLabel('Task title', { exact: true }).fill('Updated packaged task'); await page.getByRole('button', { name: 'Save changes', exact: true }).click();
  await page.getByRole('button', { name: 'Edit Updated packaged task', exact: true }).waitFor();
  await page.getByRole('checkbox', { name: 'Mark Updated packaged task complete', exact: true }).click();
  await page.getByRole('checkbox', { name: 'Mark Updated packaged task incomplete', exact: true }).waitFor();
  await page.screenshot({ path: path.join(root, 'target/task-desktop.png'), fullPage: true });
  await page.getByRole('button', { name: 'Delete Updated packaged task', exact: true }).click();
  await page.getByRole('button', { name: 'Confirm delete', exact: true }).click();
  await page.getByText('A clear start', { exact: true }).waitFor();
  await page.getByLabel('New task', { exact: true }).fill('Survives packaged restart'); await page.getByRole('button', { name: 'Add task', exact: true }).click();
  await page.getByRole('button', { name: 'Edit Survives packaged restart', exact: true }).waitFor();
  const originalUrl = page.url(); await page.evaluate(() => { location.href = 'https://example.invalid/'; });
  assert.equal(page.url(), originalUrl, 'renderer navigation is denied');
  assert.equal(await page.evaluate(() => window.open('https://example.invalid/')), null);
  assert.equal(normal.browser.contexts()[0].pages().length, 1);
  await close(normal);
  const restored = await launch('normal-data'); await restored.page.getByRole('button', { name: 'Edit Survives packaged restart', exact: true }).waitFor(); await close(restored);

  const crash = await launch('crash-data'); await crash.page.getByText('Connected to your desktop service', { exact: true }).waitFor();
  const failedPage = crash.page;
  process.kill(crash.backendPid);
  crash.page = await waitFor(() => crash.browser.contexts()[0].pages().find(candidate => candidate !== failedPage && !candidate.isClosed()), 'Backend crash did not create a safe failure window');
  await crash.page.getByRole('heading', { name: 'Service unavailable' }).waitFor(); await close(crash);

  const orphan = await launch('orphan-data'); await orphan.page.getByText('Connected to your desktop service', { exact: true }).waitFor();
  orphan.child.kill(); await deadline(orphan.exited, 5000, 'Parent crash was not observed');
  await waitFor(() => !alive(orphan.backendPid), 'Native backend survived hard parent death'); await orphan.browser.close().catch(() => {});

  fs.renameSync(backendExecutable, backendExecutable + '.missing');
  try {
    const missing = await launch('missing-data'); await missing.page.getByRole('heading', { name: 'Service unavailable' }).waitFor();
    assert.equal(missing.backendPid, undefined); await close(missing);
  } finally { fs.renameSync(backendExecutable + '.missing', backendExecutable); }
  console.log('Packaged Windows app: external copy with no Rust/Node on PATH, hidden real Electron/native SQLite CRUD, named-only bridge, renderer isolation, denied navigation/windows, startup failure, backend crash, graceful quit and hard-parent orphan cleanup passed.');
} finally {
  for (const record of sessions) {
    if (record.child.exitCode === null) { record.child.kill(); await deadline(record.exited, 5000, 'Desktop cleanup failed'); }
    await record.browser?.close().catch(() => {});
    if (record.backendPid && alive(record.backendPid)) { process.kill(record.backendPid); await waitFor(() => !alive(record.backendPid), 'Owned native backend cleanup failed'); }
  }
  assert.equal(path.dirname(fs.realpathSync(temporary)), fs.realpathSync(os.tmpdir()));
  assert.ok(path.basename(temporary).startsWith('tsrust-desktop-package-')); fs.rmSync(temporary, { recursive: true, force: true });
}
