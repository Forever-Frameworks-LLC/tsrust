import test from 'node:test';
import assert from 'node:assert/strict';
import {quietEnough, waitForQuietMachine} from './quiet-machine.mjs';

test('timing waits for a quiet machine and refuses a persistently busy one', () => {
  assert.equal(quietEnough(0.8, 2), true);
  assert.equal(quietEnough(2.5, 2), false);
  assert.equal(quietEnough(Number.NaN), false);
  const loads = [5, 3, 1.2];
  const load = waitForQuietMachine({limit: 2, loadAverage: () => loads.shift(), sleep: () => {}, log: () => {}});
  assert.equal(load, 1.2);
  assert.throws(() => waitForQuietMachine({loadAverage: () => 6, sleep: () => {}, log: () => {}, timeoutMs: 0}), /Machine stayed busy/);
});
