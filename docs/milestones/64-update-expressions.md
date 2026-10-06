# Milestone 64: numeric update expressions

Status: complete. Four focused groups, 613 distinct Rust groups with retained passes and targeted resumption, required checks, fresh CLI evidence, eight matched comparisons and 24 fully disposed native profiles pass.

## Scope

Implement prefix and postfix increment/decrement for int and Number in all
existing expression contexts, including standalone statements and loop clauses.
Mutable names and existing supported record-field/array-index targets use the
same ownership, capture, parameter and iteration restrictions as assignment.
Strict numeric types do not introduce JavaScript coercions.

Targets and index effects evaluate once. Read the old scalar, compute the new
scalar, commit it, and return the old value for postfix or the new value for
prefix, following [ECMAScript update semantics](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-update-expressions).
Use existing checked i64 addition/subtraction for int and ordinary Rust f64
arithmetic for Number. Preserve signed-zero old values and documented NaN
normalization in observers. Failed bounds or arithmetic must not commit a write.
Postfix syntax respects line terminators, including comments.

## Acceptance

All eight forms are accepted: prefix/postfix increment/decrement for int and Number in existing expression contexts, standalone statements and loop clauses. Standard Rust checked i64 addition/subtraction and ordinary f64 arithmetic compute the new scalar. Target/index effects run once; the old value is read, the result computed, then committed. Postfix returns the saved old value directly, preserving signed zero and values whose update rounds back to the same Number. Failed bounds or checked arithmetic do not commit a write. Strict types, enclosing argument and selected-parent loans, captures, parameters, iteration bindings and partial-field availability retain their existing rules. Postfix grammar checks CR, LF, U+2028 and U+2029 in whitespace and comments; grouped expressions use the closing-token gap.

Four focused O0/O3 groups cover IEEE values, expression contexts and short-circuit effects, flat/nested storage, once-selected indices, mutable/copied captures, loop updates, grammar, diagnostics, checked traps, zero-allocation scalar updates and normal/unwind owner disposal. Failed fixture attempts remain preserved; accepted group bodies were retained. One full regression invocation stopped at the existing array-nesting witness. Moving postfix wrapping to the completed postfix-chain branch restores the primary parser frame from 6,208 to 5,040 bytes without changing AST representation, depth limits, test stack or witness. The exact affected group passes; a selected-target resumption completes unfinished targets without repeating accepted groups, followed by one doctest command. All 613 distinct groups, format, all-target Clippy, debug/release builds and fresh identical CLI emissions pass. The saved Gate63 executable rejects the new syntax. No full restart was performed.

Eight rows invoke actual ++/-- directly and retain both expression result and committed target in complete int[] or number[] outputs. Dynamic operand selection, target initialization, arithmetic, output construction and retention remain timed. Independent full observation runs outside timing. Int timing operands are bounded positive exact integers; checked i64 endpoint overflow is separate native-only evidence. Number operands include exact fractions, signed zeros and the 2^53 rounding boundary. IEEE observers preserve exact non-NaN bits and canonicalize NaN tokens; native old-payload checks remain focused semantics. Eight actual TypeScript programs pass 560 complete cases, 34 mutation/typing negatives, 80 independent hand cells, source order and two Node groups. Native O0/O3 preflight passes 34 builds, 80 complete outputs, sixteen untimed disposal checks and two effect-order executions. Eight additional native-only overflow builds/runs verify all four int update forms fail before output with implicit Rust overflow checks disabled.

The accepted execution-only record contains 192 process cells and 5,952 measured batches, publishing 64 min/average/max rate cells. Twenty-four native profiles preserve raw counters and three-sample per-operation min/average/max summaries. Scalar updates themselves allocate no heap storage in focused tests. Whole workloads retain two scalars per operation: 16 retained requested bytes per operation, one output allocation per 2,048-operation batch, and seven int or ten Number output reallocations per batch. These are output-storage costs, not update allocations. Input adaptation stays separate. Peak is the retained-batch peak divided by operations, not an isolated-call peak. Output and final disposal remain absolute zero checks. Earlier 381 measurement sections, nine primitive memory tables and auxiliary baselines remain byte-exact and were not retimed; the grouped report has 389 separately dated rows.

The initial source-only runtime seal failed because section extraction included following run metadata before its first anchor. The boundary repair preserved measurement tables, compiler/harness sources and raw rates; no measurements were repeated. The original helper and freeze history remain archived.

Native leads Node after tsc on 8/8 rows and the fastest scripting runtime on 8/8. Ratios use actual average rates. Resource columns show three-sample averages per complete operation; the runtime proof retains raw samples and all min/average/max summaries.

| Workload | Native / Node after tsc | Native / fastest scripting | Output allocations / operation | Requested bytes / operation | Retained-batch peak bytes / operation |
| --- | ---: | ---: | ---: | ---: | ---: |
| `update_int_prefix_increment` | 2.133× | 2.096× | 0.000488 | 31.875 | 16.000 |
| `update_int_prefix_decrement` | 2.005× | 2.005× | 0.000488 | 31.875 | 16.000 |
| `update_int_postfix_increment` | 1.903× | 1.818× | 0.000488 | 31.875 | 16.000 |
| `update_int_postfix_decrement` | 2.069× | 1.941× | 0.000488 | 31.875 | 16.000 |
| `update_number_prefix_increment` | 3.056× | 2.804× | 0.000488 | 31.984 | 16.000 |
| `update_number_prefix_decrement` | 3.144× | 2.862× | 0.000488 | 31.984 | 16.000 |
| `update_number_postfix_increment` | 3.098× | 2.567× | 0.000488 | 31.984 | 16.000 |
| `update_number_postfix_decrement` | 3.064× | 2.785× | 0.000488 | 31.984 | 16.000 |

[Compiler proof](../../benchmarks/runtime/diagnostics/2026-10-02T18-03-13-000Z-update-expression-compiler-proof.json) · [Source proof](../../benchmarks/runtime/diagnostics/2026-10-02T17-43-26-000Z-update-expression-source-validation.json) · [Runtime/resource proof](../../benchmarks/runtime/diagnostics/2026-10-02T18-13-02-000Z-update-expression-runtime-proof.json)

## Remaining separate work

Int bitwise/shift/exponent policies, logical assignment, assignment-valued
expressions, coercions and other pending inventory operations remain separate
gates. This gate does not imply completion of all assignment expressions.
