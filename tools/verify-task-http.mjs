import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { spawn, spawnSync } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const example = path.join(root, 'examples/backend-tasks');
const compiler = process.env.TSRUST || path.join(root, 'target/debug/tsrust' + (process.platform === 'win32' ? '.exe' : ''));
const executable = path.join(root, 'target/task-http-witness' + (process.platform === 'win32' ? '.exe' : ''));
const environment = { ...process.env, CARGO_INCREMENTAL: '0', CARGO_BUILD_JOBS: '1', CARGO_TARGET_DIR: process.env.CARGO_TARGET_DIR || path.join(root, 'target'), TSRUST_CACHE_DIR: process.env.TSRUST_CACHE_DIR || path.join(root, 'target/tsrust-native') };
const token = 'a'.repeat(64);
const origin = 'http://127.0.0.1:5173';
const headers = { authorization: `Bearer ${token}`, origin };
function command(args) {
  const result = spawnSync(compiler, args, { cwd: root, env: environment, encoding: 'utf8', timeout: 600000, maxBuffer: 8 * 1024 * 1024, windowsHide: true });
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stdout + result.stderr);
}
function bounded(promise, milliseconds, message) {
  let timer;
  return Promise.race([promise, new Promise((_, reject) => { timer = setTimeout(() => reject(new Error(message)), milliseconds); })]).finally(() => clearTimeout(timer));
}
assert.ok(process.argv.length === 2 || process.argv.length === 3 && process.argv[2] === '--run-only', 'Use --run-only to reuse the built witness');
if (!process.argv.includes('--run-only')) {
  const config = path.join(example, 'tsrust.api.json');
  command(['api', 'generate', '--config', config]);
  command(['api', 'generate', '--config', config, '--check']);
  command(['check', path.join(example, 'main.tsr')]);
  command(['build', path.join(example, 'http-witness.tsr'), '-o', executable]);
}
const clientDirectory = process.env.TSRUST_TASK_CLIENT_DIR || path.join(example, 'generated-client');
const { createClient, contractId } = await import(pathToFileURL(path.join(clientDirectory, 'index.js')));
const { createHttpTransport } = await import(pathToFileURL(path.join(clientDirectory, 'http.js')));
let createReactClient, QueryClient, QueryObserver;
if (process.env.TSRUST_TASK_QUERY === '1') {
  ({ createReactClient } = await import(pathToFileURL(path.join(clientDirectory, 'react-query.js'))));
  ({ QueryClient, QueryObserver } = await import(pathToFileURL(path.join(root, 'tools/api-client-tests/node_modules/@tanstack/react-query/build/modern/index.js'))));
}
const directory = fs.mkdtempSync(path.join(root, 'target/task-http-'));
const databasePath = path.join(directory, 'task data.sqlite');
const childEnvironment = { ...environment, TASK_TEST_DATABASE_URL: `sqlite:${databasePath.replaceAll('\\', '/')}?mode=rwc`, TASK_TEST_MIGRATIONS: path.join(example, 'migrations'), TASK_TEST_TOKEN: token };

