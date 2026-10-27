import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { spawn, spawnSync } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const compiler = process.env.TSRUST || path.join(root, 'target/debug/tsrust' + (process.platform === 'win32' ? '.exe' : ''));
const executable = path.join(root, 'target/native-api-versioned-witness' + (process.platform === 'win32' ? '.exe' : ''));
const example = path.join(root, 'examples/api-contract');
const config = path.join(example, 'tsrust.server.api.json');
const environment = { ...process.env, CARGO_INCREMENTAL: '0', CARGO_BUILD_JOBS: '1', CARGO_TARGET_DIR: process.env.CARGO_TARGET_DIR || path.join(root, 'target'), TSRUST_CACHE_DIR: process.env.TSRUST_CACHE_DIR || path.join(root, 'target/tsrust-native') };
function command(args, expected = 0) {
  const result = spawnSync(compiler, args, { cwd: root, env: environment, encoding: 'utf8', timeout: 600000, maxBuffer: 8 * 1024 * 1024, windowsHide: true });
  assert.ifError(result.error);
  assert.equal(result.status, expected, result.stdout + result.stderr);
  return result.stdout + result.stderr;
}
assert.ok(process.argv.length === 2 || process.argv.length === 3 && process.argv[2] === '--run-only', 'Use --run-only to reuse the native executable');
if (!process.argv.includes('--run-only')) {
  command(['api', 'generate', '--config', path.join(example, 'tsrust.api.json')]);
  command(['api', 'generate', '--config', config]);
  command(['api', 'generate', '--config', config, '--check']);
  const generated = path.join(example, 'generated-server/index.tsr');
  const marker = path.join(example, 'generated-server-client/.tsrust-api.json');
  const source = fs.readFileSync(generated, 'utf8');
  const markerStamp = fs.statSync(marker).mtimeMs;
  try {
    fs.writeFileSync(generated, source + '\n// owner edit\n');
    assert.match(command(['api', 'generate', '--config', config], 1), /user edits/);
    assert.equal(fs.readFileSync(generated, 'utf8'), source + '\n// owner edit\n');
    assert.equal(fs.statSync(marker).mtimeMs, markerStamp, 'both outputs preflight before either marker is replaced');
  } finally { fs.writeFileSync(generated, source); }
  const clientSource = fs.readFileSync(path.join(example, 'generated-server-client/index.js'), 'utf8');
  assert.doesNotMatch(clientSource, /server-policy\.tsr|server-operations\.tsr|FixtureContext|api-fixture-only/);
  command(['build', 'examples/api-contract/src/server-witness.tsr', '-o', executable]);
}

