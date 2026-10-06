# Logical value selection acceptance

Implemented operand-valued `||` and `&&` with type joining, truthiness, lazy right-hand evaluation, nullable narrowing and ownership checks. Analysis and native lowering live in dedicated logical-value helpers to preserve normal compiler stack limits. Result metadata supports hover and chained method documentation. Array pop/shift catalog entries include verified fallback examples.

Acceptance checks cover the original shift/string fallback, empty strings, numbers including NaN, nullable values, evaluation traces, existing boolean guards, unsupported result diagnostics and source-depth limits. A native counting-allocator witness checks borrowed selection and payload transfers without allocations or pointer changes at optimization levels 0 and 3. Focused editor checks verify string hover, nullable conjunction hover and chained methods.

Evidence is in `target/editor/logical-values-accepted.log`, `logical-values-editor.log`, `logical-values-catalog.log`, and `logical-values-regression.log`. Original failed allocation checks are retained; the acceptance log includes the repaired borrowing witness. Compiler and LSP binaries are rebuilt in target/editor.

The matched `logical_nullable_string_values` workload is registered in the runtime manifest, with an independent exact-result oracle. Nine cases passed native O0/O3 and Node after strict TypeScript compilation, totaling 27 exact-result checks (`target/editor/logical-values-proof/correctness.json`). These are untimed correctness checks. Performance measurements remain pending checkpoint 101; no timing or allocation parity with Node is claimed. SQLite remains the canonical checkpoint measurement store.

Arbitrary mixed payload unions, generalized truthy conditions, and value-returning logical assignments remain outside this change. Preserve the separately tracked nullable owned-closure capture failure and unrelated checkpoint work.
