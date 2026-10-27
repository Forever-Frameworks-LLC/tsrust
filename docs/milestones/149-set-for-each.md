# Gate 149: borrowed Set callbacks

Status: accepted with focused compiler/editor checks, runtime harness and matched
untimed verification. No full regression or timing run was performed.

## Behavior

Set.forEach visits each key once in insertion order, supplying the key, the same
key again and the readonly Set. Callbacks may use zero through three supplied
positions, with ordinary default/rest rules. Empty Sets make no calls. There is
no thisArg argument. The method returns void; callback results are evaluated and
disposed after each invocation.

Scalars copy; owned keys and the receiver borrow readonly. Receiver mutation or
transfer, consuming callbacks, incompatible access/types, incorrect method arity
and moving an owned callback input into another owner are rejected. External
mutable captures are allowed when they do not conflict with source loans.
Callback errors stop traversal and propagate through catch/finally. Catch values
can include structured runtime errors; narrow before string-only operations.
See [the guide](../guides/language/sets.md). No input implicitly clones.

## Repairs

Set traversal reuses the established callback contract machinery with a keyed
argument tuple. Native emission supplies the same key twice and borrows the
receiver. Contextual generic collection and record parameters now use their
checked nominal names rather than reparsing fresh generic applications.

A failing error witness exposed an existing Array.forEach defect: fallible
callback Results were discarded, swallowing throws and failed assertions. A
separate Result-call adapter now propagates closure, specialized closure-method
and abstract callback failures through the existing completion mechanism.
Array.forEach is repaired too; pure callback behavior stays covered by its suite.
A later readonly abstract callback witness also repairs dispatch to ReadCallback
rather than Callback; both Set and Array are checked, including abstract failures.

An additional Array.map probe demonstrates that broader array callback Result
transport still needs repair: its iterator tries collecting Results as plain
values. This is concrete next work, not a claim that the callback family is done.
Borrowed Map iteration and Map.forEach also remain open.

## Evidence

Logs are retained under target/editor/set-for-each-*.log and
target/editor/for-each-*.log.

- Original red witness rejects unsupported Set.forEach. Independent pinned Node
  assertions pass on the final fixture, including early stop and finally cleanup.
- Final set-for-each-readonly-native.log: seven compiler checks pass, plus
  Array.forEach 4/4. They cover named, borrowed abstract and readonly callbacks,
  including readonly abstract failures. Earlier complete-native passes five
  focused checks, covering 0..3
  positions, defaults/rest, discarded owned results, source access and transfers,
  mutable captures, nominal generic records, owned strings, empty input and
  catchable callback errors. The fixture's later finally addition passes directly
  in set-for-each-finally-native.log, independently checked against Node.
- for-each-callback-error-red.log retains the swallowed Array.forEach throw;
  for-each-result-native.log passes the repaired witness and Array.forEach 4/4.
- higher_order_function_values 6/6 and type_syntax 8/8 pass in
  set-for-each-final-native.log before the Result-call repair. Only affected
  traversal checks are rerun after that emitter change.
- set-for-each-editor.log: new editor check 1/1 plus adjacent Set 4/4.
  Typed help, source callback member hover and catalog/fixture synchronization pass.
- All ten Set catalog examples match native and adapted Node outputs in
  set-for-each-documented-native.log and set-for-each-documented-node.log.
- set-for-each-final-harness.log: runtime harness 375/375 on final refreshed consumers.
- set_for_each passes five cases per engine, native and node-tsc, in
  results/2026-10-06T07-22-36-134Z-verify.json. verifiedOnly is true and timing
  arrays are empty. The workload is unmeasured.
- Workload checks before refresh; debug/release builds pass. Five consumers refresh
  atomically, with hashes in set-for-each-final-consumers.log and old bytes preserved in
  archive/set-for-each-consumers-start.

The earlier pre-read-dispatch verification is preserved with exact compiler
input bytes reconstructed and checked against every recorded hash, original
consumer bytes and manifest in archive/set-for-each-before-read-dispatch. Final
acceptance flags change after verification/harness; final registration is checked
separately and the exact prior manifest is preserved in
archive/set-for-each-verification-start.

Matched sources, independent oracle, semantic guards, registration, runtime and
profiling hooks, reporting category and fingerprints are ready. Original source,
catalog, fixtures and metadata remain in archive/set-for-each-*-start. No Git.
