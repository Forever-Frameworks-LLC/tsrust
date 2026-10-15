import {spawnSync} from 'node:child_process';
import {readFileSync, writeFileSync, appendFileSync, mkdtempSync, rmSync, readdirSync} from 'node:fs';
import {createHash} from 'node:crypto';
import path from 'node:path';
import {tmpdir} from 'node:os';
import {fileURLToPath} from 'node:url';
export const root = path.dirname(fileURLToPath(import.meta.url));
export const compiler = process.env.TSRUST || '/Users/seancotter/.cargo/bin/tsrust';
export const pin = '12f069627c4e253b6656028edf384d99c9fcba8eb956c45364e401318fd7cc89';
export const sha = file => createHash('sha256').update(readFileSync(file)).digest('hex');
export function verifyCompiler() { if (sha(compiler) !== pin) throw new Error('Compiler identity changed: verify provenance before changing the pin'); }
export function run(command, args, expected = 0) {
  const result = spawnSync(command, args, {cwd: root, encoding: 'utf8', timeout: 30000,
    maxBuffer: 16 * 1024 * 1024, env: {...process.env, CARGO_INCREMENTAL: '0'}});
  const record = {command, args, status: result.status, signal: result.signal, error: result.error?.message,
    stdout: result.stdout, stderr: result.stderr};
  if (process.env.UI_EVIDENCE_LOG) appendFileSync(process.env.UI_EVIDENCE_LOG, JSON.stringify(record) + '\n');
  if (result.error || result.signal || result.status !== expected) throw new Error(JSON.stringify(record));
  return result;
}
export function temporary(fn) {
  const dir = mkdtempSync(path.join(tmpdir(), 'ui-tsrust-handles-'));
  try { return fn(dir); } finally { rmSync(dir, {recursive: true, force: true}); }
}
// shortcut: lexical checks cover these simple witnesses only; no template/regex source support.
export function tokens(source) {
  const scan = /\s+|\/\/[^\n]*|'(?:[^'\\]|\\.)*'|"(?:[^"\\]|\\.)*"|[A-Za-z_$][\w$]*|\d+|===|!==|=>|==|!=|<=|>=|&&|\|\||\+\+|--|[{}()[\];:,.+*=<>!?|&\/%-]/gy;
  const result = [];
  for (let at = 0; at < source.length;) {
    scan.lastIndex = at; const match = scan.exec(source);
    if (!match) throw new Error(`Unsupported fixture token at ${at}`);
    if (!/^\s/.test(match[0])) result.push(match[0]);
    at = scan.lastIndex;
  }
  return result;
}
export function formattedWitnesses(dir, assert) {
  for (const file of readdirSync(path.join(root, 'witnesses')).filter(x => x.endsWith('.tsr'))) {
    const source = readFileSync(path.join(root, 'witnesses', file), 'utf8');
    const dest = path.join(dir, file); writeFileSync(dest, source);
    const formatted = run(compiler, ['format', dest]).stdout;
    assert.deepEqual(tokens(formatted), tokens(source), file);
    writeFileSync(dest, formatted);
    assert.equal(run(compiler, ['format', dest]).stdout, formatted, file);
  }
}
export function execute(source, dir) {
  verifyCompiler();
  run(compiler, ['check', source]);
  const emitted = path.join(dir, 'witness.rs'), binary = path.join(dir, 'witness');
  run(compiler, ['emit', source, '-o', emitted]);
  run('rustc', ['--edition=2024', '--crate-name', 'ui_owner_witness', emitted, '-o', binary]);
  const result = run(binary, []);
  verifyCompiler();
  return result.stdout;
}

export const historicalCommit = 'b255009e366357858ab964e486353dd2e5ff5bb9';
export function historicalWitnesses(dir, assert) {
  for (const name of ['owner.tsr', 'domain-collision.tsr', 'runtime-boundary.tsr', 'named-self-replacement.tsr', 'factory-self-replacement.tsr']) {
    const source = run('git', ['show', `${historicalCommit}:experiments/ui-tsrust-owner-handles/witnesses/${name}`]).stdout;
    const file = path.join(dir, name); writeFileSync(file, source);
    const formatted = run(compiler, ['format', file]).stdout;
    assert.deepEqual(tokens(formatted), tokens(source)); writeFileSync(file, formatted);
    assert.equal(run(compiler, ['format', file]).stdout, formatted);
  }
}

export function reducedIdentityLimit(dir) {
  const file = path.join(dir, 'owner.tsr');
  const source = readFileSync(file, 'utf8');
  if ((source.match(/2147483647/g) || []).length !== 3) throw new Error('Identity limit seam changed');
  // Only the ceiling changes; allocation and failure paths are the production candidate's.
  writeFileSync(file, source.replaceAll('2147483647', '3'));
}
