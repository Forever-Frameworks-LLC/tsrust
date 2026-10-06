# Gate 117: fixed positional tuple types

Status: focused compiler/native, editor, documentation and benchmark acceptance complete. Full regressions and timing remain deferred.

## Behavior and scope

An inline annotation such as `[int,string]` describes a fixed number of independently typed positions. An array literal becomes a tuple when its annotation, parameter or return context requires that shape. Positions may contain primitives, named records, data arrays, nested tuples or supported nullable data types. `[]` describes an empty tuple. Unannotated array literals retain their existing homogeneous-array inference.

Constant signed integer indices select the declared positional type. Negative or out-of-range positions produce a bounds diagnostic. Values of the position's declared type may replace it. Length reports the fixed number of positions, including zero for an empty tuple, and cannot be assigned. Tuple arrays use ordinary array storage and support tuple-returning map callbacks. Tuple values are recognized by Array.isArray, display as arrays in console output and use comma-separated recursive array text through String() and enclosing array join/toLocaleString operations. Null and undefined positions are empty text fields.

Tuple owners follow the existing aggregate ownership rules. Copy positions are copied; owned positions may transfer from direct named storage. Restore a transferred position before observing the whole tuple. Indexed tuple-array elements remain borrowed; clone() produces independent owned data. ReadOnly and Mutable parameter access apply to tuple positions. Temporary receivers preserve their ordinary evaluation and disposal rules.

Tuple identities are structural across equivalent positional types, grouped annotations, generic function inference/specialization, generic record fields and linked project imports. Nominal record members retain their original declaration identity inside a tuple; project/editor facts expose a positional tuple, rather than a synthetic nominal record.

This foundation does not add named tuple aliases, labelled/optional/rest positions, `readonly [...]` syntax, callable positions, computed positional indices or variable-length tuple mutation/iteration methods. Tuples retain fixed owning storage. Flattening into tuple positions and flatMap callbacks returning tuples are rejected explicitly; the current homogeneous-array result model cannot represent heterogeneous flattened positions. A flat depth that only removes ordinary array dimensions remains supported. These restrictions remain separate work, not accepted JavaScript parity claims.

## Implementation

The shared annotation parser and lexical traversals retain tuple positions through generic substitution, defaults, specialization and module linking. Concrete positional products reuse the checked record layout, native storage and ownership machinery internally. Synthetic symbol names cannot collide with source record identifiers. Resolved positional types determine the canonical identity. Literal and constant-index lowering happen in a dedicated helper, before ordinary aggregate checking; shared recursive dispatch remains small.

Native console inspection, array text and nullable observations distinguish positional products from source records. Nullable positions use their tags when producing array text. The checked type-fact and project-fact paths preserve positional return/field types for source editor inference. Tuple length has a complete catalog entry and a verified native example, with hover, completion and signature-help coverage. Array.isArray and String conversion documentation explicitly includes tuple behavior.

## Evidence

- `target/editor/tuple-types-red.log`: both original witnesses fail before implementation at tuple type parsing. Expected basic console output is independently preserved in `tuple-types-node.log`.
- Eleven tuple compiler tests have passing evidence across `tuple-types-compiler-focused.log`, `tuple-types-compiler-supplement.log`, `tuple-types-project2.log`, `tuple-types-nullable-red.log` and `tuple-types-depth.log`. They cover positional typing and length, bounds, nested/empty tuples, tuple arrays, owned field transfer/restoration, mutation, cloning, temporary receivers, nullable positions/results, structural generic identities, imported nominal member facts, formatter idempotence and type-depth guards. The nullable-result witness already passed its initial run; its filename is retained rather than implying a discovered defect.
- `tuple-types-flatten-red.log` exposes the previously unchecked flattening result; the focused compiler run verifies the explicit guards and the valid flat(0) case. Existing array_join (4), array_flat (4) and nested_records (14) suites pass. Focused generic_records, generic_functions and type_substitution unit groups each pass three tests.
- `tools/tuple-types/verify-node.mjs` independently transpiles and executes six compiler fixtures on Node v24.11.1, using structuredClone only for the fixture's explicit clone operation. `tuple-types-node-fixtures3.log` retains matching output.
- Three focused editor tests pass on the final catalog in `tuple-types-editor-accepted.log`, with the earlier signature/protocol runs retained: positional hover/inference, length hover/completion, parameter access and positional signature help, and native execution of the catalog example. The missing positional editor inference is retained in `tuple-types-editor-red.log` and repaired.
- Debug compiler/LSP and release compiler rebuilt; workspace/editor consumer binaries refreshed. The matched `tuple_types_values` pair observes construction, explicit text copy, positional mutation, tuple-array construction, tuple-returning map and eight retained scalar cells. Fingerprint/AST guards, independent result oracle, existing ABI/result observers, driver and profile hooks, and report registration are complete.
- `tuple-types-workload-verify2.log` and `benchmarks/runtime/results/2026-10-04T20-14-21-278Z-verify.json`: five untimed cases pass on native and Node after tsc. The initial preparation required the established workload ABI spelling; sources and hashes are repaired, with the original failed run retained.
- `tuple-types-harness.log`: 315 of 316 tests initially pass. The tuple report category needs its matching registration expectation. After that repair, all 18 affected report/tuple registration tests pass in `tuple-types-harness-supplement.log`; completed unrelated passes are retained.

## Pending measurements

Tuple fields use fixed native aggregate storage; tuple arrays and mapped results use the existing owning array machinery. Explicit text copies and mapped positional results allocate according to existing string/array ownership rules. These costs are unmeasured; no allocation equivalence claim is made. No full compiler/LSP regression, timing or allocation profile is run. SQLite measurement records are unchanged. Exact prior sources, documentation and consumer binaries remain in `archive/tuple-types-start`.

Next gate: Array.entries, using typed `[int,T]` results and the accepted tuple foundation.
