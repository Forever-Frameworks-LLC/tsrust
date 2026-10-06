# Gate 139: own-key membership and key enumeration

Status: focused acceptance complete. Full regressions and measurements remain deferred.

## Behavior

`key in value` returns whether a fixed record, tuple or dense array has an own key. Keys are strings, ints or numbers. Inherited properties are excluded; JavaScript also checks inherited properties with `in`. The key and value evaluate once, in that order. Computed keys, including calls, work. Unsupported receiver and key types report `type` errors. Array/tuple length is present; fractional and out-of-bounds positions are absent.

`for (const key in value)` and `for (let key in value)` enumerate a snapshot of `Object.keys(value)`. Keys are borrowed strings; use `clone(key)` to store an owned copy. Neither binding can be reassigned under the existing borrowed-iteration rules. Records retain constructor key order. Arrays and tuples enumerate positions, strings enumerate UTF-16 positions, and numbers/booleans enumerate no keys. Source mutation after collection does not change the snapshot. Named bindings, single statement bodies, break and continue work; destructuring and an existing variable as the iteration binding are unsupported.

See [the guide](../in-operator.md). Ordinary access does not clone. A local binding named `Object` does not change either syntax operation.

## Implementation and repairs

- `syntax.rs` lowers membership at relational precedence into an internal `hasOwn#in` call and key loops into iteration over an internal `keys#in` call. The internal receiver name is unavailable in source and survives project linking and generic-default scope checks.
- `object_keys.rs` checks the key before the receiver for membership and retains key loans while checking the receiver. `emit.rs` stages the key before the receiver. The proposed draft's value-before-key lowering would change observable call order; it was corrected before acceptance, so computed keys need no extra restriction.
- `projections.rs` exposes checked boolean results for membership and Object.hasOwn. The editor declares key bindings as strings and offers `in` completion. The grammar includes operator highlighting. One focused editor protocol test checks hover, definition and completion.
- A long membership chain initially overflowed the normal test-worker stack. Membership lowering now applies the same expression-depth guard as other binary operations. The retained red witness aborts; the same 200-operation witness now reports `unsupported` normally. No worker stack increase was used.

## Draft corrections

The draft's guessed rejection cases were corrected: fractional number keys are accepted, primitive key-loop receivers follow Object.keys, and reassignment of a borrowed loop key reports `borrowed-element`. Executable fixtures explicitly clone a key when storing it. Applying an allocating string operation to a key works; reassigning the borrowed key does not. Native inherited-name exclusion is documented rather than claimed as Node parity.

## Evidence

- Snapshots: `archive/in-operator-start`, `archive/in-operator-benchmark-start`, `archive/in-operator-consumers-start`, `archive/in-operator-records-start`. Source snapshots preserve directory paths, including same-named compiler/editor files.
- `target/editor/in-operator-red.log`: initial compiler failures. `in-operator-editor-red.log` and `in-operator-shadow-red.log`: editor inference and local Object binding failures before repairs.
- `in-operator-final-native.log`: membership3, Object.hasOwn3, loops14, higher-order function values6, type syntax8 and formatting13 pass. `in-operator-edge-native.log` adds tuple, snapshot mutation and static loan checks. `in-operator-depth-fixed.log` passes final membership4, higher-order6 and type syntax8. Six compiler suites account for 48 final focused checks across preserved passes and supplements.
- `in-operator-final-editor-completion.log`: focused editor protocol check passes. This gate did not run the complete editor suite.
- `in-operator-node.json`, `in-operator-guide-node.json`, `in-operator-guide-final-native.log`: independent Node fixture/reference and guide execution pass with assertions and empty stdout.
- Final scratch debug/compiler/LSP and release builds pass; all five consumers were atomically refreshed before execution checks. The final LSP build includes completion.
- `in-operator-harness.log`: 361/361 checks pass. Subsequent source changes only add the parser depth guard and editor completion; affected native/editor and registration supplements pass.
- Matched `in_operator` workload: eight complete observed cells, independent formula oracle, source contracts, registration, ABI-compatible string[][] result, current source hashes, runtime/profiling/report hooks and manifest. One complete scenario is the operation denominator; no timing was performed.
- Initial untimed verification: `benchmarks/runtime/results/2026-10-05T13-16-13-239Z-verify.json`. Final refreshed-source verification is recorded in `target/editor/in-operator-final-verify.log`.

Coverage uses the existing `object.in` and `control.forIn` identities. No full compiler/LSP regression, timing or SQLite measurement was performed.
