# Compiler nesting architecture

Status: active, required before S12. Full nesting acceptance remains pending;
ordinary acyclic dependency measurements are recorded below. Baseline source is
`6fe878c2`; `f26712cf` tracks the
owner's later UI plans without changing compiler behavior.

## Acceptance boundary

Replace call-stack-dependent function and dependency traversal with explicit
worklists. Audit parser, type analysis, emission, cloning and disposal. Preserve
ownership inference, generic lexical scopes, point-sensitive effects, evaluation
order and diagnostic locations. Raising worker stacks or existing depth constants
is not acceptance. Runtime recursion and JSON depth budgets remain separate.

Require deep finite source structures, long ordinary call chains, recursive
ownership/type cases, native execution, normal worker stacks and compilation
scaling evidence. Finish affected sections before validation. Full compiler/LSP
regressions belong at the completed architecture milestone. Keep original failures
and measurements in SQLite and resume affected/unrun checks.

## Initial audit and section boundaries

- `analysis/mod.rs` shares one depth counter among ordinary functions, blocks,
  expressions and callable result discovery. Direct calls, inferred generic
  results and callable values can recursively request another function body.
  The 128-level guards remain until their recursive paths are replaced.
- Function completion also recursively checks uncalled capturing declarations.
  Lexical owner discovery recursively enters ancestor dependencies. Separate
  scheduling from body checking first, retaining depth-first completion order.
- `source_walk.rs` already walks expression and statement trees iteratively,
  but callback parameter/result annotation traversal was recursive. Give that
  traversal explicit continuations and retain first-error visitation order.
- Generic inter-call effect inference already has an explicit finite worklist
  under `generics/call_validation/branch_flow/invocation_worklist`. Preserve its
  context, environment identity and point-sensitive invalidation semantics.
- Parser/type syntax, type substitution, expression hints, block checking,
  error-effect annotation, emission and recursive syntax cloning/disposal still
  require conversion/audit. No successful annotation-only witness establishes
  end-to-end deep-source acceptance.

## First section in progress

Function scheduling now has its own analysis module. Completed owners enqueue
uncalled bodies in reverse order for a depth-first worklist. Lexical ancestor
resolution stores its continuations explicitly. Direct call dependencies inside
body checking remain recursive and are the next major section.

Callable annotation traversal uses an explicit worklist. Its isolated 16,384-level
witness uses the normal test worker stack and deliberately disposes its synthetic
tree iteratively; production tree disposal remains pending. Focused checks must
also preserve nested capture diagnostics, generic scopes, recursive inference,
callback contracts and generated execution. These are architecture changes with
no new built-in APIs; existing IntelliSense signatures remain unchanged.

S12–S21 remain pending. Preserve the S11 lifetime issue and original schema
performance records described in the acceptance handoff. Profile collection,
recursive, pipeline and export costs after this nesting priority.

## Initial section verification (2026-10-13)

Focused verification passes: two annotation worklist unit tests, 75 compiler
integration tests and 17 nested editor intelligence tests. The six compiler suites
are `control_flow`, `generic_default_scope`, `nested_function_captures`,
`nested_function_declarations`, `recursion` and `recursive_callable_results`.
They retain native ownership/evaluation-order checks and existing depth guards;
passing those guards does not satisfy the requested removal of nesting limits.
The new capture fixtures verify formatting/idempotence before checking.

Whole-project formatting changes zero files and zero benchmark fingerprints.
SQLite records `tsrust-nesting-annotation-worklist-final.log`,
`tsrust-nesting-function-scheduling.log`, `tsrust-nesting-editor.log`,
`tsrust-nesting-section-format.log` and `tsrust-nesting-section-evidence.json`
preserve outputs, source hashes and toolchain identity. Each record was restored
byte-exactly in memory before removing the loose copy; SQLite quick_check passes.
The original cold-build unit run and pre-edit formatting record are also retained.
No compilation scaling or runtime timing samples were collected. No public API
changed, so installed consumers remain on the accepted S11 build until the next
accepted public checkpoint. The shared Cargo target remains active for this goal.

## Ordinary dependency section in progress

Source function references are collected with explicit lexical scope frames.
Parameters, block bindings, closure parameters, loop variables and catch bindings
mask global function names. A worklist computes dependency finishing order and
recursive components without recursion. Components retain their first traversal
entry so existing recursive result seeds are not reordered. Capturing functions
wait for their defining typed context. Dynamic specializations continue through
ordinary checking; recursive component bodies still use the prior inference path.

The new normal-stack source witness checks, emits and disposes a 2,048-function
acyclic chain. Separate native witnesses cover 512-function inferred-return,
function-value, owned-string and imported chains, plus a 256-function throw chain
with argument evaluation order and typed catches. The previous 50-function/four-
branch combined-depth rejection now checks successfully. Deep syntax and large
recursive components remain pending; their guards are not raised or removed.

The first test run exposed two incorrect test expectations: the shared native
helper compares the entire contract map, not a selected subset. Explicit contract
assertions now check the intended entry after native execution. The original
failed run is preserved as `tsrust-nesting-dependencies-tests.log` in SQLite.

