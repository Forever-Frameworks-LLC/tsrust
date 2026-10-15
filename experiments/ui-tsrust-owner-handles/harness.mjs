import {spawnSync} from 'node:child_process';
import {readFileSync, writeFileSync, appendFileSync, mkdtempSync, rmSync, readdirSync} from 'node:fs';
import {createHash} from 'node:crypto';
import path from 'node:path';
import {tmpdir} from 'node:os';
import {fileURLToPath} from 'node:url';
export const root = path.dirname(fileURLToPath(import.meta.url));
export const compiler = process.env.TSRUST || '/Users/seancotter/.cargo/bin/tsrust';
export const pin = '57fd08aad9358fa367434bb5f5f036885b0148a7c0994939addea8f230aceca3';
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

// Known compiler panic, not an accepted ownership rejection; unexpected outcomes fail.
export function checkNamedHandlerBlocker(source, dir) {
  verifyCompiler();
  for (const command of ['check', 'emit']) {
    const args = command === 'emit' ? [command, source, '-o', path.join(dir, 'blocked.rs')] : [command, source];
    const result = run(compiler, args, 101);
    if (!/panicked at crates\/tsrust-compiler\/src\/analysis\/mod\.rs/.test(result.stderr) ||
        !/index out of bounds: the len is 22 but the index is 22/.test(result.stderr)) {
      throw new Error(`Unexpected ${command} failure: ${result.stderr}`);
    }
  }
  verifyCompiler();
  return 'B4: named-handler self-replacement panics during check and emit; factory control executes';
}
