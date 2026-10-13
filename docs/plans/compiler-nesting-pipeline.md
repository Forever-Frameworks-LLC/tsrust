# Compiler pipeline and continuation ownership

This map implements the owner's holistic nesting direction. It describes the
current source at `eee661c3`, with the discovery probe added afterward. It is not
an acceptance claim. Historical helper checkpoints remain evidence, not the
architecture's completion criteria.

## Current execution and re-entry

| Stage | Entry and state owner | Re-entry and unfinished work |
| --- | --- | --- |
| Lex and parse | `syntax::Parser` owns tokens, cursor, pending declarations, alias scopes, closure/pattern IDs and recovery state. `syntax::parse_entry_with` then lowers module state and resolves local functions. | Expressions, statements, closures and parameter defaults mutually enter one another. `types::syntax` parses annotations using the same nesting context; aliases expand during parsing. Lexer template handling, local-function resolution, module lowering and recovery belong in this boundary, not only expression parsing. |
| Source proof and materialization | `Analyzer::new` prepares namespaces/readiness, then `generics::functions::specialize` runs source proofs before draining template instances. Generic record materialization follows. | The proof engine has resumable invocation/body sessions, but full caller states and transitive facts are copied through requests. A separate engine starts for each source function. Materialization is iterative, but instance budgets and checked-request depth guards still restrict it. |
| Body/type/ownership analysis | `Analyzer` owns type arenas, signatures, capture/effect facts and checked bodies. A function takes its body out of the program and constructs a mutable `Context`. | `checked_expression`, `block_with_scope`, closure checking and function checking share depth accounting. Generic calls enter inference, publish new functions, materialize records, extend signatures and request result hints. This can re-enter body checking with caller-dependent capture state. |
| Recursive access inference | `analyze_recursive` owns the source program and an access lattice. | It clones/rebuilds analyzers for successive lattice rounds, followed by definitive checking. This is semantically separate from arbitrary nesting. Lattice progress has a mathematical bound; whole-program replay and repeated proof construction still have scaling costs. |
| Result discovery and deferred analysis | Analyzer result/callback discovery and postponed bodies depend on evolving signatures and record callback origins. | Bodies containing record parameters or exception regions may be cloned and revisited. Eager global ordering alone cannot replace this: earlier consumer refresh found that record callback environments depend on caller construction. |
| Finalization | `finish_analysis` validates callable storage/layouts, computes definite assignment and effects, then prepares named capture lowering. | `definitely_assigned` asks emission whether exception transport is needed and invokes exception-program validation. Final analysis is therefore not independent of emission traversal. These paths must support the same deep inputs as body checking. |
| Emission | `emit::Emitter` reads the checked program and owns temporary/output bookkeeping. | `expr` recursively dispatches children and restores conditional-temporary boundaries; blocks and type rendering also traverse nested structures. Generated runtime console truncation is a separate observable API policy, not a compiler nesting guard. Backend Rust compilation must be observed separately. |
| Disposal and auxiliary consumers | Source AST and annotation cloning/disposal already have explicit worklists. | Debug rendering, checked side tables, error unwinding, formatting, semantic diagnostic replay, project linking and LSP consumers must also survive the shapes newly admitted by the main pipeline. `check` reparses for recursive inference; diagnostics can parse and analyze again. Project linking calls the same analyzer after its own parse/link steps. |

## Required ownership model

Parsing needs tagged continuations for grammar position and parent construction,
with one explicit owner of token position, scopes and recovery boundaries. Child
completion must resume the exact parent operation, including mixed annotation,
default-expression and closure-body paths. Separate iterative helpers do not
suffice when their calls still form a recursive cycle.

Checking needs suspended function, block and expression frames that own their
partially checked nodes and context transitions. A frame must retain expected
type, access mode, loans, lexical shadow restoration, conditional facts, exception
region and diagnostic site. New generic instances or result dependencies must
suspend the current operation and return a dependency request to the scheduler;
they must not recursively check another body. Completion publishes facts and
resumes the original operation in source evaluation order. Active recursion needs
explicit provisional signatures and lattice dependencies, not another depth cap.

