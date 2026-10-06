# Nested function declarations: implementation and acceptance plan

This extends the compiler/editor foundation and the callable support needed by the
standard library. The full feature includes lexical scope, declaration hoisting,
self and mutual recursion, captures, callable values and escape, generic signatures,
default/rest/destructured parameters, diagnostics, formatting and editor discovery.
A capture-free implementation is an intermediate gate, not completion of this plan.

## Owner decision and language contract

On 2026-10-06 the owner selected existing Tsrust capture ownership rules over shared
JavaScript capture cells. Local captured owners are borrowed under current loan
rules; owned escaping environments transfer captures. Scalar captures retain their
existing copy semantics. There are no implicit owned clones, reference-counted
shared capture cells or weakened borrow lifetimes. Mutating a borrowed capture and
moving an escaped owner must remain subject to existing checks.

A declaration name is available throughout its enclosing lexical block. Its body
is analyzed in that lexical scope, independently of the textual declaration point.
A nested declaration shadows an outer name within its block and does not leak out
of it. Same-scope declaration conflicts are diagnosed at the conflicting name.
Explicit braces are required for declaration bodies; browser legacy block-function
leakage is outside this native/module language contract.

Hoisting a function name does not initialize captured let/const data. Calling a
function before a required capture is initialized is a compile-time flow error.
Later capture declarations may be referenced by the function if every reachable
use satisfies initialization and ownership constraints. The compiler must not
create a view of an uninitialized native slot. This needs explicit capture readiness
and initialization placement, not a textual replacement with const plus an arrow.

Self and mutual recursion must have finite native types. Recursive groups must not
embed environments inside themselves or duplicate owned captures. An explicitly
annotated recursive return type can establish the checking boundary; inferred
recursive results must resolve consistently or produce a precise diagnostic.

First-class use and return must preserve identity, callable invocation contracts,
defaults/rest arity and ownership. The existing named-function/closure mechanisms
are reused where they prove these properties; lifting an anonymous static function
must not erase the declaration's lexical scope or original source identity.

References: ECMA-262 BlockDeclarationInstantiation and lexical block declarations:
https://tc39.es/ecma262/2026/multipage/ecmascript-language-statements-and-declarations.html#sec-blockdeclarationinstantiation.
JavaScript reference witnesses validate behavior in the supported overlap. Native
ownership restrictions are deliberate and must be documented directly.

## Compiler seams and isolated modules

- Add a real lexical declaration representation, with source name/header/body,
  declaration span, enclosing scope identity and stable function identity. Keep
  top-level declarations and expression functions distinct. Do not encode local
  declarations as magic names or discard their scope into the global function map.
- Share header parsing for named declarations where practical: generic parameters,
  access/default/rest/destructured parameters and result annotations. A dedicated
  non-inlined parser helper keeps recursive statement frames small.
- A new local_functions module indexes block/function scopes, registers all direct
  declarations before resolving uses, and resolves shadowing/duplicate names. Use
  stable IDs for symbols; hidden emitted names are an implementation detail.
- Separate declaration resolution from checking. Capture-free local declarations
  may use ordinary static function emission with qualified IDs and source maps.
  Their removal from executable statement IR must preserve block scope metadata.
- A local_capture_plan module computes direct/transitive captures and recursive
  strongly connected components. Resolve captures to binding/place identities,
  type/access contracts and initialization facts. Do not treat sibling function
  names as owner payloads that create recursive environment layouts.
- Integrate capture readiness with definite-assignment and flow checks. Materialize
  borrowed environments only after capture initialization is proven; preserve the
  existing lexical loan lifetime. Escaping environments transfer supported owners
  once. Mutually recursive captures need one finite group environment plus callable
  views/static dispatch, not a cycle of Rust values or implicit capture copies.
- Reuse callable traits, stored callable transport and specialization bounds.
  Emission belongs in a small local-function module, with ordinary expression and
  statement dispatch kept thin. No unsafe code or increased stack limits.
- Preserve declaration metadata through modules, generic specialization and checked
  projections. Editor symbols must use the source name and lexical scope, with
  definition/hover/references/signature help and parameters tied to the right body.
  A local name main never becomes the program entry.

Before introducing the AST variant, inventory exhaustive statement visitors in
analysis, exception/control flow, generic specialization, linker, source indexing
and emission. Each must either traverse the local body in its correct function
context or consume the normalized declaration representation. Do not accidentally
count a nested return/break as control flow of its enclosing function.

## Statement-consumer inventory

The initial source inventory finds 18 compiler files referencing StmtKind. These
are integration obligations, not proof that every visitor has been updated:

- syntax.rs owns parsing and registration-return validation. A declaration body
  must create a new function boundary, not contribute returns to its enclosing body.
- source_walk.rs owns iterative source annotation/expression traversal and body
  extents. Visit declaration headers and bodies with their source identity intact.
- project_link.rs rewrites module symbols and local scopes; local declarations
  must survive qualification and imported type/callback resolution.
