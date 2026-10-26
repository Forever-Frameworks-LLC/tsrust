import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const compiler = process.env.TSRUST || path.join(root, 'target/debug/tsrust' + (process.platform === 'win32' ? '.exe' : ''));
const executable = path.join(root, 'target/task-repository-native' + (process.platform === 'win32' ? '.exe' : ''));
const environment = {
  ...process.env, CARGO_INCREMENTAL: '0', CARGO_BUILD_JOBS: '1',
  CARGO_TARGET_DIR: process.env.CARGO_TARGET_DIR || path.join(root, 'target'),
  TSRUST_CACHE_DIR: process.env.TSRUST_CACHE_DIR || path.join(root, 'target/tsrust-native'),
};
function command(file, args, env, timeout) {
  const result = spawnSync(file, args, { cwd: root, env, encoding: 'utf8', timeout, windowsHide: true, maxBuffer: 4 * 1024 * 1024 });
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stdout + result.stderr);
  return result.stdout;
}

assert.ok(process.argv.length === 2 || process.argv.length === 3 && process.argv[2] === '--run-only', 'Use --run-only to reuse the already built witness');
if (!process.argv.includes('--run-only')) {
  command(compiler, ['build', 'examples/backend-tasks/witness.tsr', '-o', executable], environment, 600000);
}
fs.mkdirSync(path.join(root, 'target'), { recursive: true });
const directory = fs.mkdtempSync(path.join(root, 'target/task-repository-'));
try {
  const databasePath = path.join(directory, 'task data.sqlite');
  const env = { ...environment, TASK_TEST_DATABASE_URL: `sqlite:${databasePath.replaceAll('\\', '/')}?mode=rwc`, TASK_TEST_PHASE: 'write' };
  assert.equal(command(executable, [], env, 30000), 'Task repository CRUD and validation passed\n');
  assert.ok(fs.statSync(databasePath).size > 0, 'native SQLite database must exist on disk');
  assert.equal(command(executable, [], { ...env, TASK_TEST_PHASE: 'verify' }, 30000), 'Task repository process-restart persistence passed\n');
  console.log('Task repository: native CRUD, validation, parameterized SQL, bounded cursors and process-restart SQLite persistence passed.');
} finally {
  assert.equal(path.dirname(directory), path.join(root, 'target'));
  assert.ok(path.basename(directory).startsWith('task-repository-'));
  fs.rmSync(directory, { recursive: true, force: true });
}
