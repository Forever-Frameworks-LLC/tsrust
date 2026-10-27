import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const compiler = process.env.TSRUST || path.join(root, 'target/debug/tsrust' + (process.platform === 'win32' ? '.exe' : ''));
const pnpm = process.env.PNPM || (process.platform === 'win32' ? path.join(process.env.APPDATA, 'npm/node_modules/pnpm/pnpm.exe') : 'pnpm');
const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'tsrust-web-clean-'));
const web = path.join(directory, 'web');
function command(file, args, cwd = directory) {
  const result = spawnSync(file, args, { cwd, encoding: 'utf8', timeout: 120000, windowsHide: true, maxBuffer: 8 * 1024 * 1024 });
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stdout + result.stderr);
  return result.stdout;
}
try {
  // The Windows temp folder can live on another volume. Reuse the admitted
  // source project's store explicitly instead of silently selecting a cold one.
  const store = command(pnpm, ['store', 'path'], path.join(root, 'examples/backend-tasks/web')).trim().split(/\r?\n/).at(-1);
  assert.ok(path.isAbsolute(store));
  assert.match(path.basename(store), /^v\d+$/);
  const storeOptions = ['--store-dir', path.dirname(store)];
  fs.cpSync(path.join(root, 'examples/backend-tasks/web'), web, { recursive: true, filter: source => !['node_modules', 'dist'].includes(path.basename(source)) });
  command(compiler, ['api', 'generate', '--config', path.join(root, 'examples/backend-tasks/tsrust.api.json'), '--out', path.join(directory, 'generated-client')]);
  const manifestFile = path.join(web, 'package.json');
  const manifest = fs.readFileSync(manifestFile, 'utf8');
  const source = fs.readFileSync(path.join(web, 'src/Tasks.tsx'), 'utf8');
  const lock = fs.readFileSync(path.join(web, 'pnpm-lock.yaml'), 'utf8');
  command(pnpm, ['install', '--offline', '--frozen-lockfile', '--ignore-scripts', ...storeOptions], web);
  assert.equal(fs.readFileSync(manifestFile, 'utf8'), manifest);
  assert.equal(fs.readFileSync(path.join(web, 'pnpm-lock.yaml'), 'utf8'), lock);
  const build = command(pnpm, ['build'], web);
  assert.match(build, /built in/);
  assert.ok(fs.existsSync(path.join(web, 'dist/index.html')));
  const extra = path.join(directory, 'owner-extra');
  fs.mkdirSync(extra);
  fs.writeFileSync(path.join(extra, 'package.json'), JSON.stringify({ name: 'owner-extra', version: '1.0.0', type: 'module', exports: './index.js' }));
  fs.writeFileSync(path.join(extra, 'index.js'), 'export const value = 42;\n');
  command(pnpm, ['add', '--offline', '--ignore-scripts', '--save-exact', 'file:../owner-extra', ...storeOptions], web);
  const changed = JSON.parse(fs.readFileSync(manifestFile));
  assert.deepEqual(changed.scripts, JSON.parse(manifest).scripts);
  assert.equal(changed.packageManager, JSON.parse(manifest).packageManager);
  assert.equal(fs.readFileSync(path.join(web, 'src/Tasks.tsx'), 'utf8'), source);
  command(process.execPath, ['--input-type=module', '-e', "import assert from 'node:assert/strict'; import {value} from 'owner-extra'; assert.equal(value,42)"], web);
  console.log('Task web clean adoption: external source copy, regenerated API, frozen offline pnpm install, TypeScript/production build and normal owner dependency addition passed. No compiler is used by frontend build.');
} finally {
  assert.equal(path.dirname(fs.realpathSync(directory)), fs.realpathSync(os.tmpdir()));
  assert.ok(path.basename(directory).startsWith('tsrust-web-clean-'));
  fs.rmSync(directory, { recursive: true, force: true });
}
