import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {appendFileSync, readFileSync, mkdtempSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
export const root = path.dirname(fileURLToPath(import.meta.url));
export const compiler = process.env.TSRUST || '/Users/seancotter/.cargo/bin/tsrust';
export const verifiedCompilerHash = 'cae21ac7d026d98f879656cc0776cb5080a46a681251ec7e3e9160bbc0b20c6d';
export function hash(file) { return createHash('sha256').update(readFileSync(file)).digest('hex'); }
export function run(command, args, {cwd = root, expected = 0, timeout = 30000} = {}) {
  const result = spawnSync(command, args, {cwd, encoding: 'utf8', timeout, maxBuffer: 8 * 1024 * 1024,
    env: {...process.env, CARGO_INCREMENTAL: '0'}});
  const record = {command, args, cwd, status: result.status, signal: result.signal,
    error: result.error?.message, stdout: result.stdout, stderr: result.stderr};
  if (process.env.UI_EVIDENCE_LOG) appendFileSync(process.env.UI_EVIDENCE_LOG, JSON.stringify(record) + '\n');
  if (result.error || result.signal || result.status !== expected) {
    throw new Error(JSON.stringify(record));
  }
  return result;
}
export function temporary(action) {
  const dir = mkdtempSync(path.join(tmpdir(), 'ui-state-lifecycle-'));
  try { return action(dir); } finally { rmSync(dir, {recursive: true, force: true}); }
}
export function rust(source, output, test = false) {
  run('rustc', ['--edition=2024', '--crate-name', 'ui_lifecycle', ...(test ? ['--test'] : []), source, '-o', output]);
}
