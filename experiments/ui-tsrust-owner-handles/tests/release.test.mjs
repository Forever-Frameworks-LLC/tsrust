import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import path from 'node:path';
import {root, temporary, verifyCompiler, pin} from '../harness.mjs';
import {prepareRelease, boundaryEvidence, nativeCalibration} from '../release.mjs';

test('L03-L07: actual native Drop calibration, wrong-expectation controls and transfer rejection', () => temporary(dir => {
  prepareRelease(dir); nativeCalibration(dir);
}));
test('L23: checked resource Drop rejects while plain owned data checks/emits/executes', () => temporary(dir => {
  prepareRelease(dir); boundaryEvidence(dir);
}));
test('L01-L26: complete scoped proof dispositions and historical reconciliation', () => {
  verifyCompiler();
  const matrix = JSON.parse(readFileSync(path.join(root, 'release-matrix.json'), 'utf8'));
  assert.deepEqual(matrix.rows.map(row => row.id), Array.from({length:26}, (_,i) => `L${String(i+1).padStart(2,'0')}`));
  assert.equal(matrix.synchronousB2, 'blocked');
  assert.equal(matrix.asyncWorkerReadiness, 'unproved; required future direction');
  for (const row of matrix.rows) {
    assert.ok(['executed','expected rejection','analysis/emission only','blocked','not applicable with reason'].includes(row.status), row.id);
    assert.equal(row.compiler_sha256, pin);
    for (const field of ['witness','expected','observed','evidence']) assert.ok(row[field], `${row.id}: ${field}`);
    assert.ok(row.commands.length, row.id);
    if (row.status === 'blocked') assert.ok(row.blockers.includes('B2-bridge'), row.id);
  }
  for (const [key, file] of [['H','matrix.json'],['R','resource-matrix.json'],['I','identity-matrix.json']]) {
    const prior = JSON.parse(readFileSync(path.join(root,file),'utf8'));
    assert.deepEqual(Object.keys(matrix.reconciliation[key]), prior.rows.map(row => row.id));
    for (const row of prior.rows) assert.equal(matrix.reconciliation[key][row.id].status, row.status, row.id);
  }
  assert.equal(matrix.rows.find(row => row.id === 'L18').status, 'blocked');
  assert.equal(matrix.rows.find(row => row.id === 'L11').status, 'blocked');
  assert.equal(matrix.observerCalibration, 'native-only; checked Tsrust boundary blocked');
});
