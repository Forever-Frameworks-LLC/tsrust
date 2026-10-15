import test from 'node:test';
import assert from 'node:assert/strict';
import path from 'node:path';
import {readFileSync, writeFileSync} from 'node:fs';
import {root, compiler, run, temporary, formattedWitnesses, execute, verifyCompiler, tokens, checkNamedHandlerBlocker} from '../harness.mjs';

test('H01: verified compiler; command failures and unsupported scanner input fail loudly', () => {
  verifyCompiler(); run(compiler, ['--version']); run('rustc', ['--version']);
  assert.throws(() => run(process.execPath, ['-e', 'process.exit(7)']), /"status":7/);
  assert.throws(() => run('/nonexistent/ui-handles-tool', []), /ENOENT/);
  assert.throws(() => tokens('`unsupported`'), /Unsupported/);
  assert.notDeepEqual(tokens("'a b'"), tokens("'ab'"));
  assert.notDeepEqual(tokens('a >= b'), tokens('a > = b'));
});
test('H02-H13,H16,H18: formatted source model executes natively; scopes and drop limits stay explicit', () => temporary(dir => {
  formattedWitnesses(dir, assert);
  for (const file of ['retained-registry.tsr', 'extracted-dispatch.tsr', 'owner-smoke.tsr']) {
    assert.equal(execute(path.join(dir, file), dir), '');
  }
  const output = ['owners-lists.tsr', 'disposal.tsr', 'callback-lists.tsr', 'cycles.tsr'].map(file => execute(path.join(dir, file), dir)).join('');
  assert.equal(output, [
    'H02-H04 scoped owners, retained registry and non-owning handles: executed',
    'H05-H07 queued stale events, fresh identities and keyed lists: executed',
    'H08-H09 FIFO and recursive drain rejection: executed',
    'H10 self/ancestor/root removal ends active state access; sibling-only removal preserves it: executed; B3 adopted; resource contract blocked by B2',
    'H11 callback removal regression: executed', 'H12 callback removal regression: executed',
    'H13 callback removal regression: executed',
    'H16 1000 logical cleanup cycles and checked exhaustion: executed; resource drops unproven',
    '',
  ].join('\n'));
  console.log(output.trim());
  assert.equal(execute(path.join(dir, 'domain-collision.tsr'), dir), 'BLOCKED: handles from independent allocator scopes collide\n');
  assert.equal(execute(path.join(dir, 'scope-exit.tsr'), dir), 'BLOCKED: exact state/capture destruction is not observable in these witnesses\n');
  const source = readFileSync(path.join(dir, 'owners-lists.tsr'), 'utf8');
  const mutation = source.replace('strictEqual(read(a, x), 6)', 'strictEqual(read(a, x), 99)');
  assert.notEqual(source, mutation);
  const file = path.join(dir, 'mutation.tsr'); writeFileSync(file, mutation);
  run(compiler, ['emit', file, '-o', path.join(dir, 'mutation.rs')]);
  run('rustc', ['--edition=2024', path.join(dir, 'mutation.rs'), '-o', path.join(dir, 'mutation')]);
  assert.match(run(path.join(dir, 'mutation'), [], 1).stderr, /AssertionError: Expected values to be strictly equal/);
  verifyCompiler();
}));
test('H09,H17: direct registry overlap and existing ownership violations reject strictly', () => temporary(dir => {
  const prior = path.resolve(root, '../ui-state-lifecycle/witnesses');
  const cases = [
    [path.join(root, 'witnesses/direct-dispatch.tsr'), 'conflicting-access'],
    [path.join(prior, 'escaped-loan.tsr'), 'closure-escape'],
    [path.join(prior, 'overlapping-access.tsr'), 'conflicting-access'],
    [path.join(prior, 'consumed-owner.tsr'), 'transferred'],
  ];
  for (const [file, code] of cases) {
    assert.match(run(compiler, ['emit', file], 1).stderr, new RegExp(code));
    const strict = readFileSync(file, 'utf8').replace(/^\s*\/\/ @tsrust ignore[^\n]*\n/gm, '\n');
    const input = path.join(dir, 'rejection.tsr'); writeFileSync(input, strict);
    for (const command of ['check', 'emit']) assert.match(run(compiler, [command, input], 1).stderr, new RegExp(code));
  }
}));
test('H01-H19: every row is dispositioned; incomplete contract is explicitly blocked', () => {
  const matrix = JSON.parse(readFileSync(path.join(root, 'matrix.json'), 'utf8'));
  assert.deepEqual(matrix.rows.map(row => row.id), Array.from({length: 19}, (_, i) => `H${String(i + 1).padStart(2, '0')}`));
  const statuses = ['executed', 'analysis/emission only', 'expected rejection', 'blocked', 'not applicable with reason'];
  for (const row of matrix.rows) { assert.ok(statuses.includes(row.status)); assert.ok(row.evidence.length); }
  assert.equal(matrix.rows.at(-1).status, 'blocked');
  const h10 = matrix.rows.find(row => row.id === 'H10');
  assert.equal(h10.status, 'blocked');
  assert.match(h10.evidence, /B2/);
  assert.equal(matrix.blockers.B3.status, 'resolved by adopted contract');
  assert.deepEqual(matrix.blockers.B3.dependentRows, ['H10', 'H19']);
  for (const blocker of Object.values(matrix.blockers)) assert.ok(readFileSync(path.join(root, blocker.witness)).length);
  console.log(JSON.stringify({contract: 'blocked', rows: matrix.rows}, null, 2));
});