const current = await import(pathToFileURL(path.join(example, 'generated-server-client/index.js')));
const currentHttp = await import(pathToFileURL(path.join(example, 'generated-server-client/http.js')));
const previous = await import(pathToFileURL(path.join(example, 'generated/index.js')));
const previousHttp = await import(pathToFileURL(path.join(example, 'generated/http.js')));
const child = spawn(executable, [], { cwd: root, env: environment, windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'] });
let output = '', errors = '', base;
const exited = new Promise((resolve, reject) => { child.once('error', reject); child.once('exit', (code, signal) => resolve({ code, signal })); });
child.stderr.on('data', chunk => { errors += chunk; });
const ready = new Promise((resolve, reject) => {
  child.once('error', reject);
  child.once('exit', code => reject(new Error(`Native fixture exited before readiness (${code}): ${errors}`)));
  child.stdout.on('data', chunk => {
    output += chunk;
    const line = output.split('\n').slice(0, -1).find(line => line.startsWith('{') && line.includes('"ready"'));
    if (line) {
      try {
        const ready = JSON.parse(line);
        assert.match(ready.address, /^127\.0\.0\.1:\d+$/);
        resolve(`http://${ready.address}`);
      } catch (error) { reject(error); }
    }
  });
});
async function bounded(promise, ms, message) {
  let timer;
  try { return await Promise.race([promise, new Promise((_, reject) => { timer = setTimeout(() => reject(new Error(message)), ms); })]); }
  finally { clearTimeout(timer); }
}
const headers = { authorization: 'Bearer api-fixture-only' };
const task = { kind: 'task', id: 7, title: 'Ship prototype', completed: false };
try {
  base = await bounded(ready, 20000, 'Native API readiness timed out');
  const api = current.createClient(currentHttp.createHttpTransport({ url: base + '/api', headers }));
  const old = previous.createClient(previousHttp.createHttpTransport({ url: base + '/api', headers }));
  assert.deepEqual(await api.tasks.list({ search: null }), [{ ...task, revision: 1 }]);
  assert.deepEqual(await old.tasks.list({ search: null }), [task], 'strict old client receives projected old DTOs');
  assert.equal(await api.context.count(2), 12, 'borrowed private Context supplies native state');
  assert.equal(await old.math.count(4), 5);
  assert.equal(await api.meta.marker(), '__TSRUST_API_ID__');
  assert.deepEqual(await api.tasks.create({ title: 'Native request' }), { ...task, id: 8, title: 'Native request' });
  const prototypeData = JSON.parse('{"__proto__":"native data"}');
  const echoedKeys = await api.data.roundTrip(prototypeData);
  assert.equal(Object.hasOwn(echoedKeys, '__proto__'), true);
  assert.equal(echoedKeys.__proto__, 'native data');
  for (const ratio of [1e20, -1e20, Number.MAX_VALUE, 0.125, Number.MAX_SAFE_INTEGER + 1]) {
    const rich = { pair: [2, '100000000000000000000'], ratio, enabled: true, variant: { kind: 'b', text: 'Native union' } };
    assert.deepEqual(await api.data.echo(rich), rich);
  }
  assert.deepEqual(await api.data.rows([task]), [task]);
  assert.deepEqual(await api.data.rows([]), []);
  for (const item of [undefined, task]) for (const enabled of [undefined, false, true]) for (const maybe of [null, task]) for (const variant of [{ kind: 'a', value: 7 }, { kind: 'b', text: 'Nested union' }]) {
    const nested = { rows: [task], pair: [task, 3], item, enabled, maybe, variant };
    const expected = { ...nested };
    if (item === undefined) delete expected.item;
    if (enabled === undefined) delete expected.enabled;
    assert.deepEqual(await api.data.nested(nested), expected);
  }
  const request = async (body, extra = {}) => fetch(base + '/api', { method: 'POST', headers: { ...headers, 'content-type': 'application/json', 'x-tsrust-contract': current.wireContractId, ...extra }, body, signal: AbortSignal.timeout(5000) });
  for (const [body, extra, status, code] of [
    ['{', {}, 400, 'api-validation'],
    ['{}', {}, 400, 'api-validation'],
    [JSON.stringify({ operation: 'serverOnly', input: null }), {}, 404, 'api-operation'],
    [JSON.stringify({ operation: 'math.count', input: 'bad' }), {}, 400, 'api-validation'],
    [JSON.stringify({ operation: 'math.count', input: 1, extra: true }), {}, 400, 'api-validation'],
    [JSON.stringify({ operation: 'context.count', input: 2, context: { increment: 999 } }), {}, 400, 'api-validation'],
    [JSON.stringify({ operation: 'data.echo', input: { pair: ['two', 2], ratio: 1, enabled: true, variant: { kind: 'a', value: 3 } } }), {}, 400, 'api-validation'],
    [JSON.stringify({ operation: 'math.count', input: 9007199254740992 }), {}, 400, 'api-validation'],
    [JSON.stringify({ operation: 'math.count', input: 1 }), { 'x-tsrust-contract': 'unknown' }, 409, 'api-incompatible'],
    [JSON.stringify({ operation: 'context.count', input: 1 }), { 'x-tsrust-contract': previous.contractId }, 409, 'api-incompatible'],
    ['{', { authorization: 'wrong' }, 403, 'api-denied'],
    ['{}', { 'content-type': 'text/plain' }, 415, 'api-media-type'],
  ]) {
    const response = await request(body, extra);
    assert.equal(response.status, status);
    assert.equal((await response.json()).error.code, code);
  }
  await assert.rejects(api.fixture.unsafeInteger(), error => error.code === 'api-result');
  await assert.rejects(api.fixture.privateFailure(), error => error.code === 'api-internal' && !error.message.includes('SQL'));
  console.log('Native API: checked dispatch, private Context, auth, strict DTOs, tuples/unions, safe numbers, old-client projection and private errors passed.');
} finally {
  if (child.exitCode === null && base) {
    try { await fetch(base + '/__fixture_stop', { method: 'POST', headers, signal: AbortSignal.timeout(5000) }); }
    catch { child.kill(); }
  } else if (child.exitCode === null) child.kill();
  let result;
  try { result = await bounded(exited, 10000, 'Native API fixture failed to drain'); }
  catch (error) { child.kill(); throw error; }
  assert.equal(result.code, 0, `Native API fixture failed: ${errors}`);
}
