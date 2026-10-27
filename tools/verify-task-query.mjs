import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const fixture = path.join(root, 'tools/api-client-tests');
const compiler = process.env.TSRUST || path.join(root, 'target/debug/tsrust' + (process.platform === 'win32' ? '.exe' : ''));
assert.ok(process.argv.length === 2 || process.argv.length === 3 && process.argv[2] === '--run-only');
const directory = fs.mkdtempSync(path.join(fixture, '.tmp-task-query-'));
const client = path.join(directory, 'client');
function command(file, args, env = process.env, timeout = 60000) {
  const result = spawnSync(file, args, { cwd: root, env, encoding: 'utf8', timeout, maxBuffer: 8 * 1024 * 1024, windowsHide: true });
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stdout + result.stderr);
  if (result.stdout) process.stdout.write(result.stdout);
}
try {
  const args = ['api', 'generate', '--config', path.join(root, 'examples/backend-tasks/tsrust.api.json'), '--out', client];
  command(compiler, args);
  command(compiler, [...args, '--check']);
  command(process.execPath, [path.join(root, 'tools/verify-task-http.mjs'), ...process.argv.slice(2)], { ...process.env, TSRUST_TASK_CLIENT_DIR: client, TSRUST_TASK_QUERY: '1' }, 900000);
} finally {
  assert.equal(path.dirname(fs.realpathSync(directory)), fs.realpathSync(fixture));
  assert.ok(path.basename(directory).startsWith('.tmp-task-query-'));
  fs.rmSync(directory, { recursive: true, force: true });
}
