import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync, writeFileSync} from 'node:fs';
import path from 'node:path';
import {root, compiler, verifiedCompilerHash, hash, run, temporary, rust} from '../harness.mjs';

// shortcut: this scanner covers only these simple fixtures, not arbitrary Tsrust source.
// Add a compiler token API before extending witnesses to templates or regex literals.
function tokens(source) {
  const token = /\s+|\/\/[^\n]*|\/(?:\*(?:[^*]|\*(?!\/))*\*\/)|'(?:[^'\\]|\\.)*'|"(?:[^"\\]|\\.)*"|[A-Za-z_$][\w$]*|\d+|===|!==|=>|==|!=|<=|>=|&&|\|\||\+\+|--|[{}()[\];:,.+*=<>!?\/-]/gy;
  const result = [];
  for (let at = 0; at < source.length;) {
    token.lastIndex = at;
    const match = token.exec(source);
    assert.ok(match, `Unsupported fixture token at ${at}`);
    at = token.lastIndex;
    if (!/^\s/.test(match[0])) result.push(match[0]);
  }
  return result;
}
function formatted(source, dir) {
  const input = path.join(dir, 'input.tsr');
  writeFileSync(input, source);
  const output = run(compiler, ['format', input]).stdout;
  assert.deepEqual(tokens(output), tokens(source), 'token preservation');
  writeFileSync(input, output);
  assert.equal(run(compiler, ['format', input]).stdout, output, 'format idempotence');
  return input;
}
test('harness propagates failure, missing tool, timeout; scanner detects changed literals/operators', () => {
  assert.throws(() => run(process.execPath, ['-e', 'process.exit(7)']), /"status":7/);
  assert.throws(() => run('/nonexistent/ui-lifecycle-command', []), /ENOENT/);
  assert.throws(() => run(process.execPath, ['-e', 'setInterval(()=>{},1000)'], {timeout: 100}), /ETIMEDOUT/);
  assert.notDeepEqual(tokens("const s = 'a b';"), tokens("const s = 'ab';"));
  assert.notDeepEqual(tokens('a >= b'), tokens('a > = b'));
  assert.throws(() => tokens('`unsupported`'), /Unsupported/);
});
test('L01-L14,L17-L18: Rust ownership model and exact demo', () => temporary(dir => {
  const binary = path.join(dir, 'tests');
  rust(path.join(root, 'lifecycle.rs'), binary, true);
  const result = run(binary, ['--test-threads=1']);
  assert.match(result.stdout, /20 passed; 0 failed/);
  console.log(result.stdout.trim());
}));
test('L15-L16: verified compiler, formatted positives and strict ownership rejections', () => temporary(dir => {
  assert.equal(hash(compiler), verifiedCompilerHash, 'Compiler changed: verify new provenance before updating pin');
  run(compiler, ['--version']); run('rustc', ['--version']);
  const source = readFileSync(path.join(root, 'witnesses/retained-counter.tsr'), 'utf8');
  const input = formatted(source, dir);
  run(compiler, ['check', input]);
  const emitted = path.join(dir, 'counter.rs');
  run(compiler, ['emit', input, '-o', emitted]);
  const binary = path.join(dir, 'counter'); rust(emitted, binary);
  assert.equal(run(binary, []).stdout, '');
  // Independent reference for the counter script, without copying the ownership mechanism.
  const values = [0, 0];
  for (const target of [0, 0, 1]) values[target] += 1;
  assert.deepEqual(values, [2, 1]);
  // Mutating an expected value must fail the generated native assertion.
  writeFileSync(input, source.replace('strictEqual(first(), 2)', 'strictEqual(first(), 99)'));
  run(compiler, ['emit', input, '-o', emitted]); rust(emitted, binary);
  const mutation = run(binary, [], {expected: 101});
  assert.match(mutation.stderr, /AssertionError: Expected values to be strictly equal/);
  for (const [file, diagnostic] of [
    ['escaped-loan', 'closure-escape'], ['overlapping-access', 'conflicting-access'],
    ['consumed-owner', 'transferred'], ['retained-owner-conflict', 'capture-conflict'],
  ]) {
    const original = readFileSync(path.join(root, `witnesses/${file}.tsr`), 'utf8');
    const fixture = formatted(original, dir);
    // Emission must reject even with editor suppression still present.
    const emission = run(compiler, ['emit', fixture], {expected: 1});
    assert.match(emission.stderr, new RegExp(diagnostic), file);
    const strict = original.replace(/^\s*\/\/ @tsrust ignore[^\n]*\n/gm, '\n');
    writeFileSync(fixture, strict);
    for (const command of ['check', 'emit']) {
      const result = run(compiler, [command, fixture], {expected: 1});
      assert.match(result.stderr + result.stdout, new RegExp(diagnostic), `${file}: ${command}`);
    }
    console.log(`${file}: expected ${diagnostic} rejection, including strict analysis/emission`);
  }
  assert.equal(hash(compiler), verifiedCompilerHash, 'Compiler changed during validation');
  console.log('retained-counter: formatted, checked, emitted and natively executed; assertion mutation rejected');
  console.log('CAPABILITY GAP: externally invalidated non-owning handles and subtree disposal remain Rust-only; ordinary retained capture conflicts with owner access.');
}));
