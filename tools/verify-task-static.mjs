import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import http from 'node:http';
import net from 'node:net';
import { spawn, spawnSync } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const example = path.join(root, 'examples/backend-tasks');
const compiler = process.env.TSRUST || path.join(root, 'target/debug/tsrust' + (process.platform === 'win32' ? '.exe' : ''));
const executable = path.join(root, 'target/task-static-witness' + (process.platform === 'win32' ? '.exe' : ''));
assert.ok(process.argv.length === 2 || process.argv.length === 3 && process.argv[2] === '--run-only');
function command(args) {
  const result = spawnSync(compiler, args, { cwd: root, env: { ...process.env, CARGO_INCREMENTAL: '0', CARGO_BUILD_JOBS: '1', CARGO_TARGET_DIR: path.join(root, 'target'), TSRUST_CACHE_DIR: path.join(root, 'target/tsrust-native') }, encoding: 'utf8', timeout: 900000, maxBuffer: 8 * 1024 * 1024, windowsHide: true });
  assert.ifError(result.error); assert.equal(result.status, 0, result.stdout + result.stderr);
}
function bounded(promise, milliseconds = 15000) {
  let timer;
  return Promise.race([promise, new Promise((_, reject) => { timer = setTimeout(() => reject(new Error('Static acceptance timed out')), milliseconds); })]).finally(() => clearTimeout(timer));
}
if (!process.argv.includes('--run-only')) {
  command(['api', 'generate', '--config', path.join(example, 'tsrust.api.json')]);
  command(['check', path.join(example, 'main.tsr')]);
  command(['build', path.join(example, 'http-witness.tsr'), '-o', executable]);
}
const directory = fs.mkdtempSync(path.join(root, 'target/task-static-'));
const web = path.join(directory, 'web'), admin = path.join(directory, 'admin');
fs.mkdirSync(web); fs.mkdirSync(admin);
fs.writeFileSync(path.join(web, 'index.html'), '<!doctype html><title>Web mount</title><h1>Web only</h1>');
fs.writeFileSync(path.join(admin, 'index.html'), '<!doctype html><title>Admin mount</title><h1>Admin only</h1>');
fs.writeFileSync(path.join(web, 'main.js'), 'export const app = "web";');
fs.writeFileSync(path.join(admin, 'main.js'), 'export const app = "admin";');
fs.writeFileSync(path.join(web, 'bytes.bin'), Buffer.from([0, 255, 1, 128, 13, 10]));
fs.writeFileSync(path.join(web, 'large.bin'), Buffer.alloc(8 * 1024 * 1024, 42));
fs.writeFileSync(path.join(web, '.env'), 'private');
fs.writeFileSync(path.join(directory, 'outside.txt'), 'outside root');
const outside = path.join(directory, 'outside'); fs.mkdirSync(outside);
fs.writeFileSync(path.join(outside, 'private.txt'), 'outside root');
fs.symlinkSync(outside, path.join(web, 'escape'), process.platform === 'win32' ? 'junction' : 'dir');
const reservation = net.createServer();
await new Promise(resolve => reservation.listen(0, '127.0.0.1', resolve));
const port = reservation.address().port;
await new Promise(resolve => reservation.close(resolve));
const base = `http://127.0.0.1:${port}`;
const token = 'a'.repeat(64), headers = { authorization: `Bearer ${token}` };
const child = spawn(executable, [], { cwd: directory, windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'], env: { ...process.env,
  TASK_TEST_DATABASE_URL: `sqlite:${path.join(directory, 'tasks.sqlite').replaceAll('\\', '/')}?mode=rwc`, TASK_TEST_MIGRATIONS: path.join(example, 'migrations'), TASK_TEST_TOKEN: token,
  TASK_TEST_ADDRESS: `127.0.0.1:${port}`, TASK_TEST_ORIGIN: base, TASK_WEB_ROOT: web, TASK_ADMIN_ROOT: admin,
} });
let output = '', errors = '', browser;
child.stderr.on('data', chunk => { errors += chunk; });
const exited = new Promise((resolve, reject) => { child.once('exit', resolve); child.once('error', reject); });
const ready = new Promise((resolve, reject) => {
  child.once('error', reject); child.once('exit', code => reject(new Error(`Static fixture exited ${code}: ${errors}`)));
  child.stdout.on('data', chunk => { output += chunk; const line = output.split('\n').slice(0, -1).find(line => line.startsWith('{')); if (line) { try { assert.equal(JSON.parse(line).address, `127.0.0.1:${port}`); resolve(); } catch (error) { reject(error); } } });
});
async function request(url, options = {}) { return fetch(base + url, { ...options, signal: AbortSignal.timeout(5000) }); }
async function raw(url) {
  return bounded(new Promise((resolve, reject) => { const request = http.get(base + '/', { path: url }, response => { response.resume(); response.once('end', () => resolve(response.statusCode)); }); request.once('error', reject); }));
}
try {
  await bounded(ready);
  for (const prefix of ['web', 'admin']) {
    const redirect = await request('/' + prefix, { redirect: 'manual' }); assert.equal(redirect.status, 307); assert.equal(redirect.headers.get('location'), '/' + prefix + '/');
    for (const suffix of ['/', '/projects/42']) {
      const response = await request('/' + prefix + suffix, { headers: { accept: 'text/html' } }); assert.equal(response.status, 200); assert.match(await response.text(), prefix === 'web' ? /Web only/ : /Admin only/);
    }
    for (const suffix of ['/missing.js', '/projects', '/.env']) assert.equal((await request('/' + prefix + suffix)).status, suffix === '/.env' ? 400 : 404);
    assert.equal((await request('/' + prefix + '/projects', { headers: { accept: 'text/html;q=0' } })).status, 404);
    assert.equal((await request('/' + prefix + '/missing.js', { headers: { accept: 'text/html' } })).status, 404);
    assert.equal((await request('/' + prefix + '/', { method: 'POST', headers: { accept: 'text/html' } })).status, 404);
  }
  const script = await request('/web/main.js'); assert.equal(script.status, 200); assert.match(script.headers.get('content-type'), /javascript/); assert.equal(script.headers.get('x-content-type-options'), 'nosniff'); const modified = script.headers.get('last-modified'); await script.text();
  assert.equal((await request('/web/main.js', { headers: { 'if-modified-since': modified } })).status, 304);
  const bytes = await request('/web/bytes.bin'); assert.deepEqual(Buffer.from(await bytes.arrayBuffer()), Buffer.from([0, 255, 1, 128, 13, 10]));
  const range = await request('/web/bytes.bin', { headers: { range: 'bytes=1-3' } }); assert.equal(range.status, 206); assert.deepEqual(Buffer.from(await range.arrayBuffer()), Buffer.from([255, 1, 128]));
  const head = await request('/web/bytes.bin', { method: 'HEAD' }); assert.equal(head.headers.get('content-length'), '6'); assert.equal((await head.arrayBuffer()).byteLength, 0);
  const large = await request('/web/large.bin'); await large.body.cancel();
  assert.equal((await request('/web/escape/private.txt')).status, 404);
  for (const value of ['%2e%2e/outside.txt', '%2foutside.txt', '%5coutside.txt', '%252e%252e/outside.txt', 'a%00b', 'file%3Asecret', 'CON']) assert.equal(await raw('/web/' + value), 400);
  for (const url of ['/webish/', '/unknown', '/api/unknown']) assert.equal((await request(url, { headers: { accept: 'text/html' } })).status, 404);
  assert.equal((await request('/api', { headers: { accept: 'text/html' } })).status, 404);
  const { createClient } = await import(pathToFileURL(path.join(example, 'generated-client/index.js')));
  const { createHttpTransport } = await import(pathToFileURL(path.join(example, 'generated-client/http.js')));
  const api = createClient(createHttpTransport({ url: base + '/api', headers }));
  assert.equal((await api.tasks.create({ title: 'Static and API coexist' })).kind, 'task');
  const tasks = await api.tasks.list({ after: 0, limit: 100 }); assert.equal(tasks.kind, 'page'); assert.equal(tasks.items[0].title, 'Static and API coexist');
  // Replace one mount's deployment assets with the real already-built React app.
  const dist = path.join(example, 'web/dist'); assert.ok(fs.existsSync(path.join(dist, 'index.html')), 'Build the web app with TSRUST_WEB_BASE=/web/ first');
  fs.cpSync(dist, web, { recursive: true });
  const { chromium } = await import(pathToFileURL(path.join(example, 'web/node_modules/@playwright/test/index.mjs')));
  browser = await chromium.launch({ headless: true });
  const page = await browser.newPage(); const pageErrors = []; page.on('pageerror', error => pageErrors.push(error.message));
  await page.goto(base + '/web/');
  assert.equal(await page.getByLabel('API address', { exact: true }).inputValue(), base + '/api');
  await page.getByLabel('Access token', { exact: true }).fill(token); await page.getByRole('button', { name: 'Connect', exact: true }).click();
  await page.getByRole('button', { name: 'Edit Static and API coexist', exact: true }).waitFor();
  await page.getByLabel('New task', { exact: true }).fill('Served by native'); await page.getByLabel('New task', { exact: true }).press('Enter');
  await page.getByRole('button', { name: 'Edit Served by native', exact: true }).waitFor();
  assert.deepEqual(pageErrors, []); await page.screenshot({ path: path.join(root, 'target/task-native-web.png'), fullPage: true });
  console.log('Static mounts: isolated SPA fallback/API, traversal/symlink rejection, MIME/binary/range/HEAD/304, stream cancellation, actual native-served React CRUD and orderly drain passed.');
} finally {
  await browser?.close();
  if (child.exitCode === null) { try { await request('/__fixture_stop', { method: 'POST', headers }); } catch { child.kill(); } }
  try { assert.equal(await bounded(exited), 0, errors); } finally {
    if (child.exitCode === null) { child.kill(); await bounded(exited); }
    assert.equal(path.dirname(fs.realpathSync(directory)), fs.realpathSync(path.join(root, 'target'))); assert.ok(path.basename(directory).startsWith('task-static-'));
    fs.unlinkSync(path.join(web, 'escape')); fs.rmSync(directory, { recursive: true, force: true });
  }
}
