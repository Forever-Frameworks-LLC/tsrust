# Gate 129: explicit rest and object-spread copies

Status: focused compiler/native, editor, documentation, current consumer rebuild and matched benchmark acceptance complete. Full compiler/LSP regressions and measurements remain deferred.

## Behavior and ownership

Array rest creates an independent array containing the remaining positions. Tuple rest preserves the remaining position types, including an empty tuple. Record rest copies the fields not explicitly selected, using original field names for exclusions and preserving constructor key order. Named rest bindings work in declarations, nested patterns, function/arrow parameters and for...of. A rest name must come last, without a default or trailing comma.

Earlier selections retain their existing access rules. They may borrow or move according to their source. Rest reads only remaining tuple/record fields, so an earlier selected move does not prevent rest. Reading an already transferred remaining field is rejected. Readonly sources can produce independently owned rest results. Ordinary borrowed selections retain their lexical source loans.

Fixed-record object spread copies source fields and merges surrounding ordinary fields. Sources and fields evaluate once, left to right. Later values replace earlier values while keys retain first-insertion order. An annotation checks the final record shape and field types; supported union widening adapts the result. Without an annotation, the final fixed shape is inferred. Empty, inline and nested spread sources work. Copying completes before the next part evaluates, allowing a later part to mutate an otherwise available source without changing the earlier copy.

Rest and record spread explicitly request copying. Ordinary bindings/fields do not gain implicit clones. Native nested owners copy independently, following existing slice/clone semantics; JavaScript shallow rest/spread shares nested references. Object-spread sources must be fixed records. Primitive, array, tuple, string, nullable and union sources are unsupported. Dynamic property names, symbols and nested rest targets are not implemented. Mixed-alternative default sorting remains unsupported by owner decision.

See [the syntax guide](../guides/language/destructuring.md) for examples and diagnostics.

## Focused evidence

- Initial snapshots are retained in `archive/destructuring-rest-start` and `archive/object-spread-start`; consumer snapshots are in `archive/destructuring-rest-consumers-start`. Red tests and intermediate failures remain intact.
- `target/editor/destructuring-rest-final-native.log`: rest/spread23, adjacent destructuring7 and defaults16 pass. Executable checks use std:testing assertions. Cover independent nested-owner copies, retained sources, readonly/owned parameter access, scalar/union values, partial moves, empty results, nested patterns, arrows/loops, overriding values, exact key order, source evaluation and later source mutation.
- `target/editor/object-spread-focused.log`: adjacent higher-order function values6, type syntax8, Object.keys6 and Object.values/entries4 pass. Later repairs affect the dedicated spread helper/emitter and are covered by the final23 checks.
- Focused defects repaired: whole-tuple reads after an earlier transfer; scalar rest emission; empty spread-source order emission; premature annotation checking of overridden fields; result union adaptation; inferred spread results missing editor metadata.
- `target/editor/destructuring-rest-final-editor.log`: array/tuple/record-rest type, member and definition checks pass, alongside an existing rest-parameter documentation check. `object-spread-editor-current.log` verifies inferred spread field hover/completion and structural type display without internal names. Record-rest completion excludes selected fields. Checked synthesized field metadata supplies real remaining field types.
- Independent Node assertion evidence is retained in `destructuring-rest-node.json`, `destructuring-rest-tuple-node.json`, `destructuring-rest-record-node.json`, `object-spread-node.json` and `object-spread-expanded-node.json`. Native nested-owner independence is tested separately from JavaScript shallow copying.
- All three new guide examples pass current native execution and independent Node assertions with empty stdout in `destructuring-rest-guide-verification.json`.
- Debug/compiler/LSP/editor and release consumers rebuild successfully in `destructuring-rest-final-debug.log` and `destructuring-rest-final-release.log`; debug/editor consumers are refreshed.

## Matched workload readiness

`destructuring_rest` retains eleven cells per iteration across array/tuple/record rest and object spread, including independent array lengths and constructor key order. Its counterpart uses actual TypeScript syntax and native operations. The independent oracle calculates expected values and key strings without executing the tested syntax. Scalar/string source fields avoid nested-reference-sharing differences in the matched scenario.

The manifest binds current source hashes and a whole-scenario denominator. Guards retain actual syntax and every observed cell; registration checks reject altered behavior even after recomputing a source hash. Runtime and allocation-profile preparation enforce guards and fingerprint the oracle/contract; report categorization and the existing complete-shape ABI/observer are ready.

`target/editor/destructuring-rest-registration.log` passes both focused registration/oracle checks. `destructuring-rest-harness.log` passes341/341. Five current-release native/Node-after-tsc cases pass untimed in `benchmarks/runtime/results/2026-10-05T11-01-33-461Z-verify.json` and `target/editor/destructuring-rest-verify.log`.

Implementation acceptance is separate from pending value-family measurements. No timing, allocation profiling or complete compiler/LSP regression is claimed. SQLite measurements remain unchanged. Next gate is130.

Coverage supplement: the status updater initially rejected an unregistered rest-syntax row. Its taxonomy now registers syntax.destructuringRest and uses the existing syntax.spreadObject identity for object spread. The repaired update passes in destructuring-rest-coverage-current.log; final metadata-only registration checks pass in destructuring-rest-registration-current.log. No workload source or compiler behavior changed after the five-case verification.
