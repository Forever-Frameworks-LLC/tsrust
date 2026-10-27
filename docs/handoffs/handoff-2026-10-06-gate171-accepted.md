# Handoff: gate 171 accepted (nested function captures)

Date: 2026-10-06. Read `AGENTS.md`, `docs/development-workflow.md`,
`docs/plans/nested-function-declarations-plan.md` and
`docs/milestones/171-nested-function-captures.md` (section "Acceptance") first.

## State

Gate 171 is accepted and committed. The working tree holds only the four owner
fixture edits, which stay uncommitted, unstaged and untouched:

- `crates/tsrust-compiler/tests/fixtures/comma-contexts/guards.tsr`
- `crates/tsrust-compiler/tests/fixtures/interfaces/shapes.tsr`
- `crates/tsrust-compiler/tests/fixtures/map-for-each/values.tsr`
- `crates/tsrust-compiler/tests/fixtures/map-group-by/documented.tsr`

Use explicit Git paths. Never `git add -A` or `git commit -a`.

## How captures work

- `local_functions.rs` resolves names, lifts nested declarations and plans captures.
  A local function name outside the callee position gets the suffix
  `local_function_values::VALUE_MARK`.
- `local_function_values.rs` replaces a marked use of a capturing function with the
  arrow `(parameters) => name(parameters)`; other marked uses lose the mark.
- `named_capture_context.rs` gives each capturing function, each concrete copy of a
  callback-taking function and each generic instance a typed capture context. A
  variable reaches only functions lexically inside its owner (`encloses`).
- `analysis.rs`: `declared_named_call` records hidden argument slots in
  `Expr::capture_slots` and checks the borrows; `prepare_uncalled_capturing_functions`
  checks functions that no call reaches; `Analyzer::closure` projects every declared
  variable when the program has capture plans.
- `named_capture_lowering.rs` validates the lowered form; `emit.rs` appends the hidden
  arguments to signatures and calls.

## Next: gate 172, then gate 173

Gate 172 (owned escape). The basic witness passes through the arrow form. Still owed:
multiple factory instances, aggregate storage of nested functions, disposal, callback
failures, recursive group lifetime, a nested function that takes ownership of a
captured variable, editor and guide text, and a matched workload.

Gate 173. `nested_declarations_infer_nonrecursive_result_when_omitted` is the red
witness; do not weaken it. Also: function values of generic functions and of capturing
functions with optional or default parameters, parameter patterns, catch/finally
scopes, deep nesting, and precise fixed-array storage for callers of capturing
functions.

## Checks

```
export CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target CARGO_INCREMENTAL=0
cargo test --offline --locked --no-fail-fast -p tsrust-compiler --lib \
  --test nested_function_captures --test nested_function_declarations \
  --test closures --test for_of --test owned_arrays --test nullish_coalescing \
  --test nullish_assignment --test higher_order_function_values \
  --test number_operators --test type_syntax
cargo test --offline --locked --no-fail-fast -p tsrust-lsp
(cd benchmarks/runtime && npm test)
(cd benchmarks/runtime && node run.mjs --verify --only=nested_function_captures --engines=native,node-tsc)
```

Expected: one failure, the gate 173 witness. Editor 276, harness 398, unit 308.
Node is `/Users/seancotter/.nvm/versions/node/v24.11.1/bin/node`.

New benchmark manifest rows go at the end of the workload list; harness prefix guards
depend on it. Evidence is in SQLite (`benchmarks/records.sqlite`); the concise index
is `target/editor/gate171.log`.