Proof analysis needs summaries of relevant inputs and observable effects rather
than whole caller snapshots threaded through every callee. Reuse keys must include
lexical/type identities, actual callable origins, capture instance identity and
point-sensitive versions. Returned callbacks and callback arguments can reference
caller bindings outside the callee's lexical scope, so simply dropping all
nonlexical slots is unsound. Projecting effects back must preserve untouched
bindings and distinguish per-instance mutation, consumed environments and merged
instances. Establish those dependencies before caching summaries across roots.

Emission needs output continuations carrying mode and temporary-lifetime
boundaries. Child fragments should append to an output sink or use bounded-cost
fragment storage instead of repeatedly copying an entire descendant string.
Emission validation, diagnostics and disposal must use the same structural
coverage before parser/checker guards are retired.

## Coherent implementation and acceptance

1. Establish stage observations and a mixed-shape discovery corpus. Classify each
   encountered stop as an implementation guard, semantic rejection, resource
   exhaustion, crash, operational timeout or unfinished observation. Do not infer
   a support ceiling from the last passing sample.
2. Design and implement dependency suspension together with capture-aware proof
   summaries and result publication. Preserve source spans, ownership and effects
   through dynamic specialization and recursive inference. Avoid another sequence
   of isolated guard edits or scalar-only scheduling exceptions.
3. Implement syntax/type/default continuations and checker structural frames with
   their emission, diagnostic and disposal consumers. Use the map above to test
   phase transitions, not just isolated structures constructed inside unit tests.
4. Grow mixed finite inputs geometrically on normal stacks, recording phase times,
   work counts, peak process memory and output size. Exercise imported projects,
   formatting, LSP diagnostics and native execution where applicable. Measure the
   backend separately. Only observed resource behavior should inform operational
   budgets; fixed product limits require independent justification.

The opt-in `nesting_probe::observe_pipeline_stages` test marks parse, analysis,
recursive replay, emission and disposal. `benchmarks/compiler/nesting-discovery`
runs each shape/size in a separate ordinary test worker process and retains exact
source, events, exit status and peak RSS. It is initial observability: it does not
yet separate proof/materialization/type-check subphases or cover every mixed shape.
Formal matched timing protocols remain unchanged.

## First discovery evidence (2026-10-13)

The initial sweep observes 35 cases across seven families: 28 complete parse,
analysis, emission and disposal. Grouping, blocks, binary expressions, grouped
types and arrays encounter parser guards at sampled size 128. Inferred generic
chains encounter the retained analyzer-request guard at sampled size 64. Explicit
generic size 256 is stopped by the ten-second diagnostic budget during analysis,
with peak process RSS 170,688,512 bytes. This is neither memory exhaustion nor a
supported-size boundary. Original records retain every smaller observation.

A separate mixed call/closure/body sweep completes size 8, then **overflows the
normal test-worker stack at size 16 during analysis**. LLDB reproduces the fault
and captures a repeated cycle through `expression`, call dispatch,
`checked_expression`, `closure`, `check_source_callable_body`, `block_with_scope`
and `return_statement`. The crash occurs on entry to `owned_assignment_value`;
that leaf is where stack space runs out, not an independently established root
cause. Moving or shrinking that leaf would not remove the recursive cycle.

This changes the next implementation priority: central checker continuations must
cover the entire call/expression/closure/body/return cycle, including ownership
context restoration and result publication. Do not add a closure-depth guard,
raise the stack, or declare size 8 an acceptable ceiling. The same driver must
subsequently handle named and dynamically materialized function dependencies.
Capture-aware proof reuse remains necessary for compilation scaling.

The machine has 8,589,934,592 bytes of physical memory, observed separately because
the sandbox blocks `sysctl` during ordinary driver execution. The original JSON
correctly records unavailable hardware metadata; it is not rewritten to hide the
observation failure. Peak RSS is still recorded independently per child process.
SQLite records under `tsrust-nesting-discovery` retain both sweeps, source bytes,
phase events, debugger transcripts and build/format evidence. The debugger child
is explicitly terminated after capture. No production limit or behavior changes
in this diagnostic checkpoint, and no formal timing acceptance is claimed.
