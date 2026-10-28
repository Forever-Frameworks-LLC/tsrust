// Headless admission of ordinary npm consumers; never publishes or changes a user app.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const compiler = process.env.TSRUST || path.join(root, 'target/debug/tsrust' + (process.platform === 'win32' ? '.exe' : ''));
const fixture = path.join(root, 'tools/api-client-tests');
const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'tsrust-api-package-'));
const npmCli = process.env.npm_execpath || path.join(path.dirname(process.execPath), 'node_modules/npm/bin/npm-cli.js');
const windowsNpm = process.platform === 'win32';
if (windowsNpm) assert.ok(fs.existsSync(npmCli), 'Run through npm or install the standard Node/npm distribution');
function command(file, args, cwd = directory) {
  const result = spawnSync(file, args, { cwd, encoding: 'utf8', timeout: 120000, windowsHide: true, maxBuffer: 8 * 1024 * 1024 });
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stdout + result.stderr);
  return result.stdout;
}
function npm(args, cwd) {
  return command(windowsNpm ? process.execPath : 'npm', [...(windowsNpm ? [npmCli] : []), ...args], cwd);
}
function json(file, value) { fs.writeFileSync(file, JSON.stringify(value, null, 2) + '\n'); }
function app(name, manifest) {
  const folder = path.join(directory, name);
  fs.mkdirSync(folder);
  json(path.join(folder, 'package.json'), { name, private: true, type: 'module', ...manifest });
  return folder;
}
const install = ['install', '--offline', '--ignore-scripts', '--no-audit', '--no-fund'];
const consumerSource = `import assert from 'node:assert/strict';
import { createClient } from '@example/query-api';
const transport = { async request(operation, input) { assert.equal(operation, 'math.count'); return { ok: true, value: input + 1 }; } };
assert.equal(await createClient(transport).math.count(4), 5);
`;

try {
  const output = path.join(directory, 'exported');
  command(compiler, ['api', 'generate', '--config', path.join(root, 'examples/api-contract/tsrust.query.api.json'), '--out', output]);
  command(compiler, ['api', 'generate', '--config', path.join(root, 'examples/api-contract/tsrust.query.api.json'), '--out', output, '--check']);
  const generated = JSON.parse(fs.readFileSync(path.join(output, 'package.json')));
  fs.writeFileSync(path.join(output, 'server-secret.tsr'), 'must never enter the client package');
  fs.writeFileSync(path.join(output, '.env'), 'TOKEN=must-not-ship');
  const [packed] = JSON.parse(npm(['pack', '--json', '--ignore-scripts', '--pack-destination', directory], output));
  assert.equal(packed.name, generated.name);
  assert.equal(packed.version, generated.version);
  assert.match(packed.integrity, /^sha512-/);
  assert.deepEqual(packed.files.map(file => file.path).sort(), [...generated.files, 'package.json'].sort());
  const tarball = path.join(directory, packed.filename);
  assert.ok(fs.existsSync(tarball));
  const plain = app('plain-consumer', { scripts: { check: 'node consumer.mjs' } });
  fs.writeFileSync(path.join(plain, 'consumer.mjs'), consumerSource);
  npm([...install, tarball], plain);
  assert.ok(!fs.existsSync(path.join(plain, 'node_modules/react')), 'optional peers do not burden plain clients');
  assert.ok(!fs.existsSync(path.join(plain, 'node_modules/@tanstack')), 'plain entry installs no framework');
  command(process.execPath, ['consumer.mjs'], plain);

  const deps = JSON.parse(fs.readFileSync(path.join(fixture, 'package.json'))).devDependencies;
  const existing = app('existing-consumer', { scripts: { check: 'node consumer.mjs', owner: 'echo owner-script' }, devDependencies: deps, ownerSetting: { retained: true } });
  const ownerSource = '// Existing application source stays byte-identical.\n';
  fs.writeFileSync(path.join(existing, 'owner.ts'), ownerSource);
  fs.writeFileSync(path.join(existing, '.npmrc'), 'save-exact=true\n');
  fs.writeFileSync(path.join(existing, 'consumer.mjs'), consumerSource);
  npm([...install, tarball], existing);
  // Remove generation inputs: installed consumers must be self-contained.
  assert.equal(path.dirname(fs.realpathSync(output)), fs.realpathSync(directory));
  fs.rmSync(output, { recursive: true, force: true });
  command(process.execPath, ['consumer.mjs'], existing);
  const types = `import { createClient } from '@example/query-api';
const api = createClient({ async request() { return {ok:true,value:5}; } });
const value: number = await api.math.count(4);
// @ts-expect-error compiler-derived input rejects strings
api.math.count('wrong');
// @ts-expect-error compiler-derived result remains numeric
const wrong: string = value;
`;
  fs.writeFileSync(path.join(existing, 'consumer.ts'), types);
  command(process.execPath, ['node_modules/typescript/bin/tsc', 'consumer.ts', '--strict', '--noEmit', '--module', 'NodeNext', '--target', 'ES2022', '--lib', 'ES2022,DOM'], existing);
  const before = JSON.parse(fs.readFileSync(path.join(existing, 'package.json')));
  const addition = app('owner-added-dependency', { version: '1.0.0', exports: './index.js' });
  fs.writeFileSync(path.join(addition, 'index.js'), 'export const ownerValue = 42;\n');
  npm([...install, '--install-links', addition], existing);
  const after = JSON.parse(fs.readFileSync(path.join(existing, 'package.json')));
  assert.deepEqual(after.scripts, before.scripts);
  assert.deepEqual(after.devDependencies, before.devDependencies);
  assert.deepEqual(after.ownerSetting, before.ownerSetting);
  assert.equal(fs.readFileSync(path.join(existing, 'owner.ts'), 'utf8'), ownerSource);
  assert.equal(fs.readFileSync(path.join(existing, '.npmrc'), 'utf8'), 'save-exact=true\n');
  assert.ok(fs.existsSync(path.join(existing, 'package-lock.json')));
  command(process.execPath, ['--input-type=module', '-e', "import assert from 'node:assert/strict'; import {ownerValue} from 'owner-added-dependency'; assert.equal(ownerValue,42)"], existing);
  command(process.execPath, ['consumer.mjs'], existing);

  const workspace = app('workspace-consumer', { workspaces: ['packages/*', 'web'], devDependencies: deps });
  fs.mkdirSync(path.join(workspace, 'packages'));
  const local = path.join(workspace, 'packages/api');
  command(compiler, ['api', 'generate', '--config', path.join(root, 'examples/api-contract/tsrust.query.api.json'), '--out', local]);
  fs.mkdirSync(path.join(workspace, 'web'));
  json(path.join(workspace, 'web/package.json'), { name: 'owner-web', private: true, type: 'module', dependencies: { [generated.name]: generated.version } });
  fs.writeFileSync(path.join(workspace, 'web/consumer.mjs'), consumerSource);
  npm(install, workspace);
  assert.equal(fs.realpathSync(path.join(workspace, 'node_modules/@example/query-api')), fs.realpathSync(local));
  command(process.execPath, ['consumer.mjs'], path.join(workspace, 'web'));
  console.log(`API package: npm ${npm(['--version']).trim()}, exact tarball allowlist/integrity, detached plain/TypeScript consumers, user dependency edits and npm workspace imports passed. No publication.`);
} finally {
  assert.equal(path.dirname(fs.realpathSync(directory)), fs.realpathSync(os.tmpdir()));
  assert.ok(path.basename(directory).startsWith('tsrust-api-package-'));
  fs.rmSync(directory, { recursive: true, force: true });
}
