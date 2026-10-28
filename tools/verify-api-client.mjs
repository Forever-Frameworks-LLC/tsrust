import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { verifyHttp } from './verify-api-http.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const compiler = process.argv[2] || process.env.TSRUST || path.join(root, 'target/debug/tsrust' + (process.platform === 'win32' ? '.exe' : ''));
const config = path.join(root, 'examples/api-contract/tsrust.api.json');
fs.mkdirSync(path.join(root, 'target'), { recursive: true });
const directory = fs.mkdtempSync(path.join(root, 'target/api-client-'));
const output = path.join(directory, 'client');
function command(file, args, expected = 0) {
  const result = spawnSync(file, args, { cwd: directory, encoding: 'utf8', timeout: 60000, windowsHide: true });
  assert.ifError(result.error);
  assert.equal(result.status, expected, result.stdout + result.stderr);
  return result.stdout + result.stderr;
}

try {
  command(compiler, ['api', 'generate', '--config', config, '--out', output]);
  command(compiler, ['api', 'generate', '--config', config, '--out', output, '--check']);
  const original = fs.readFileSync(path.join(output, 'index.js'), 'utf8');
  command(compiler, ['api', 'generate', '--config', config, '--out', output]);
  assert.equal(fs.readFileSync(path.join(output, 'index.js'), 'utf8'), original);
  const { createClient, ApiError, manifest, contractId } = await import(pathToFileURL(path.join(output, 'index.js')));
  assert.match(contractId, /^[a-f0-9]{64}$/);
  assert.equal(Object.isFrozen(manifest.operations), true);
  assert.equal(manifest.operations.length, 6);
  const seen = [];
  const expectedTask = { kind: 'task', id: 7, title: 'Ship prototype', completed: false };
  const api = createClient({ async request(name, args, options) {
    seen.push({ name, args, options });
    if (name === 'tasks.list') return { ok: true, value: [expectedTask] };
    if (name === 'tasks.create') return { ok: true, value: { kind: 'task', id: 8, title: args.title, completed: false } };
    if (name === 'meta.marker') return { ok: true, value: '__TSRUST_API_ID__' };
    if (name === 'math.count') return { ok: true, value: args + 1 };
    if (name === 'data.roundTrip') return { ok: true, value: args };
    if (name === 'data.echo') return { ok: true, value: args };
    assert.fail('unlisted transport operation');
  } });
  assert.equal(api.serverOnly, undefined);
  assert.equal(Object.getPrototypeOf(api), null);
  const input = { search: null, completed: undefined };
  assert.deepEqual(await api.tasks.list(input), [expectedTask]);
  assert.notEqual(seen[0].args, input);
  assert.deepEqual(seen[0].args, { search: null });
  assert.equal(Object.hasOwn(input, 'completed'), true);
  assert.deepEqual(await api.tasks.create({ title: 'User-owned app' }), { kind: 'task', id: 8, title: 'User-owned app', completed: false });
  assert.equal((await api.tasks.create({ title: 'Unicode \u{1f680}' })).title, 'Unicode \u{1f680}');
  assert.equal(await api.meta.marker(), '__TSRUST_API_ID__');
  assert.equal(seen.at(-1).args, null);
  const controller = new AbortController();
  assert.equal(await api.math.count(4, { signal: controller.signal }), 5);
  assert.equal(seen.at(-1).options.signal, controller.signal);
  const prototypeData = JSON.parse('{"__proto__":"plain data"}');
  const roundTrip = await api.data.roundTrip(prototypeData);
  assert.equal(Object.hasOwn(roundTrip, '__proto__'), true);
  assert.equal(roundTrip.__proto__, 'plain data');
  assert.equal(Object.getPrototypeOf(roundTrip), Object.prototype);
  const rich = { pair: [2, 'two'], ratio: 1.25, enabled: true, variant: { kind: 'a', value: 3 } };
  assert.deepEqual(await api.data.echo(rich), rich);
  assert.deepEqual(await api.data.echo({ ...rich, variant: { kind: 'b', text: 'other' } }), { ...rich, variant: { kind: 'b', text: 'other' } });
  await assert.rejects(api.data.echo({ ...rich, ratio: Infinity }), /finite number/);
  await assert.rejects(api.data.echo({ ...rich, pair: [2] }), /tuple of declared length/);
  await assert.rejects(api.data.echo({ ...rich, variant: { kind: 'a', text: 'wrong fields' } }), /union member/);
  const count = seen.length;
  for (const invalid of [{}, { search: 1 }, { search: null, completed: null }, { search: null, extra: true }, JSON.parse('{"search":null,"__proto__":{}}')]) {
    await assert.rejects(api.tasks.list(invalid), error => error instanceof ApiError && error.code === 'api-validation');
  }
  let accessed = false;
  const accessor = Object.defineProperty({}, 'search', { enumerable: true, get() { accessed = true; return null; } });
  await assert.rejects(api.tasks.list(accessor), /declared data fields/);
  assert.equal(accessed, false);
  for (const invalid of [NaN, Infinity, 1.5, Number.MAX_SAFE_INTEGER + 1, '4']) await assert.rejects(api.math.count(invalid), /safe integer/);
  assert.equal(seen.length, count, 'invalid inputs never reach transport');
  const response = value => createClient({ async request() { return value; } });
  await assert.rejects(response({ ok: true, value: 'wrong literal' }).meta.marker(), /declared literal/);
  await assert.rejects(response({ ok: true, value: [{ ...expectedTask, id: '7' }] }).tasks.list({ search: null }), /safe integer/);
  await assert.rejects(response({ ok: true, value: [{ ...expectedTask, secret: 'hidden' }] }).tasks.list({ search: null }), /declared data fields/);
  await assert.rejects(response({ ok: true, value: new Array(1) }).tasks.list({ search: null }), /dense array/);
  await assert.rejects(response({ ok: true, value: Number.MAX_SAFE_INTEGER + 1 }).math.count(1), /safe integer/);
  await assert.rejects(response({ ok: false, error: { code: 'denied', message: 'Denied' } }).math.count(1), error => error instanceof ApiError && error.code === 'denied');
  await assert.rejects(response({ ok: 1, value: 2 }).math.count(1), error => error.code === 'api-protocol');
  await assert.rejects(response({ ok: true }).math.count(1), error => error.code === 'api-protocol');

  const tsc = ['editors/vscode/node_modules/typescript/bin/tsc', 'benchmarks/runtime/node_modules/typescript/bin/tsc']
    .map(relative => path.join(root, relative)).find(file => fs.existsSync(file));
  assert.ok(tsc, 'Install the repository TypeScript development dependencies before API acceptance');
  const consumer = path.join(directory, 'consumer.mts');
  fs.writeFileSync(consumer, `import {createClient} from './client/index.js';
const api = createClient({async request() {return {ok:true,value:[]};}});
const tasks: Promise<{kind:'task';id:number;title:string;completed:boolean}[]> = api.tasks.list({search:null});
const count: Promise<number> = api.math.count(1);
const marker: Promise<'__TSRUST_API_ID__'> = api.meta.marker();
api.data.echo({pair:[2,'two'],ratio:1.25,enabled:true,variant:{kind:'a',value:3}}).then(value=>{
  if(value.variant.kind==='a'){const narrowed:number=value.variant.value;void narrowed;}
});
// @ts-expect-error required search field
api.tasks.list({});
// @ts-expect-error optional boolean does not accept null
api.tasks.list({search:null,completed:null});
// @ts-expect-error unlisted backend function
api.serverOnly();
// @ts-expect-error wrong scalar argument
api.math.count('1');
// @ts-expect-error discriminated union uses value for kind a
api.data.echo({pair:[2,'two'],ratio:1,enabled:true,variant:{kind:'a',text:'wrong'}});
void tasks; void count; void marker;
`);
  command(process.execPath, [tsc, consumer, '--strict', '--noEmit', '--module', 'NodeNext', '--target', 'ES2022', '--lib', 'ES2022']);
  const browser = path.join(directory, 'browser.mts');
  fs.writeFileSync(browser, `import {createClient} from './client/index.js';
import {createHttpTransport} from './client/http.js';
const remote=createClient(createHttpTransport({url:'http://localhost/api',fetch:globalThis.fetch,headers:async()=>({authorization:'Bearer token'})}));
remote.math.count(1,{signal:new AbortController().signal});
const api=createClient({async request(){return {ok:true,value:1};}});
api.math.count(1,{signal:new AbortController().signal});
`);
  command(process.execPath, [tsc, browser, '--strict', '--noEmit', '--module', 'NodeNext', '--target', 'ES2022', '--lib', 'ES2022,DOM']);
  await verifyHttp(output);

  fs.writeFileSync(path.join(output, 'index.js'), original + '\n// owner edit\n');
  assert.match(command(compiler, ['api', 'generate', '--config', config, '--out', output], 1), /user edits/);
  assert.equal(fs.readFileSync(path.join(output, 'index.js'), 'utf8'), original + '\n// owner edit\n');
  fs.writeFileSync(path.join(output, 'index.js'), original);

  const backend = path.join(directory, 'backend');
  fs.mkdirSync(backend);
  const backendSource = path.join(backend, 'main.tsr');
  const changedConfig = path.join(backend, 'tsrust.api.json');
  const changedOutput = path.join(directory, 'changing-client');
  const writeBackend = source => {
    fs.writeFileSync(backendSource, source);
    command(compiler, ['format', backendSource, '--write']);
    const formatted = fs.readFileSync(backendSource, 'utf8');
    assert.equal(command(compiler, ['format', backendSource]), formatted, 'temporary fixtures format idempotently');
  };
  writeBackend('export function value():int{return 1;} function main(){}');
  fs.writeFileSync(changedConfig, JSON.stringify({ schemaVersion: 1, entry: 'main.tsr', output: 'client', package: { name: 'changing-api', version: '0.1.0' }, operations: [{ name: 'value', module: 'main.tsr', export: 'value', kind: 'query' }] }));
  const generateChanged = ['api', 'generate', '--config', changedConfig, '--out', changedOutput];
  command(compiler, generateChanged);
  const oldDeclaration = fs.readFileSync(path.join(changedOutput, 'index.d.ts'), 'utf8');
  const oldHistory = fs.readFileSync(path.join(changedOutput, 'compatibility.json'), 'utf8');
  const oldMarker = fs.readFileSync(path.join(changedOutput, '.tsrust-api.json'), 'utf8');
  writeBackend('export function value():string{return "changed";} function main(){}');
  assert.match(command(compiler, [...generateChanged, '--check'], 1), /stale/);
  assert.equal(fs.readFileSync(path.join(changedOutput, 'index.d.ts'), 'utf8'), oldDeclaration);
  assert.match(command(compiler, generateChanged, 1), /SDK version change/);
  assert.equal(fs.readFileSync(path.join(changedOutput, 'compatibility.json'), 'utf8'), oldHistory);
  assert.equal(fs.readFileSync(path.join(changedOutput, '.tsrust-api.json'), 'utf8'), oldMarker, 'rejected SDK replacement changes no ownership metadata');
  const changedOptions = JSON.parse(fs.readFileSync(changedConfig));
  changedOptions.package.version = '0.2.0';
  fs.writeFileSync(changedConfig, JSON.stringify(changedOptions));
  assert.match(command(compiler, generateChanged, 1), /breaks retained clients/);
  assert.equal(fs.readFileSync(path.join(changedOutput, 'compatibility.json'), 'utf8'), oldHistory, 'a version bump alone cannot bypass the API compatibility gate');
  changedOptions.apiVersion = 2;
  changedOptions.package.version = '1.0.0';
  fs.writeFileSync(changedConfig, JSON.stringify(changedOptions));
  command(compiler, generateChanged);
  assert.match(fs.readFileSync(path.join(changedOutput, 'index.d.ts'), 'utf8'), /Promise<string>/);
  const currentDeclaration = fs.readFileSync(path.join(changedOutput, 'index.d.ts'), 'utf8');
  writeBackend('export function value():string{return 1;} function main(){}');
  command(compiler, generateChanged, 1);
  assert.equal(fs.readFileSync(path.join(changedOutput, 'index.d.ts'), 'utf8'), currentDeclaration, 'invalid backend edits preserve the last generated package');
  for (const [source, expected] of [
    ['export function value():9007199254740993{return 9007199254740993;} function main(){}', /outside JavaScript's safe integer range/],
    ['type Link={next:Link|null;}; export function value(input:Link):Link{return input;} function main(){}', /recursive.*wire data/],
    ["import {TrackedResource,trackedResource} from 'std:testing'; export function value():TrackedResource{return trackedResource();} function main(){}", /native resources/],
  ]) {
    writeBackend(source);
    assert.match(command(compiler, generateChanged, 1), expected);
    assert.equal(fs.readFileSync(path.join(changedOutput, 'index.d.ts'), 'utf8'), currentDeclaration, 'unsupported wire types preserve the previous package');
  }
  writeBackend("import {TrackedResource,trackedResource} from 'std:testing'; export function value():int{return 3;} export function privateResource():TrackedResource{return trackedResource();}");
  changedOptions.apiVersion = 3;
  changedOptions.package.version = '2.0.0';
  fs.writeFileSync(changedConfig, JSON.stringify(changedOptions));
  command(compiler, generateChanged);
  assert.doesNotMatch(fs.readFileSync(path.join(changedOutput, 'index.js'), 'utf8'), /TrackedResource|privateResource/);
  const existingApp = path.join(directory, 'existing-app');
  fs.mkdirSync(existingApp);
  const ownerPackage = '{"name":"owner-app","scripts":{"dev":"my-own-tool"}}';
  fs.writeFileSync(path.join(existingApp, 'package.json'), ownerPackage);
  assert.match(command(compiler, ['api', 'generate', '--config', config, '--out', existingApp], 1), /empty or previously generated/);
  assert.equal(fs.readFileSync(path.join(existingApp, 'package.json'), 'utf8'), ownerPackage);
  const scopeDirectory = path.join(existingApp, 'node_modules/@example');
  fs.mkdirSync(scopeDirectory, { recursive: true });
  fs.symlinkSync(output, path.join(scopeDirectory, 'tasks-api'), process.platform === 'win32' ? 'junction' : 'dir');
  assert.match(command(compiler, ['api', 'generate', '--config', config, '--out', path.join(scopeDirectory, 'tasks-api')], 1), /link/);
  const imported = path.join(existingApp, 'consumer.mjs');
  fs.writeFileSync(imported, `import {createClient} from '@example/tasks-api';
const api=createClient({async request(){return {ok:true,value:2};}});
if(await api.math.count(1)!==2)throw new Error('Exported package failed');
`);
  command(process.execPath, [imported]);
  const importedTypes = path.join(existingApp, 'consumer.mts');
  fs.writeFileSync(importedTypes, fs.readFileSync(consumer, 'utf8').replace("'./client/index.js'", "'@example/tasks-api'"));
  command(process.execPath, [tsc, importedTypes, '--strict', '--noEmit', '--module', 'NodeNext', '--target', 'ES2022', '--lib', 'ES2022']);
  assert.equal(fs.readFileSync(path.join(existingApp, 'package.json'), 'utf8'), ownerPackage, 'standalone consumption does not adopt frontend tooling');
  console.log('API client: named calls, strict codecs, safe integers, errors, type inference, portable package and owner-edit guards passed.');
} finally {
  assert.equal(path.dirname(directory), path.join(root, 'target'));
  assert.ok(path.basename(directory).startsWith('api-client-'));
  fs.rmSync(directory, { recursive: true, force: true });
}
