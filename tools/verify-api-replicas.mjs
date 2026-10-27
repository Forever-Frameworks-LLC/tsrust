import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import http from 'node:http';
import { spawn, spawnSync } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';

// Exercise an actual pre-versioning executable, not two profiles handled by
// the new dispatcher. Never overwrite the retained legacy executable here.
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const example = path.join(root, 'examples/backend-tasks');
const suffix = process.platform === 'win32' ? '.exe' : '';
const compiler = process.env.TSRUST || path.join(root, 'target/debug/tsrust' + suffix);
const legacyExecutable = path.join(root, 'target/task-http-witness' + suffix);
const executable = path.join(root, 'target/task-versioned-witness' + suffix);
const environment = { ...process.env, CARGO_INCREMENTAL: '0', CARGO_BUILD_JOBS: '1', CARGO_TARGET_DIR: path.join(root, 'target'), TSRUST_CACHE_DIR: path.join(root, 'target/tsrust-native') };
const token = 'a'.repeat(64), headers = { authorization: `Bearer ${token}` };
const prefix = `generated-rollout-${process.pid}-`;
const temporary = fs.mkdtempSync(path.join(root, 'target/api-replicas-'));
const configs = [], sources = [], outputs = [], children = [], servers = [];
let browser;
function command(args) {
  const result = spawnSync(compiler, args, { cwd: root, env: environment, encoding: 'utf8', timeout: 900000, maxBuffer: 8 * 1024 * 1024, windowsHide: true });
  assert.ifError(result.error); assert.equal(result.status, 0, result.stdout + result.stderr);
}
function bounded(promise, milliseconds = 20000) {
  let timer;
  return Promise.race([promise, new Promise((_, reject) => { timer = setTimeout(() => reject(new Error('Replica acceptance timed out')), milliseconds); })]).finally(() => clearTimeout(timer));
}
function configure(name, configuration) {
  configuration.output = prefix + name;
  const filename = path.join(example, `.tsrust-rollout-${process.pid}-${name}.json`);
  fs.writeFileSync(filename, JSON.stringify(configuration), { flag: 'wx' }); configs.push(filename);
  const output = path.join(example, configuration.output); outputs.push(output);
  command(['api', 'generate', '--config', filename]);
  command(['api', 'generate', '--config', filename, '--check']);
  return output;
}
async function start(filename) {
  const child = spawn(filename, [], { cwd: temporary, env: { ...environment,
    TASK_TEST_DATABASE_URL: `sqlite:${path.join(temporary, 'tasks.sqlite').replaceAll('\\', '/')}?mode=rwc`,
    TASK_TEST_MIGRATIONS: path.join(example, 'migrations'), TASK_TEST_TOKEN: token,
    TASK_TEST_ORIGIN: 'http://127.0.0.1:5173', TASK_TEST_ADDRESS: '127.0.0.1:0',
  }, windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'] });
  const record = { child, errors: '', base: undefined };
  record.exited = new Promise((resolve, reject) => { child.once('error', reject); child.once('exit', resolve); });
  children.push(record); child.stderr.on('data', chunk => { record.errors += chunk; });
  let output = '';
  record.base = await bounded(new Promise((resolve, reject) => {
    child.once('error', reject); child.once('exit', code => reject(new Error(`Replica exited before readiness (${code}): ${record.errors}`)));
    child.stdout.on('data', chunk => {
      output += chunk;
      const line = output.split('\n').slice(0, -1).find(line => line.startsWith('{'));
      if (line) { try { const ready = JSON.parse(line); assert.match(ready.address, /^127\.0\.0\.1:\d+$/); resolve(`http://${ready.address}`); } catch (error) { reject(error); } }
    });
  }));
  return record.base;
}
async function listen(server, port = 0) {
  servers.push(server);
  await new Promise((resolve, reject) => { server.once('error', reject); server.listen(port, '127.0.0.1', resolve); });
  return `http://127.0.0.1:${server.address().port}`;
}
async function sdk(directory) {
  const client = await import(pathToFileURL(path.join(directory, 'index.js')));
  const transport = await import(pathToFileURL(path.join(directory, 'http.js')));
  return { ...client, at: url => client.createClient(transport.createHttpTransport({ url: url + '/api', headers })) };
}
try {
  assert.ok(fs.existsSync(legacyExecutable), 'Retain the accepted pre-versioning task-http-witness executable');
  command(['api', 'generate', '--config', path.join(example, 'tsrust.api.json')]);
  const original = JSON.parse(fs.readFileSync(path.join(example, 'tsrust.api.json')));
  const modern = structuredClone(original);
  modern.package.version = '0.2.0'; modern.apiVersion = 1;
  modern.compatible = ['generated-client/compatibility.json'];
  const operationFile = `.tsrust-rollout-${process.pid}-operations.tsr`;
  const operationPath = path.join(example, operationFile);
  fs.writeFileSync(operationPath, `import {TaskContext} from './context.tsr';
import {list,update,remove} from './operations.tsr';
import {Item,Failure,create as createTask} from './repository.tsr';
type RolloutInput={title:string;label?:string;};
type RolloutItem={kind:'task';id:int;title:string;completed:boolean;replica?:string;};
function enrich(result:ReadOnly<Item|Failure>):RolloutItem|Failure{
  if(result.kind==='task'){
    return {kind:'task',id:result.id,completed:result.completed,replica:'modern',title:clone(result.title)};
  }else{
    if(result.code==='invalid-input'){return {kind:'error',code:'invalid-input',message:clone(result.message)};}
    return {kind:'error',code:'not-found',message:clone(result.message)};
  }
}
export async function create(context:ReadOnly<TaskContext>,input:RolloutInput):Task<RolloutItem|Failure>{
  let title=input.title;
  if(input.label!==undefined){title=title+' '+input.label;}
  const result=await createTask(context.env.pool,{title});
  return enrich(result);
}
`, { flag: 'wx' }); sources.push(operationPath);
  command(['format', operationPath, '--write']); const formatted = fs.readFileSync(operationPath, 'utf8');
  command(['format', operationPath, '--write']); assert.equal(fs.readFileSync(operationPath, 'utf8'), formatted);
  modern.operations.find(operation => operation.name === 'tasks.create').module = operationFile;
  modern.entry = operationFile;
  modern.operations.push({ ...modern.operations[0], name: 'tasks.preview' });
  const modernDirectory = configure('modern', modern);
  command(['build', path.join(example, 'http-witness.tsr'), '-o', executable]);
  const subset = structuredClone(modern);
  delete subset.server; subset.package = { name: '@tsrust-example/rollout-subset', version: '1.0.0' };
  subset.compatible = [path.basename(modernDirectory) + '/compatibility.json'];
  subset.operations = subset.operations.filter(operation => ['tasks.list', 'tasks.create', 'tasks.preview'].includes(operation.name));
  const subsetDirectory = configure('subset', subset);
  const nextMajor = structuredClone(original); delete nextMajor.server;
  nextMajor.apiVersion = 2; nextMajor.package = { name: '@tsrust-example/rollout-v2', version: '2.0.0' };
  const majorDirectory = configure('major', nextMajor);
  const legacyDirectory = path.join(temporary, 'legacy'); fs.mkdirSync(legacyDirectory);
  fs.writeFileSync(path.join(legacyDirectory, 'package.json'), '{"type":"module"}');
  const manifest = fs.readFileSync(path.join(example, 'generated-client/manifest.json'), 'utf8');
  const current = await sdk(path.join(example, 'generated-client'));
  for (const [source, destination] of [['client.js', 'index.js'], ['wire.js', 'wire.js'], ['http.js', 'http.js']]) {
    const result = spawnSync('git', ['show', `03f20f0:crates/tsrust-compiler/src/api/${source}`], { cwd: root, encoding: 'utf8', windowsHide: true });
    assert.equal(result.status, 0, result.stderr);
    fs.writeFileSync(path.join(legacyDirectory, destination), result.stdout.replace('__TSRUST_API_MANIFEST__', JSON.stringify(manifest)).replace('__TSRUST_API_ID__', JSON.stringify(current.contractId)));
  }
  const oldBase = await start(legacyExecutable), newBase = await start(executable);
  let posts = 0, preflights = 0;
  const proxy = await listen(http.createServer((request, response) => {
    const base = request.method === 'OPTIONS' ? (preflights++, oldBase) : (posts++ % 2 === 0 ? oldBase : newBase);
    const upstream = http.request(base + request.url, { method: request.method, headers: { ...request.headers, host: new URL(base).host } }, incoming => {
      response.writeHead(incoming.statusCode, incoming.headers); incoming.pipe(response);
    });
    upstream.on('error', error => { response.destroy(error); }); request.pipe(upstream);
  }));
  const newer = await sdk(subsetDirectory), old = await sdk(legacyDirectory), major = await sdk(majorDirectory);
  assert.notEqual(newer.contractId, current.contractId, 'subset has a different whole-manifest fingerprint');
  assert.notEqual(major.wireContractId, current.wireContractId, 'API major affects identity even with identical DTOs');
  const unsupportedSubset = await fetch(oldBase + '/api', { method: 'POST', headers: { ...headers, 'content-type': 'application/json', 'x-tsrust-contract': newer.contractId }, body: JSON.stringify({ operation: 'tasks.list', input: { after: 0, limit: 100 } }) });
  assert.equal(unsupportedSubset.status, 409, 'legacy executable really rejects the new whole-manifest hash');
  assert.equal((await unsupportedSubset.json()).error.code, 'api-incompatible');
  const api = newer.at(proxy);
  for (let index = 0; index < 4; index++) {
    const result = await api.tasks.create({ title: `Replica ${index}` }); assert.equal(result.kind, 'task');
    assert.equal(result.replica, index % 2 === 0 ? undefined : 'modern', 'optional new results decode on either replica');
  }
  for (let index = 0; index < 4; index++) assert.equal((await api.tasks.list({ after: 0, limit: 100 })).items.length, 4);
  assert.equal(posts, 8, 'exactly one upstream write/read per invocation');
  await assert.rejects(api.tasks.preview({ after: 0, limit: 100 }), error => error instanceof newer.ApiCompatibilityError && error.code === 'api-operation');
  assert.equal((await api.tasks.preview({ after: 0, limit: 100 })).items.length, 4);
  assert.equal(posts, 10, 'unsupported new operation was never retried');
  for (const base of [oldBase, newBase]) {
    assert.equal((await old.at(base).tasks.list({ after: 0, limit: 100 })).items.length, 4);
    await assert.rejects(major.at(base).tasks.create({ title: 'Must not execute' }), error => error instanceof major.ApiCompatibilityError && error.code === 'api-incompatible');
  }
  const legacyWrite = await old.at(newBase).tasks.create({ title: 'Old SDK projection' });
  assert.deepEqual(Object.keys(legacyWrite).sort(), ['completed', 'id', 'kind', 'title'], 'strict old SDK receives no new optional result fields');
  assert.equal((await api.tasks.list({ after: 0, limit: 100 })).items.length, 5, 'rejected major-version writes have no effect');
  await assert.rejects(newer.at(oldBase).tasks.create({ title: 'New input', label: 'retained' }), error => error instanceof newer.ApiCompatibilityError && error.code === 'api-incompatible');
  assert.equal((await api.tasks.list({ after: 0, limit: 100 })).items.length, 5, 'new input is not silently removed to execute an old write');
  assert.equal((await newer.at(newBase).tasks.create({ title: 'New input', label: 'retained' })).title, 'New input retained');
  await assert.rejects(newer.createClient((await import(pathToFileURL(path.join(subsetDirectory, 'http.js')))).createHttpTransport({ url: newBase + '/api', headers: { authorization: 'Bearer wrong' } })).tasks.create({ title: 'Denied' }), error => error.code === 'api-denied');
  const browserBase = await listen(http.createServer((request, response) => {
    if (request.url === '/') { response.setHeader('content-type', 'text/html'); response.end('<!doctype html><title>Replica acceptance</title>'); return; }
    const filename = request.url.slice(1);
    if (!['index.js', 'wire.js', 'http.js', 'compatibility.js'].includes(filename)) { response.writeHead(404); response.end(); return; }
    response.setHeader('content-type', 'text/javascript'); response.end(fs.readFileSync(path.join(subsetDirectory, filename)));
  }), 5173);
  const { chromium } = await import(pathToFileURL(path.join(example, 'web/node_modules/@playwright/test/index.mjs')));
  browser = await chromium.launch({ headless: true }); const page = await browser.newPage();
  const errors = []; page.on('pageerror', error => errors.push(error.message)); await page.goto(browserBase);
  const before = posts;
  assert.equal(await page.evaluate(async ({ proxy, token }) => {
    const { createClient } = await import('/index.js'); const { createHttpTransport } = await import('/http.js');
    const api = createClient(createHttpTransport({ url: proxy + '/api', headers: { authorization: `Bearer ${token}` } }));
    for (let index = 0; index < 2; index++) await api.tasks.create({ title: `Browser replica ${index}` });
    return (await api.tasks.list({ after: 0, limit: 100 })).items.length;
  }, { proxy, token }), 8);
  assert.equal(posts - before, 3); assert.ok(preflights > 0, 'real browser passed the legacy CORS preflight'); assert.deepEqual(errors, []);
  console.log('Mixed replicas passed: immutable legacy SDK/server, subset clients, common reads/writes, typed new-operation failures, API-major isolation, no retry/effect on rejection, browser legacy CORS and orderly drain.');
} finally {
  await browser?.close();
  for (const server of servers) { server.closeAllConnections(); await new Promise(resolve => server.close(resolve)); }
  for (const record of children) {
    if (record.child.exitCode === null) { try { await fetch(record.base + '/__fixture_stop', { method: 'POST', headers, signal: AbortSignal.timeout(5000) }); } catch { record.child.kill(); } }
    try { assert.equal(await bounded(record.exited), 0, record.errors); } finally { if (record.child.exitCode === null) record.child.kill(); }
  }
  command(['api', 'generate', '--config', path.join(example, 'tsrust.api.json')]);
  for (const filename of configs) fs.unlinkSync(filename);
  for (const directory of outputs) {
    assert.equal(path.dirname(directory), example); assert.ok(path.basename(directory).startsWith(prefix)); fs.rmSync(directory, { recursive: true, force: true });
  }
  for (const filename of sources) fs.unlinkSync(filename);
  assert.equal(path.dirname(fs.realpathSync(temporary)), fs.realpathSync(path.join(root, 'target')));
  assert.ok(path.basename(temporary).startsWith('api-replicas-')); fs.rmSync(temporary, { recursive: true, force: true });
}
