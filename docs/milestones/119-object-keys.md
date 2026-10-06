# Gate 119: Object.keys

Status: focused compiler/native, editor, documentation and benchmark acceptance complete. Full regressions and timing remain deferred.

## Behavior and scope

Object.keys(value) returns an independent string array of own enumerable names. Fixed source records use the field order of each value's constructor literal, which may differ from the type declaration. Replacing a field or explicitly cloning a record preserves its order. Data arrays and fixed tuples return decimal position names in ascending order. Strings return decimal UTF-16 code-unit positions; supplementary characters contribute two names. Numbers and booleans return an empty array. The argument is borrowed, evaluated once and unchanged; changing returned names cannot change the source. Exactly one argument is required.

Null, undefined, nullable receivers and functions are unsupported. Source records have fixed declared fields, with no dynamic property additions/deletions, prototypes, symbol keys or numeric/quoted field declarations. The accepted supported-value behavior matches Node Object.keys without iterator or snapshot adaptations.

## Implementation

A dedicated analyzer helper validates arity and supported receiver types without adding large temporary state to recursive dispatch. Checked result facts and editor inference expose string[]. The Object catalog entry includes observable behavior, errors, limitations, ownership, complete signature help and two executable examples.

Native emission discovers enumerated record identities across checked functions, closures and specialized closure methods. Only those record types receive a fixed constructor-order index array. Source literals initialize it in their actual field order; ordinary clones, fixed-record conversion and iterative recursive graph clones preserve it. Tuples retain their positional storage. Unobserved record types retain their prior layouts. Enumeration borrows fields rather than copying record contents. String keys use UTF-16 counts even though ordinary native strings hold valid Unicode text.

## Evidence

- object-keys-red.log retains the initial failing compiler witnesses. Six focused compiler tests pass in target/editor/object-keys-generic-supplement.log. They cover differing constructor orders, independent keys, receiver evaluation, primitive/array/tuple/UTF-16 values, arity/nullable diagnostics, fixed and recursive copies, generic record/function contexts, owned field restoration and absent metadata for unobserved types.
- object-keys-compiler-accepted.log records adjacent Array.entries (9), nested_records (14) and tuple_types (11) passes. After extracting the dedicated analyzer helper, object-keys-helper-accepted.log verifies keys and the depth-bearing tuple suite again.
- tools/object-keys/verify-node.mjs independently transpiles four compiler fixtures and checks exact outputs on Node v24.11.1 using Object.keys directly. Only explicit clone() is adapted to structuredClone. object-keys-node-fixtures2.log retains the results.
- object-keys-editor-red.log preserves the missing editor entry. object-keys-editor-focused.log records two passing editor checks: hover/completion/signature/result inference and native execution of both documentation examples.
- Debug compiler/LSP consumers in both debug directories and the release compiler are rebuilt and refreshed; final build logs are object-keys-formatted-debug-build.log and object-keys-formatted-release-build.log.
- object_keys_order has matched authored sources, independent formula observations, current fingerprints, AST/source/denominator guards, natural ABI/driver support, run/profile oracle input hashes and report registration. object-keys-registration.log records two passing registration/oracle checks.
- object-keys-workload-verify2.log and benchmarks/runtime/results/2026-10-04T21-11-41-967Z-verify.json record five untimed cases on native and pinned Node after tsc, with exact retained observations and zero execution samples. The earlier launch is rejected by the Node version guard and retained in object-keys-workload-verify.log.
- object-keys-harness.log records 319 of 320 checks passing; the missing Other report registration is repaired. All 18 affected report/registration checks pass in object-keys-harness-supplement.log, preserving completed unrelated checks. The initial npm login shell selects Node v26.3.0; the native/Node benchmark, fixture and affected supplement use v24.11.1.

## Pending measurements

Enumerated records carry constructor-order indices; unobserved record layouts remain unchanged. Key results allocate under the existing owning string-array rules. These costs are unmeasured. No full compiler/LSP regression, timing or allocation profile is run. SQLite measurement records are unchanged. Exact baseline sources, documentation and consumer binaries remain in archive/object-keys-start.

Next dependency: heterogeneous data values for Object.values and Object.entries.