A live sample of the confirmed long-chain test identified repeated whole-stream
cloning in formatting annotation probes. The formatter now reuses one scratch
stream and journals packed-closer edits, restoring them even after failed probes.
A differential unit witness compares the original parser result and exact token
restoration. Sorted annotation span membership uses binary search. This repair
retains formatting's token-preservation check; the whole fixture corpus verifies
239 lexable fixtures, 84 standalone contracts and idempotence.
The original sample is `tsrust-nesting-dependencies-sample.log` in SQLite.

`benchmarks/compiler/function-dependencies` registers exact generated native and
TypeScript chains at 32, 128, 512 and 2,048 functions. The fixed protocol is two
warmups and eleven fresh-process samples for native checking, native emission and
Node/TypeScript checking. It retains exact source bytes/hashes, compiler identity,
commands and durations. This scope excludes recursive components, deep syntax,
backend compilation and runtime throughput. Registration is not measurement.

## Ordinary dependency focused evidence (2026-10-13)

Focused verification covers 142 compiler checks: the retained 11-test control-flow
run, 126 successful integration checks after the test-expectation repair, three
scope/probe unit tests, one imported-chain native check and one formatter corpus
check. The latter covers 239 fixtures and 84 standalone contracts. Fifteen editor
intelligence tests pass, including the new 512-function inferred hover/signature/
definition witness. These are affected checks, not complete compiler/LSP regressions.

The release build passes. All 12 scaling groups pass premeasurement validation
and complete two warmups plus eleven samples, totaling 132 measured fresh-process
durations. SQLite stores the original record and proof as
`tsrust-nesting-dependency-scaling.json` and
`tsrust-nesting-dependency-scaling-proof.json`; normalized rows use analytics kind
`compilation` and epoch `function-dependencies-v1`, not runtime throughput.
From 512 to 2,048 functions, median native check time grows about 3.10x and emission
about 3.16x for four times as many functions. Small counts are dominated by startup.
No prior comparable scheduler baseline exists. Recursive components, deep syntax
and backend compilation remain unmeasured.

Whole-project formatting changes zero files/fingerprints. Remaining recursive
checking inside components, capture-dependent/dynamically selected calls, parser,
block/expression analysis, types and syntax disposal are not accepted by this
section. Keep the full architecture priority active before S12.

## Consumer validation and record-context repair

Refreshing consumers from `3f7a38e9` exposed a regression in the existing
`fallible-record-callbacks/values.tsr` fixture. Both the installed CLI's project
check and the live editor reported `Expected int, found #RuntimeError.` at
21:89. The original failed refresh, installed hashes and CLI output are retained
in SQLite as `tsrust-nesting-consumer-refresh.json`,
`tsrust-nesting-consumer-build.log` and `tsrust-nesting-installed-check.log`.

Eager scheduling of record-parameter functions ran before their callers had
constructed the callback environments that determine field error contracts.
Such dependencies now retain their original checking point. This is a semantic
ordering constraint, not stackless acceptance for record-parameter call chains;
explicit suspension with the caller's context remains required. All four
existing fallible-record-callback tests pass with this repair. The scalar-chain
measurements retain their original source identity and are not relabeled as
measurements of the repair.

The affected repair batch passes 28 integration checks: four fallible record
callbacks, six function dependencies and eighteen schema pipelines. Whole-project
formatting changes zero files/fingerprints. SQLite retains the diagnostic probe
and repair outputs as `tsrust-nesting-callback-order-probe.log`,
`tsrust-nesting-callback-order-repair.log` and
`tsrust-nesting-callback-order-format.log`.

## Recursive dependency groups in progress

Annotated recursive groups with concrete data results now use explicit group
entry, body and release work items. Calls within an active group read the finite
access summaries already maintained by recursive inference. Bodies are checked
in reverse dependency discovery order, so a forwarding cycle does not need a
whole-program inference round for each edge. Recursive callback membership and
the borrowed-wrapper rejection remain explicit while group members are active.
The scheduler cleans group membership on errors as well as normal completion.

This removes body-call recursion for those groups without increasing any depth
guard or worker stack. A 2,048-function cycle witnesses normal-stack checking,
emission and disposal. A 256-function owned-string cycle witnesses native
execution, consuming access propagation and the original transferred-value
diagnostic span. Editor coverage includes a 512-function cycle. Existing recursive
scalar and borrowed-callback depth-rejection witnesses now expect success.

Inferred results, concrete callable result environments, context-dependent record
parameters and dynamically selected specializations still need further work.
Deep syntax, type operations and production tree disposal remain separate
pending sections. The new `recursive-manifest.json` registers matched generated
native/TypeScript compilation workloads with the existing two-warmup/eleven-sample
protocol; registration and untimed validation are not measurements.

Focused verification passes 94 compiler checks across the retained batches
(`fallible_record_callbacks`, `function_dependencies`, `exception_handling`,
`recursion`, `recursive_callable_results`, `recursive_callbacks` and
`recursive_owned_callbacks`) and 16 editor intelligence checks. Two original
runs stopped at obsolete scalar/callback depth-rejection assertions; those
original outputs are preserved and the affected/unrun suites subsequently pass.
The 32- and 2,048-function recursive sources pass all three untimed benchmark
phases. SQLite records use the `tsrust-nesting-recursive-groups` prefix, including
the exact generated sources and six validated groups in `-untimed.json`.
