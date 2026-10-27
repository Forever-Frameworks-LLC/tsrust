import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { spawn } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const example = path.join(root, 'examples/backend-tasks');
const { startBackend, deadline } = await import(pathToFileURL(path.join(example, 'native/src/backend.ts')));
const executable = path.join(example, 'native/resources/backend/tasks-backend' + (process.platform === 'win32' ? '.exe' : ''));
const directory = fs.mkdtempSync(path.join(root, 'target/desktop-backend-'));
const migrations = path.join(example, 'migrations');
const database = path.join(directory, 'tasks with # and %.sqlite');
let active, direct;
try {
  await assert.rejects(startBackend({ executable: path.join(directory, 'missing.exe'), database, migrations }), /could not be launched/);
  await assert.rejects(startBackend({ executable: process.execPath, database, migrations, startupTimeoutMs: 100 }), /readiness timed out/);
  await assert.rejects(startBackend({ executable, database, migrations: path.join(directory, 'missing-migrations') }), /exited before readiness/);
  active = await startBackend({ executable, database, migrations });
  assert.equal((await active.api.tasks.list({ after: 0, limit: 100 })).items.length, 0);
  const task = await active.api.tasks.create({ title: 'Desktop persistence' }); assert.equal(task.kind, 'task');
  assert.equal((await active.api.tasks.update({ id: task.id, title: 'Updated desktop', completed: true })).completed, true);
  assert.equal((await active.api.tasks.delete({ id: task.id })).kind, 'deleted');
  await active.api.tasks.create({ title: 'Survives graceful restart' });
  const stop = active.stop(); assert.equal(active.stop(), stop, 'shutdown is idempotent'); await stop;
  assert.equal((await active.exited).code, 0); active = undefined;
  active = await startBackend({ executable, database, migrations });
  assert.equal((await active.api.tasks.list({ after: 0, limit: 100 })).items[0].title, 'Survives graceful restart');
  const crashed = active.exited; process.kill(active.pid); await deadline(crashed, 5000, 'Native crash was not observed');
  await active.stop(); active = undefined;

  const pipeDatabase = path.join(directory, 'pipe.sqlite');
  const token = 'b'.repeat(64);
  direct = spawn(executable, [], { windowsHide: true, stdio: ['pipe', 'pipe', 'pipe'], env: { ...process.env,
    TASK_DESKTOP_TOKEN: token, TASK_DESKTOP_DATABASE_URL: `sqlite:${pipeDatabase.replaceAll('\\', '/')}?mode=rwc`, TASK_DESKTOP_MIGRATIONS: migrations,
  } });
  direct.stdin.on('error', () => {}); let errors = '', output = '';
  direct.stderr.on('data', chunk => { errors += chunk; });
  const exited = new Promise(resolve => direct.once('close', resolve));
  const address = await deadline(new Promise((resolve, reject) => {
    direct.once('error', reject); direct.once('exit', code => reject(new Error(`Parent-pipe fixture exited ${code}: ${errors}`)));
    direct.stdout.on('data', chunk => { output += chunk; const line = output.split('\n').slice(0, -1)[0]; if (line) { try { resolve(JSON.parse(line).address); } catch (error) { reject(error); } } });
  }), 20000, 'Parent-pipe fixture did not become ready');
  const request = async authorization => fetch(`http://${address}/__desktop_shutdown`, { method: 'POST', headers: { authorization }, signal: AbortSignal.timeout(5000) });
  const denied = await request('Bearer wrong'); assert.equal(denied.status, 403); await denied.text();
  direct.stdin.end();
  assert.equal(await deadline(exited, 5000, 'Orphan did not exit after parent-pipe EOF'), 74);
  direct = undefined;
  console.log('Desktop backend: startup error/timeout/migration failure, native CRUD, encoded local DB paths, persistence, idempotent graceful exit, crash observation, denied shutdown and parent-pipe orphan exit passed.');
} finally {
  if (active) await active.stop();
  if (direct && direct.exitCode === null) { direct.stdin.end(); direct.kill(); }
  assert.equal(path.dirname(fs.realpathSync(directory)), fs.realpathSync(path.join(root, 'target')));
  assert.ok(path.basename(directory).startsWith('desktop-backend-')); fs.rmSync(directory, { recursive: true, force: true });
}
