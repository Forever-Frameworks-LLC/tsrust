import {test} from 'node:test';
import assert from 'node:assert/strict';
import {callableFoundationIds, expectedCallableFoundation} from './callable-foundation-oracle.mjs';
import {expectedChecksum} from './oracle.mjs';
test('checkpoint callable oracles match independent per-iteration specifications', () => {
  for (const n of [0, 1, 2, 3, 17, 1009, 20000]) {
    for (const seed of [0, 7, 1008]) {
      const totals = Object.fromEntries(callableFoundationIds.map(id => [id, seed]));
      for (let i = 0; i < n; i++) {
        totals.owned_callable_fields += 2 * (i + seed);
        totals.annotated_once_locals += 2 * i + seed;
        totals.once_factory_results += i + seed;
        totals.callable_array_mutable_pipeline += 6 * (i + seed) + 19;
        totals.callable_array_transfer_pipeline += 4 * (i + seed) + 12;
        totals.callable_array_once_transfer += 4 * (i + seed) + 10;
        const base = i + seed;
        totals.callable_array_reduction_pipeline += (base + 1) + (base + 2) + 10 * (base + 4) + (base + 2) + (base + 3) + (base + 6);
        totals.default_conditional_callable_fields += (i % 2 === 0 ? seed + 2 * i : seed)
          + (i % 3 === 0 ? 2 * i : i + 1);
        totals.generic_function_record_pipeline += (i + seed) % 1009;
      }
      for (const id of callableFoundationIds) {
        assert.equal(expectedCallableFoundation(id, n, seed), totals[id]);
        assert.equal(expectedChecksum(id, n, seed), totals[id]);
      }
    }
  }
  assert.throws(() => expectedCallableFoundation('unknown', 1, 7), /No callable foundation oracle/);
});