async function service(verify) {
  const child = spawn(executable, [], { cwd: directory, env: childEnvironment, windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'] });
  let output = '', errors = '', base, query;
  child.stdout.setEncoding('utf8');
  child.stderr.setEncoding('utf8');
  child.stderr.on('data', chunk => { errors += chunk; });
  const exited = new Promise(resolve => {
    child.once('error', error => resolve({ error }));
    child.once('exit', (code, signal) => resolve({ code, signal }));
  });
  const ready = new Promise((resolve, reject) => {
    child.once('error', reject);
    child.once('exit', code => reject(new Error(`Task HTTP exited before readiness (${code}): ${errors}`)));
    child.stdout.on('data', chunk => {
      output += chunk;
      const line = output.split('\n').slice(0, -1).find(line => line.startsWith('{') && line.includes('"ready"'));
      if (!line) return;
      try {
        const value = JSON.parse(line);
        assert.equal(value.kind, 'ready');
        assert.match(value.address, /^127\.0\.0\.1:\d+$/);
        resolve(`http://${value.address}`);
      } catch (error) { reject(error); }
    });
  });
  try {
    base = await bounded(ready, 15000, `Task HTTP readiness timed out: ${errors}`);
    const api = createClient(createHttpTransport({ url: base + '/api', headers }));
    if (createReactClient) query = {
      api: createReactClient(createHttpTransport({ url: base + '/api', headers }), { scope: `native-task/${base}/fixture-user` }),
      cache: new QueryClient({ defaultOptions: { queries: { gcTime: Infinity }, mutations: { gcTime: Infinity } } }),
    };
    await verify(api, base, query);
  } finally {
    query?.cache.clear();
    if (child.exitCode === null && base) {
      try {
        const stop = await fetch(base + '/__fixture_stop', { method: 'POST', headers, signal: AbortSignal.timeout(5000) });
        assert.equal(stop.status, 200);
      } catch { child.kill(); }
    } else if (child.exitCode === null) child.kill();
    let result;
    try { result = await bounded(exited, 10000, 'Task HTTP did not drain'); }
    catch (error) { child.kill(); await bounded(exited, 5000, 'Task HTTP did not exit after cleanup'); throw error; }
    assert.ifError(result.error);
    assert.equal(result.code, 0, `Task HTTP exit failed (${result.signal}): ${errors}`);
  }
}

try {
  let saved;
  await service(async (api, base, query) => {
    assert.deepEqual(await api.tasks.list({ after: 0, limit: 100 }), { kind: 'page', items: [] });
    const item = await api.tasks.create({ title: '  Persistent task  ' });
    assert.equal(item.kind, 'task');
    assert.equal(item.title, 'Persistent task');
    assert.equal(item.completed, false);
    const sqlTitle = "'; DROP TABLE tasks;-- <script>data</script>";
    const second = await api.tasks.create({ title: sqlTitle });
    assert.equal(second.kind, 'task');
    assert.equal(second.title, sqlTitle);
    assert.deepEqual(await api.tasks.list({ after: item.id, limit: 1 }), { kind: 'page', items: [second] });
    for (const title of ['   ', 'a'.repeat(201), '🚀'.repeat(51)]) {
      const failure = await api.tasks.create({ title });
      assert.equal(failure.kind, 'error');
      assert.equal(failure.code, 'invalid-input');
    }
    const unicode = await api.tasks.create({ title: '🚀'.repeat(50) });
    assert.equal(unicode.kind, 'task');
    assert.equal(unicode.title, '🚀'.repeat(50));
    saved = await api.tasks.update({ id: item.id, title: 'Verified native HTTP task', completed: true });
    assert.deepEqual(saved, { kind: 'task', id: item.id, title: 'Verified native HTTP task', completed: true });
    assert.deepEqual(await api.tasks.delete({ id: second.id }), { kind: 'deleted', id: second.id });
    assert.equal((await api.tasks.delete({ id: second.id })).code, 'not-found');
    assert.deepEqual(await api.tasks.delete({ id: unicode.id }), { kind: 'deleted', id: unicode.id });
    assert.equal((await api.tasks.list({ after: 0, limit: 0 })).code, 'invalid-input');

    const denied = createClient(createHttpTransport({ url: base + '/api', headers: { authorization: 'Bearer wrong' } }));
    await assert.rejects(denied.tasks.create({ title: 'Denied' }), error => error.code === 'api-denied');
    const raw = (extra, body) => fetch(base + '/api', { method: 'POST', headers: { ...headers, 'content-type': 'application/json', 'x-tsrust-contract': contractId, ...extra }, body, signal: AbortSignal.timeout(5000) });
    for (const [extra, body, status, code] of [
      [{ origin: 'http://untrusted.invalid' }, JSON.stringify({ operation: 'tasks.list', input: { after: 0, limit: 100 } }), 403, 'api-denied'],
      [{}, JSON.stringify({ operation: 'tasks.create', input: { title: 'Wrong shape', extra: true } }), 400, 'api-validation'],
      [{ 'x-tsrust-contract': 'stale' }, JSON.stringify({ operation: 'tasks.list', input: { after: 0, limit: 100 } }), 409, 'api-incompatible'],
    ]) {
      const response = await raw(extra, body);
      assert.equal(response.status, status);
      assert.equal((await response.json()).error.code, code);
      assert.equal(response.headers.get('cache-control'), 'no-store');
      assert.equal(response.headers.get('x-content-type-options'), 'nosniff');
    }
    const preflight = await fetch(base + '/api', { method: 'OPTIONS', headers: { origin, 'access-control-request-method': 'POST', 'access-control-request-headers': 'authorization,content-type,x-tsrust-contract' }, signal: AbortSignal.timeout(5000) });
    assert.equal(preflight.status, 204);
    assert.equal(preflight.headers.get('access-control-allow-origin'), origin);
    assert.match(preflight.headers.get('access-control-allow-headers'), /x-tsrust-contract/);
    const wrongOrigin = await fetch(base + '/api', { method: 'OPTIONS', headers: { origin: 'http://untrusted.invalid', 'access-control-request-method': 'POST' }, signal: AbortSignal.timeout(5000) });
    assert.equal(wrongOrigin.status, 403);
    assert.equal(wrongOrigin.headers.get('access-control-allow-origin'), null);
    if (query) {
      const options = query.api.tasks.list.queryOptions({ after: 0, limit: 100 }, { staleTime: Infinity });
      assert.deepEqual(await query.cache.fetchQuery(options), { kind: 'page', items: [saved] });
      const observer = new QueryObserver(query.cache, options);
      const unsubscribe = observer.subscribe(() => {});
      const mutate = (operation, input) => query.cache.getMutationCache().build(query.cache, operation.mutationOptions()).execute(input);
      try {
        const created = await mutate(query.api.tasks.create, { title: 'Native query integration' });
        assert.deepEqual(query.cache.getQueryData(options.queryKey), { kind: 'page', items: [saved, created] });
        const updated = await mutate(query.api.tasks.update, { id: created.id, title: 'Query cache refreshed', completed: true });
        assert.deepEqual(query.cache.getQueryData(options.queryKey), { kind: 'page', items: [saved, updated] });
        await mutate(query.api.tasks.delete, { id: created.id });
        assert.deepEqual(query.cache.getQueryData(options.queryKey), { kind: 'page', items: [saved] });
      } finally { unsubscribe(); }
    }
  });
  assert.ok(fs.statSync(databasePath).size > 0);
  await service(async api => {
    assert.deepEqual(await api.tasks.list({ after: 0, limit: 100 }), { kind: 'page', items: [saved] });
    assert.deepEqual(await api.tasks.delete({ id: saved.id }), { kind: 'deleted', id: saved.id });
    assert.deepEqual(await api.tasks.list({ after: 0, limit: 100 }), { kind: 'page', items: [] });
  });
  console.log(`Task HTTP: generated named CRUD, validation, access policy/CORS, parameterized SQL, orderly drain and process-restart SQLite persistence${createReactClient ? ', plus real TanStack query/mutation invalidation' : ''} passed.`);
} finally {
  assert.equal(path.dirname(fs.realpathSync(directory)), fs.realpathSync(path.join(root, 'target')));
  assert.ok(path.basename(directory).startsWith('task-http-'));
  fs.rmSync(directory, { recursive: true, force: true });
}
