# Compiler nesting architecture

Status: active, required before S12. No nesting acceptance or new scaling
measurement is claimed. Baseline source is `6fe878c2`; `f26712cf` tracks the
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
