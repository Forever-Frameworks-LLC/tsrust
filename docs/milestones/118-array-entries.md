# Gate 118: Array.entries

Status: focused compiler/native, editor, documentation and benchmark acceptance complete. Full regressions and timing remain deferred.

## Behavior and scope

Calling entries() on a supported data array returns an ordinary eager array of fixed [int,T] tuples. Each first position is the zero-based index; each second position has the source element type. Empty arrays return an empty result. There are no arguments. Unsupported receivers, arguments and callable environments receive diagnostics. Source data remains unchanged. Owned strings, records, nested arrays and tuples are copied independently, including owned descendants. Results support for...of, Array.from and ordinary array operations under the existing ownership rules. Positional tuple access requires constant integer indices.

The native convention matches keys()/values(): an owning snapshot rather than a live JavaScript iterator. It does not expose next() or Symbol.iterator. Node fixtures and matched TypeScript workloads explicitly materialize and copy their iterator values to compare the accepted snapshot contract. This is a native variant, not an unadapted iterator-parity claim.

## Implementation and repairs

A dedicated analyzer helper materializes structural [int,T] tuple records and checks cloneability. Native emission evaluates the receiver once, enumerates its data and constructs independent pairs. Contextual empty literals retain the value-column type. Fixed storage and literal strings retain their existing storage paths. Checked facts and editor inference expose positional result types. The Array catalog entry supplies complete behavior, limitations, ownership, signature help and two executable examples.

Acceptance exposed tuple indexing after generic call resolution. Checked indexing now selects the actual tuple type and lowers the selection without evaluating the receiver again. Borrowed owned positions retain transfer restrictions. Record-array copying uses its actual record representation. Editor array detection parses nullable tuple positions so an internal union does not hide array methods; whole nullable arrays retain their existing restrictions.

## Evidence

- Initial failing compiler witnesses are retained in target/editor/array-entries-red.log and subsequent context/generic repair logs. Nine compiler tests pass across array-entries-compiler-accepted.log (eight) and array-entries-recursive2.log (one supplement). Coverage includes ordering, independent descendants, arity/types, contextual empty arrays, literal/fixed storage, receiver evaluation, generic results, recursive calls and ownership diagnostics. Adjacent keys/values (2) and tuple_types (11) pass in the accepted compiler log.
- tools/array-entries/verify-node.mjs independently transpiles and executes seven compiler fixtures on Node v24.11.1. array-entries-node-fixtures2.log records matching outputs with explicit eager independent entry snapshots.
- array-entries-editor-accepted2.log records three passing editor tests: result inference/signatures/hover/completion, native execution of both catalog examples, and nullable tuple-array members. The adjacent nullable-array removal check is retained in array-entries-nullable-adjacent.log.
- Debug compiler/LSP consumers are refreshed in both debug locations; the release compiler is rebuilt. Build evidence is retained in array-entries-debug-build.log, array-entries-release-build.log and array-entries-final-debug-build.log.
- array_entries_string and array_entries_records have authored TypeScript counterparts, independent formula oracles, current source fingerprints, AST contracts, ABI/result observations, run/profile hooks and manifest/report registrations. array-entries-registration.log records 18 passing affected registration/report checks.
- array-entries-workload-verify.log and benchmarks/runtime/results/2026-10-04T20-46-45-173Z-verify.json record five untimed cases per workload on native and Node after tsc, with exact retained observations. No execution samples are collected.
- array-entries-harness.log records 318/318 passing harness tests. Completed checks are retained; the later editor-only repair does not change compiler/workload fingerprints.

## Pending measurements

Pair materialization and independent owned data copies allocate under existing aggregate rules. These costs are unmeasured. No full compiler/LSP regression, timing or allocation profile is run. SQLite measurement records are unchanged. Exact baseline sources, documentation and consumer binaries remain in archive/array-entries-start.

Next gate: investigate record enumeration order and heterogeneous value representation for Object.keys/values/entries.
