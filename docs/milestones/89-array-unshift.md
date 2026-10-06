# Milestone 89: Array.unshift

Status: implementation, executable documentation, combined regression and 8 matched comparisons accepted at checkpoint 91.

`unshift(...values: Element[]): int` prepends exactly matching elements in argument order, preserving previous element order, and returns the new signed native int length. Zero arguments return the existing length without changing contents and require only Read access. Nonzero insertion requires mutable access.

The receiver evaluates once, followed by every argument once from left to right before the outer prepend commits. Permitted argument effects may change the receiver first; insertion uses its resulting state. Resulting length must fit int and ordinary Rust capacity limits; allocation can fail. Evaluation failures do not commit the outer prepend, while completed argument effects retain their behavior.

Existing dense homogeneous scalar, nested-array and record families are supported. Arrays used as elements are not flattened. Coercion, mixed types, sparse arrays and nullable elements remain unsupported. Nullable receivers require checked guards. Owned arguments transfer without cloning; borrowed indexed owners require explicit clone, and incoming/parent loans prohibit conflicts. The receiver remains reusable; zero-argument Read preserves fixed and scoped read-only views.

[Accepted evidence](../../benchmarks/runtime/diagnostics/2026-10-03T16-15-35-000Z-array-unshift-implementation.json) retains four unique compiler groups, narrow fixture retries, and one first-pass editor group. Five exact catalog snippets produce six complete stdout lines at O0/O3 with empty stderr. Editor checks cover variadic signatures, rest help, completion, inferred int results, concrete type families, guards and zero-argument scoped views; native push still returns void. Accepted staged bytes were guarded into main without repeating the passed check.

Eight source pairs pass 296 cases, 64 hands, 128 negative guards, 24 semantic mutants and five order lines. Accepted rates include setup, arguments, insertion and retained observations. SQLite is canonical analytics; Markdown is an optional historical export. No full regression or timing suite repeats in this gate.

[Completed checkpoint 91](../../benchmarks/runtime/diagnostics/2026-10-03T18-25-41-000Z-checkpoint91-completed-proof.json) accepts 890 distinct checks without a full restart, 68 matched native/compiled-Node comparisons and 204 fully disposed profiles. Native is faster on 34/68; there is no universal speed claim. SQLite preserves exact original records and restoration. Across history, 622 workloads are measured among 623 registered sources; the separate peer path workload remains unmeasured.
