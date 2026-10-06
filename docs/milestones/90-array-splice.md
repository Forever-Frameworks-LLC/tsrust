# Milestone 90: Array.splice

Status: implementation, executable documentation, combined regression and 8 matched comparisons accepted at checkpoint 91.

`splice(start?: int, deleteCount?: int, ...items: Element[]): Element[]` removes a range, inserts same-type elements in argument order and returns an independently mutable array of removed elements. Zero arguments leave the receiver unchanged and return an empty array using Read access. A start alone removes the tail; explicit zero or negative deletion removes nothing. Calls with arguments require mutable access.

Signed starts count from the end and clamp safely, including i64 extremes. Receiver and arguments evaluate once from left to right before the outer commit; normalization uses the post-argument receiver length. Both resulting receiver and removed-output lengths must fit native int and ordinary Rust capacity limits. Argument failures do not commit the outer splice, while prior argument effects remain. Owned elements transfer without implicit cloning; borrowed indexed owners require explicit clone. Existing dense homogeneous scalar, nested-array and record families are supported. Coercion, sparse/mixed arrays, nullable elements and explicit undefined integer arguments remain unsupported.

[Accepted implementation evidence](../../benchmarks/runtime/diagnostics/2026-10-03T16-42-24-000Z-array-splice-implementation.json) binds four retained compiler groups, the source compatibility supplement and one first-pass editor group. Six exact catalog examples produce nine byte-identical output lines at O0/O3, covering omitted versus zero deletion, signed extremes, insertion order and independent owned results. Original failed compiler fixture attempts remain preserved.

Eight matched comparisons are measured at checkpoint 91. Their receiver setup, argument evaluation, mutation and retained output costs are included. SQLite is canonical analytics, with Markdown as an optional export.

[Completed checkpoint 91](../../benchmarks/runtime/diagnostics/2026-10-03T18-25-41-000Z-checkpoint91-completed-proof.json) accepts 890 distinct checks without a full restart, 68 matched native/compiled-Node comparisons and 204 fully disposed profiles. Native is faster on 34/68; there is no universal speed claim. SQLite preserves exact original records and restoration. Across history, 622 workloads are measured among 623 registered sources; the separate peer path workload remains unmeasured.