test('R03-R05,R09: runtime boundary counterexample and real resource capture execute; follow-up matrix is complete', () => temporary(dir => {
  formattedWitnesses(dir, assert);
  assert.equal(execute(path.join(dir, 'runtime-boundary.tsr'), dir), 'B1: hiding the allocator in a retained closure does not prevent cross-runtime handle aliasing\n');
  assert.equal(execute(path.join(dir, 'resource-capture.tsr'), dir), 'B2: real File capture runs and leaves the registry; exact release count and order remain unobserved\n');
  const followup = JSON.parse(readFileSync(path.join(root, 'resource-matrix.json'), 'utf8'));
  assert.deepEqual(followup.rows.map(row => row.id), Array.from({length:19}, (_,i)=>`R${String(i+1).padStart(2,'0')}`));
  const old = JSON.parse(readFileSync(path.join(root, 'matrix.json'), 'utf8'));
  assert.deepEqual(Object.keys(followup.reconciliation), old.rows.map(row=>row.id));
  for(const row of followup.rows) {
    assert.ok(['executed','analysis/emission only','expected rejection','blocked','not applicable with reason'].includes(row.status));
    assert.ok(row.evidence.length);
    if(row.status === 'blocked') assert.ok(row.blockers.length);
  }
  assert.equal(followup.decisions.B3, 'adopted');
  assert.equal(followup.contract, 'blocked');
  assert.equal(followup.rows.find(row=>row.id==='R09').status, 'blocked');
  assert.equal(followup.rows.find(row=>row.id==='R19').status, 'blocked');
}));

test('B4: exact named-handler compiler panic and native factory control stay distinct', () => temporary(dir => {
  formattedWitnesses(dir, assert);
  assert.equal(checkNamedHandlerBlocker(path.join(dir, 'named-self-replacement.tsr'), dir),
    'B4: named-handler self-replacement panics during check and emit; factory control executes');
  assert.equal(execute(path.join(dir, 'factory-self-replacement.tsr'), dir),
    'PASS active self-replacement, stale queued event, unaffected FIFO, reentrant rejection, final root removal\n');
  const matrix = JSON.parse(readFileSync(path.join(root, 'matrix.json'), 'utf8'));
  const followup = JSON.parse(readFileSync(path.join(root, 'resource-matrix.json'), 'utf8'));
  assert.equal(matrix.blockers.B4.kind, 'compiler analysis panic');
  for (const row of matrix.rows) assert.deepEqual(followup.reconciliation[row.id], {status: row.status, evidence: row.evidence});
  assert.ok(followup.rows.find(row=>row.id==='R08').limitations.includes('B4'));
  assert.ok(followup.rows.find(row=>row.id==='R19').blockers.includes('B4'));
}));
