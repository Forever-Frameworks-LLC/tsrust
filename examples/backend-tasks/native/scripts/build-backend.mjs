import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
const native = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const backend = path.resolve(native, '..');
const destination = path.join(native, 'resources/backend');
fs.mkdirSync(destination, { recursive: true });
for (const args of [ ['api', 'generate', '--config', path.join(backend, 'tsrust.api.json')],
  ['build', path.join(backend, 'desktop.tsr'), '-o', path.join(destination, 'tasks-backend' + (process.platform === 'win32' ? '.exe' : ''))] ]) {
  const result = spawnSync(process.env.TSRUST || 'tsrust', args, { cwd: backend,
    env: { ...process.env, CARGO_INCREMENTAL: '0', CARGO_BUILD_JOBS: '1' }, stdio: 'inherit', windowsHide: true });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}
for (const extension of ['.pdb', '.exp', '.lib']) fs.rmSync(path.join(destination, 'tasks-backend' + extension), { force: true });
if (process.platform === 'win32') {
  const script = fs.readFileSync(path.join(native, 'scripts/windows-runtime.ps1'), 'utf8');
  const prepared = spawnSync('powershell.exe', ['-NoProfile', '-NonInteractive', '-Command', script], {
    env: { ...process.env, TSRUST_DESKTOP_RUNTIME_DEST: destination, TSRUST_DESKTOP_RUNTIME_ARCH: process.arch },
    stdio: 'inherit', windowsHide: true,
  });
  if (prepared.error) throw prepared.error;
  if (prepared.status !== 0) process.exit(prepared.status ?? 1);
}
fs.cpSync(path.join(backend, 'migrations'), path.join(destination, 'migrations'), { recursive: true });
