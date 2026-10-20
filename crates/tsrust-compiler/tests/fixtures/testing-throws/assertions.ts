import assert from 'node:assert/strict';

assert.throws(() => { throw 'expected'; });
let releases = 0;
// Explicit disposal is the Node counterpart; Rust destructor observation is native-only.
assert.throws(() => {
  using resource = { [Symbol.dispose]() { releases++; } };
  throw 'released';
});
assert.strictEqual(releases, 1);
let failed = false;
try { assert.throws(() => {}, undefined, 'normal return'); } catch { failed = true; }
assert.strictEqual(failed, true);
console.log('throws passed');