- generic_functions.rs, generic_records.rs and generic_default_scope.rs specialize
  signatures and resolve parameter initializer scopes. Nested declarations need
  fresh parameter/type-parameter scope and stable specialization identity.
- analysis.rs, callable_dispatch.rs, rest_calls.rs and fixed_storage.rs analyze
  calls, captures and storage. Normalize static local declarations before ordinary
  checking; retain declaration metadata and defer captures to their planned stages.
- completion.rs, definite_assignment.rs, error_effects.rs and exception_state.rs
  analyze completion, initialization and errors. Declaring a function does not run
  its body or propagate its return/throw into the enclosing statement flow.
- emit.rs and exception_contract_emit.rs emit ordinary and exceptional control
  paths. Emit a declaration's function body independently from enclosing execution.
- projections.rs and project_facts.rs expose checked editor facts and source spans.
  Preserve lexical declaration identity, original names and local main semantics.

The declaration header/body parser is now extracted into named_function_syntax.rs
and called by top-level parsing. This is a preparatory module seam; it adds no nested
syntax support and does not satisfy gate 170 acceptance. Its dedicated non-inlined
helper shares generic/access/default/rest/pattern/result parsing for the forthcoming
nested declaration path. No AST variant is introduced in this preparatory chunk.

## Milestone gates

### Gate 170: lexical declaration foundation and capture-free execution

Parse and retain nested declarations; register names before uses; resolve lexical
shadowing and declaration conflicts. Execute capture-free forward calls, self and
mutual recursion, nested blocks and first-class synchronous callbacks. Retain source
spans and editor symbols. Build a matched retained-result benchmark for these cases.
The guide must explicitly say captured declarations remain pending until gate 171;
this is a staging boundary, not the final feature contract.

Accept when the hoisting fixture passes native and pinned Node, static scope/conflict
checks pass, formatting and local declaration editor navigation pass, all eight depth
canaries and the whole editor crate pass, matched benchmark support is complete and
verified untimed, and records/consumers are current. Captured-function red witnesses
remain open and must not be rewritten as accepted unsupported behavior.

### Gate 171: nonescaping captures and initialization flow

Support readonly and mutable captures, declarations referencing later initialized
locals, recursive captured groups and nested capture forwarding. Match existing
arrow/function-expression borrowing and lexical loan lifetimes. Diagnose a call
before capture initialization, conflicting access, transferred inputs and invalid
reentrant mutable sharing with declaration/use locations. Use fresh environments
per loop/block entry. Add focused capture witnesses and matched benchmark support.
Accept after compiler/native, editor/guide/examples, depth/whole-editor checks,
untimed verification, durable records and a source checkpoint.

### Gate 172: escaping and retained nested callables

Return/pass/store nested functions using existing owned callable transport. Transfer
captured owners once, keep scalar copy semantics, preserve once versus reusable
invocation and reject escaping borrowed data. Prove multiple factory instances,
aggregate storage, disposal, callback failures and recursive group lifetime. No
shared JavaScript cells are introduced. Verify capture ownership diagnostics and
native behavior, then complete editor and matched benchmark acceptance as above.

### Gate 173: full signature and context coverage

Finish local generics, inference boundaries, defaults, rest, parameter patterns,
abstract callbacks, imported nominal types, catch/finally scopes and deep nesting.
Ensure templates preserve local symbol identity, recursion bounds and source spans.
Named function expressions remain a separate feature; do not accidentally accept
or rewrite them while parsing declarations. Complete documentation, compiler and
editor coverage, matched workloads and records for all accepted paths.

### Named-declaration family milestone

Audit this plan requirement by requirement against current sources and direct
execution. Then run the meaningful complete compiler/LSP regressions and measure
accumulated affected workloads under the retained protocols. Preserve completed
checks, compare compatible native/Node signals, investigate actual regressions and
optimize. Individual gate numbers do not trigger this heavy run. Full plan completion
requires every stage, including captures and escape, rather than only gate 170.

## Current state and execution discipline

Gate 170 is accepted for lexical capture-free declarations, native execution,
editor navigation and formatting, with a complete matched untimed benchmark.
See milestones/170-nested-function-foundation.md for evidence. Borrowed/escaping
captures remain a genuine red witness for gates 171/172. A separate omitted-result
native witness is red with a passing independent Node reference; the current
unannotated named result is void, and inference is tracked in gate 173.

One agent works in this tree. Preserve the three owner fixture edits. Use explicit
Git paths and commits, one shared Cargo target with CARGO_INCREMENTAL=0, bounded
focused checks, complete benchmark support per implemented gate and SQLite records
with exact read-back. Refresh five executable consumers only after implementation.
No archives, routine full regressions or timing during these foundation chunks.

The next implementation stage is gate 171: capture planning and initialization
flow, under the settled existing ownership model. Gate 170 does not complete this
plan. Keep capture and inference red witnesses visible until their stages pass.
