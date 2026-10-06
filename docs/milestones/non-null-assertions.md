# Postfix non-null assertions

## Plan and gates

1. Parse postfix `!`, distinguish it from prefix logical negation, allow continued calls/fields/indexes, and preserve the expression-depth limit. Complete.
2. Check a nullable operand once and derive its present payload. Reject absence-only/void sources; retain transfer and borrow checks. Lower selection through the existing tagged-union projection machinery. Complete.
3. Give a failed assertion a clear native message and source-byte range. Retain temporary owners through borrowed use without reordering operand evaluation. Preserve named storage selection for nullable array mutation/indexing. Complete.
4. Publish checked result types for editor hover and provide a conservative source fallback for unfinished chains. Document the runtime contract and verify catalog examples. Complete.
5. Run affected compiler, ownership/allocation, formatting, depth and editor checks; rebuild both binaries. Complete with the evidence below. Combined compiler regression/timing remains at its scheduled checkpoint, separately from this focused acceptance.

## Focused acceptance

- target/editor/non-null-boundaries.log: three feature tests passed, covering the reported popped string, chains, direct temporary inspection, nullable callback invocation, named nullable array mutation/indexing, nonnullable operands, logical negation, format roundtrip, transfers, readonly views with explicit clone, and absent runtime failure.
- target/editor/non-null-regression.log: the feature tests plus the normal-stack depth test passed. Three existing nullable_narrowing tests passed; the allocation observer initially needed an explicit Union1 type on its Cow-based input after generic union layouts.
- target/editor/non-null-allocation-repaired.log: only that affected allocation observer was repaired/rerun; zero-allocation reads, payload-pointer preservation and unwind disposal passed. Preserve the original regression and its three passed checks. Together these establish all four nullable_narrowing checks.
- target/editor/non-null-editor.log: asserted pop hover is string, method hover works, and both direct and bound unfinished chains offer string completion.
- target/editor/non-null-catalog.log: pop/shift catalog signatures/help remain consistent and all examples, including the new assertion examples, execute with exact expected stdout. Removed a brittle fixed output-line count; exact stdout verification remains.
- target/editor/non-null-focused.log: contextual-arrow regression passed alongside the initial assertion feature checks.
- target/editor/non-null-build.log: compiler and language-server binaries rebuilt.

## Performance and broader work

No timing was performed. Existing registered narrowing_int/narrowing_string/narrowing_array workloads exercise the shared successful tagged projection path and remain relevant checkpoint regression coverage. They do not measure postfix assertion syntax or cold failed assertions independently; no assertion-specific throughput claim is made. Successful extraction reuses that lowering and the allocation observer verifies its read behavior. A separate operator microbenchmark is not a meaningful general runtime comparison because TypeScript erases the assertion whereas Tsrust checks an absent payload.

The separate owned nullable closure-capture failure recorded in nullable-console-inspection.md remains outstanding; this gate does not establish general nullable capture correctness or complete the standard-library roadmap.
