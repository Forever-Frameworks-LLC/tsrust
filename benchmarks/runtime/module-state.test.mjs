import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertModuleStateSource,moduleStateABI,moduleStateDriver} from './module-state-contract.mjs';
import {expectedChecksum} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read = path => fs.readFileSync(new URL(path, import.meta.url), 'utf8');
const rows = JSON.parse(read('./manifest.json')).workloads.filter(row => row.id === 'native_module_state');
const row = rows[0];
test('module state workload preserves operations, oracle, and registration', () => {
  assert.equal(rows.length, 1);
  assert.equal(reportCategory(row.id), 'module-state');
  for (const [kind, file, key] of [['native', row.source, 'sourceSha256'], ['typescript', row.counterpart, 'counterpartSha256']]) {
    const source = read('./' + file);
    assertModuleStateSource(source, row, kind);
    const changed = source.replace('completed++;', 'completed += 2;');
    assert.throws(() => assertModuleStateSource(changed, {...row, [key]: createHash('sha256').update(changed).digest('hex')}, kind));
  }
  for (const n of [0, 1, 17, 2000]) assert.equal(expectedChecksum(row.id, n, 7), 7 + 3 * n);
});
test('module driver retains one exclusive owner outside sample loops', () => {
  const emitted = 'let value0: i64 = 0i64;\nfn tsr_result_function2(value0: i64, value1: i64, value2: &mut i64) -> Result<i64,TsrFunctionError2<Record0,String>> {\n}';
  const source = read('./' + row.source);
  assert.equal(moduleStateABI(emitted, row, source).functionName, 'tsr_result_function2');
  const driver = moduleStateDriver(read('./driver.rs'), row, emitted, source);
  assert.equal((driver.match(/let mut module_completed/g) ?? []).length, 1);
  assert.ok(driver.indexOf('let mut module_completed') < driver.indexOf('for _ in 0..warmups'));
  assert.match(driver, /&mut module_completed/);
  assert.throws(() => moduleStateABI(emitted.replace('&mut i64', '&i64'), row, source));
});
