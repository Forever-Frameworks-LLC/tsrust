# Milestone 66: boolean logical assignment

Status: complete. Both strict-boolean forms, 621 distinct Rust groups, required checks, two matched rows and six fully disposed profiles are accepted.

## Scope

Implement strict-boolean `&&=` and `||=` as value-producing expressions in every supported expression context, statements and classic-for clauses. Both operands must be boolean. Assignment has low precedence and associates right. Existing target, capture, parameter, iteration, partial-transfer and enclosing loan rules apply.

Select the target and indices once, check bounds, then copy its old value. `&&=` evaluates the RHS only when the old value is true; `||=` only when false. The skipped branch returns the old value. The executed branch evaluates the RHS once, commits after success and returns the committed value. RHS failure preserves its own effects without committing the outer assignment. Selected parent loans remain protected across the RHS.

Join availability between skipped and executed branches. Conditional transfers remain possibly transferred; conditional restoration cannot restore storage on the skipped path. Preserve unconditional target-selection effects. Use standard Rust boolean locals and conditional blocks, with no runtime container.

## Acceptance

Four focused O0/O3 groups pass truth tables, returned values, precedence, expression contexts, source order, short-circuit effects, captures and storage. They also cover conditional ownership joins, inherited loans, bounds before RHS, inline-to-heap growth, target reacquisition, RHS failure without outer commit, zero scalar allocations and owned disposal on success/unwind.

The sole full compiler invocation stopped at the unchanged deep-array/index guard. The first outlined parser repair still failed and is retained. Phase probes isolated parsing of the 200-index rejection; debug assembly identified a large Result temporary in the recursive binary frame. Pure expression construction reduced that frame below the accepted parent without changing AST representation, depth limits, the witness or normal thread stacks. The exact affected group passed; earlier passing groups were retained and unfinished groups completed by targeted continuation. All 621 distinct Rust groups, doctests, format, all-target Clippy, debug/release builds and fresh identical CLI emissions pass. The saved Gate65 executable rejects the new forms. No full restart is claimed.

Two TypeScript workloads pass 140 complete cases, fourteen mutation negatives, eight independent hand cells and two focused Node groups. Two auxiliary TypeScript programs cover hand values and source order. Native O0/O3 preflight passes ten builds, twenty complete outputs, four untimed disposal cases and two order executions. Both timed workloads exercise taken and skipped branches with dynamic booleans. Each operation retains the actual expression result and final target as two 0/1 int-array leaves. Dynamic selection, boolean-to-integer branch conversion and complete output construction/retention are timed; independent observation is outside timing. JavaScript uses its real boolean logical assignment operators on this strict common domain.

The accepted execution-only run contains 48 process cells and 1,488 timed batches, publishing sixteen min/average/max rate cells across eight runtimes. Native leads compiled Node on 2/2 rows and the fastest scripting runtime on 2/2. Average-rate ratios span 2.627–2.840× versus Node and 2.555–2.602× versus the fastest scripting runtime. All six native profiles dispose output and final owners to zero. `boolean_logical_and_assign` averages 16 retained bytes and 31.875 cumulative requested bytes per complete operation; its 2,048-operation batch averages 1 output allocations and 7 reallocations. `boolean_logical_or_assign` averages 16 retained bytes and 31.875 cumulative requested bytes per complete operation; its 2,048-operation batch averages 1 output allocations and 7 reallocations. These are retained output costs; focused scalar assignments allocate no heap storage. Raw counters remain retained alongside per-operation min/average/max across three samples. Input adaptation is separate; peak means the retained-batch peak divided by operations, not an isolated-call peak.

The grouped report contains 407 individually dated rows. Earlier 405 measured sections, nine primitive memory tables and auxiliary baselines remain byte-exact and were not retimed. No combined 407-row run is claimed. Actual command and repair histories remain bound by the proofs.

[Compiler proof](../../benchmarks/runtime/diagnostics/2026-10-02T19-50-44-000Z-logical-assignment-compiler-proof.json) · [Source proof](../../benchmarks/runtime/diagnostics/2026-10-02T19-21-36-000Z-logical-assignment-source-validation.json) · [Runtime/resource proof](../../benchmarks/runtime/diagnostics/2026-10-02T19-53-46-000Z-logical-assignment-runtime-proof.json)

## Remaining separate work

`??=` requires null/undefined/nullable types and remains pending. Truthiness/coercion and ordinary or arithmetic compound assignment values remain separate capabilities. Owned assignment results require an explicit ownership policy.
