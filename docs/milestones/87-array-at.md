# Milestone 87: Array.at and scoped optional views

Status: implementation, executable documentation, combined regression and 8 matched comparisons accepted at checkpoint 91.

## Behavior

`at(index: int): Element | undefined` takes exactly one signed 64-bit integer, with no default. Zero selects the first element; a negative index counts backward from the current end. Out-of-range indices, including either i64 extreme, return undefined without a bounds error. The receiver and index evaluate once in order; lookup does not change length or elements. Number indices, implicit coercion, fractional-index truncation and extra arguments are unsupported.

The supported dense homogeneous scalar, nested-array and record families retain canonical optional result types. Undefined elements collapse the duplicate undefined alternative; null elements return null or undefined. Number results retain their original IEEE bits. Sparse arrays, heterogeneous or nullable element arrays and arbitrary borrowed annotations remain separate work.

## Storage and ownership

Copyable elements return copied optional values. Reusable named owned receivers provide scoped read-only optional views of strings, nested arrays and records. Views support aliases, whole-value observation, guarded payload reads and Read nullable arguments. Conflicting receiver mutation or moves are prohibited while a view is live; its payload cannot be consumed, mutated, returned or captured by an escaping closure. Explicit `clone(view)` creates an independent owning optional value. A temporary owned receiver can transfer its selected payload without an implicit clone.

Nullable receivers and optional results require compiler-proved strict null/undefined guards before payload access. Editor views use the existing cached compiler projection metadata; there is no separate textual flow checker.

## Focused acceptance

Four compiler groups and affected supplements accept O0/O3 content, dense storage families, canonical types, Number bits, signed index extremes, source order, loan/escape diagnostics, retagging, conditional receivers and normal/unwind disposal. Read-only optional views cover String, nested arrays, records and recursive record arrays through whole-value observers and guarded Read calls. Concurrent native-binding changes are preserved.

One focused editor group verifies hover, signature/parameter help, completion, inferred optional results and checked refinement across nine concrete type witnesses, including a nominal type beginning with `unknown`. Existing indexed receivers, field chains and native int formatting remain covered. The exact three catalog snippets produce five complete output lines at O0/O3 with empty stderr. The initial compiler compatibility build failure and subsequent prose-case fixture failure are retained; only this affected group was rerun.

Eight registered comparisons cover int, Number, boolean, String, nested arrays, records, null and undefined. Independent TypeScript checks accept 296 cases, 64 hand cells, 78 negative guards and eight semantic mutants, with a six-line order fixture. Complete receiver/query setup, lookup, observation encoding and retained output costs belong to the accepted whole-workload measurements. No isolated lookup latency, allocation parity or universal native-speed claim is made. SQLite is the canonical analytics record; Markdown remains an optional historical export.

[Accepted implementation evidence](../../benchmarks/runtime/diagnostics/2026-10-03T15-09-13-000Z-array-at-implementation.json) preserves compiler/editor attempts, repairs, exact generated documentation outputs and the separate actual source-registration record. No full regression or benchmark run is repeated in this gate.

[Completed checkpoint 91](../../benchmarks/runtime/diagnostics/2026-10-03T18-25-41-000Z-checkpoint91-completed-proof.json) accepts 890 distinct checks without a full restart, 68 matched native/compiled-Node comparisons and 204 fully disposed profiles. Native is faster on 34/68; there is no universal speed claim. SQLite preserves exact original records and restoration. Across history, 622 workloads are measured among 623 registered sources; the separate peer path workload remains unmeasured.
