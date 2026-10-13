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

## Checker continuation migration in progress

The working implementation begins by making suspended state explicit across the
whole observed cycle. `analysis/continuations` separates closure preparation and
publication, block entry/cursor/scope restoration, return inference/escape checks,
typed-expression failure-point publication, expression completion effects and
call capture-loan cleanup. These boundaries retain existing evaluation order;
closure capture access is still applied after its body is checked, block names and
loans are restored at the existing scope boundary, and call completion does not
publish native effects before argument evaluation finishes.

This is unfinished migration, not stackless acceptance. Remaining compatibility
operations can still recurse through closure parameter defaults, expression
preludes, contextual callee preparation, arguments, branches and exceptions. No
guard is removed and no stress ceiling is claimed. Installed consumers remain at
the last accepted public checkpoint while this migration is incomplete.

The driver must own a work stack and transfer partially checked nodes between
request and completion frames. Context slots must retain stable identities without
cloning entire caller contexts at every child expression. Child closure contexts
remain distinct from the suspended parent's context; finishing the closure applies
its captured accesses back to that parent. On failure, pending scope/callable
frames must preserve the existing ownership-diagnostic filtering and restore any
state required by callers that recover. Recursive entry into a second driver would
merely relocate the same problem, so all body/expression/default/dependency requests
must return to the same scheduler. Scalar or capture-free special cases are not the
intended architecture.

The first connected driver now schedules block bodies, including labeled blocks.
It moves each child body's existing allocation into a work frame while retaining
the parent statement's metadata. Scope entry, statement cursor advancement and
storage observations run once; completion restores the child tree before the
parent proceeds. Error propagation restores every suspended body without running
successful-scope completion effects. Labeled blocks retain their existing break,
owner and flow joins. No caller context is cloned for this scheduling operation.

Four focused internal witnesses pass for shadow restoration, typed child
preservation, nested-error tree restoration and locations, return reachability,
and labeled-break completion. These resolve the ownership-transfer uncertainty
needed to extend the driver; they are not a feature-family acceptance run.
SQLite records under `tsrust-nesting-driver` retain the checks and exact tested
source hashes. A follow-up run also passes all four witnesses after removing the
unreachable legacy block dispatch and its unused synchronous labeled-block
wrapper. That run checks formatted fixture behavior and formatter idempotence.

The driver now also schedules return expressions, conditional callees and closure
bodies on the same work stack. Closure contexts occupy separate owned slots; the
parent context remains suspended until capture effects are applied at completion.
Failed expression and body results carry their partially checked syntax back to
the parent, preserving callable ownership-diagnostic filtering. Root expressions
borrow their existing loans, and child callee tasks share them without copying
the loan map. Contextual literal completion uses explicit wrapper state.

The original mixed closure-body discovery now completes all phases at sampled
sizes 8, 16 and 32 on the normal test-worker stack. Size 16 previously overflowed
during analysis. Size 64 encounters the existing parser expression-depth guard;
that guard is remaining architectural debt, not an observed resource boundary.
These single diagnostic observations are not formal timing or scaling acceptance.
SQLite records under `tsrust-nesting-unified-driver` preserve the exact source,
phase events, executable hash and source hashes.

Focused native witnesses check nested owned and mutable captures, capture-transfer
diagnostic locations, callee-before-argument evaluation, argument order, default
parameters and contextual literal types. One initial order fixture tried to
replace a scalar while a returned callable still held its capture access. The
installed eee661c3 compiler rejects the same fixture with `capture-conflict`;
the fixture was corrected to observe callee order without violating that existing
contract. The original failure and baseline reproduction are retained.

The four driver restoration witnesses, three new integration witnesses and 31
existing callable-result, callback-return and literal-type regressions pass. The
corrected order witness is rerun alone; the other two new witnesses retain their
original passing results. These are focused migration checks. They do not replace
whole-pipeline acceptance, full compiler/LSP regressions or formal measurements.

Defaults, remaining expression variants, branches, loops, exceptions and
dynamically requested dependencies still need to yield to this driver. Native,
forwarded and specialized call paths also retain compatibility argument checking.
Compatibility calls can still start nested driver invocations, so the work is
not accepted as a stackless checker. Parser/type-syntax, final analysis, emission
and the other cross-phase paths in this map remain required. Whole-pipeline
stress, matched scaling measurements and integrated consumer acceptance remain
pending; the retained guards have no new justification from these samples.

## Queued call arguments and the next observed failure

Declared functions, ordinary local callable bindings, inline/conditional callees
and stored callable fields now queue their argument expressions on the shared
checker stack. Parameter adaptation retains optional borrowed-union and mutable
input conversions. Each call retains its original loan policy: declared calls
keep precise record-field loans and owned-inline-callback marking, while other
callable forms retain their existing root loans and capture-release boundaries.
Loan state is shared with the active child and updated after that child completes;
there is no additional loan-map copy for each sibling argument. Failed arguments
restore the callee and all partially checked arguments before propagating errors.

Five driver restoration witnesses, the stored-call consuming-access unit witness
and 86 affected integration tests pass. Integration coverage includes callable
expressions and fields, callbacks, optional parameters and facts, and native
nested-call witnesses. An initial unit build caught a private test still calling
the old synchronous named-call entry; that test now exercises the expression
scheduler while retaining its signature, target and consuming-access assertions.
The original failed build remains in SQLite.

Geometric discovery completes analysis at sampled size 64 for all four callable
forms. Declared arguments also complete emission and disposal at 64, then meet
the parser guard at 128. Bound, inline and stored-field arguments **overflow the
normal worker stack during emission at 64**. These are observed failures, not new
support limits. LLDB reproduces the bound-call failure and captures the repeated
`Emitter::expr` / `expr_dispatch` / `expr_dispatch_inner` / `expr_inner` cycle,
with closure-argument rendering in `expr_inner` recursively re-entering `expr`.
The debugger child is explicitly terminated after capturing the backtrace.

The next cross-phase implementation priority is an emitter work stack retaining
expression mode, logical expression depth, conditional temporary boundaries,
callee/argument ordering and ownership adapters. Merely moving the large leaf,
rewriting `map` as a loop, increasing the worker stack or adding an admission
limit would leave the recursive cycle intact. Remaining checker compatibility
paths and parser/type work stay required; the newly exposed emitter failure does
not narrow the architecture goal.

Matched compilation workloads are registered in `benchmarks/compiler/call-arguments`
using the existing two-warmup, eleven-sample protocol. All four forms validate
compiler check/emission and Node-after-tsc strict checking at sizes 8 and 32,
without timing samples. Native and Node execution both produce the independent
`7\n` oracle at size 32. The old generator's 28 sampled output combinations remain
byte-identical. Larger registered samples remain pending, and none of this is a
completed scaling measurement or whole-pipeline acceptance. SQLite records under
`tsrust-nesting-arguments` preserve sources, tool and source hashes, checks,
original failures, phase events, debugger output and untimed matched validation.

## Callable emission continuations in progress

`emit/expression_work.rs` now owns the suspended expression boundary and a stack
of receiver/argument requests. Declared functions, forwarded functions, materialized
generic methods, concrete closures, stored callables, stored fields and conditional
callbacks use this stack. Completion retains their existing ownership adapters,
argument counts, record reification and Result transport. Declared calls retain
pre-child transport/name lookup; forwarded calls retain post-child lookup. The
conditional record-point early return still has its distinct access-wrapper policy.
Logical expression depth and conditional temporary boundaries are restored after
each completed expression. Conditional receiver-access selection also uses a worklist.

The four original call-argument discovery families now complete all pipeline stages
at sizes 8, 16, 32 and 64 on normal test-worker stacks. At 128, all four encounter
the existing parser expression guard. The former bound/inline/field emitter crashes
at 64 are gone in this sweep. A phase-isolated checked-AST witness also emits and
drops 4,096 nested calls for each of those four families, checks exact expression
output and verifies temporary/depth restoration. It bypasses parsing only inside
an internal emitter test; it does not establish source acceptance at that depth.

This remains a migration checkpoint. Other expression variants, builtin children,
closure bodies and statement emission can still re-enter synchronous dispatch.
Repeated descendant string construction also remains a scaling concern. Parser,
checker compatibility paths, dependency publication and auxiliary consumers remain
required before removing guards or claiming whole-pipeline acceptance. No new
language limit, increased stack or completed formal timing is introduced.

The completed focused batch passes 210 integration tests across callable fields,
callbacks, forwarding, generic methods/adapters, optional parameters, call spread,
record callback failures and direct borrowed stores. The native nesting witness
now executes all four argument forms at 64. Ten representative generated-Rust
outputs match the previous compiler after substituting only the exact temporary
source filename; diagnostic spans, lines and columns remain compared. Matched
check/emit/Node-tsc validation passes at 8 and 32 for every form, without timing.
Whole-project formatting changes zero files and zero fingerprints.

SQLite records under `tsrust-nesting-emitter` retain original build/fixture failures,
the corrected emitter witness, native regressions, discovery, generated-output
comparison, benchmark validation and formatting. One initial fixture incorrectly
returned a value from `main`; another oracle repeated the integer literal suffix.
Both are corrected without changing compiler behavior. The original raw output
comparison also retains differences caused by its distinct temporary directories.
No full compiler/LSP milestone or consumer refresh is claimed here.

## Parser continuation migration (working tree)

`syntax/expression_work.rs` now queues precedence, associativity, unary completion,
grouping/comma operands, ordinary/explicit/optional call arguments, indexing,
array/record/template children and postfix completion. Operator frames retain token
indices because grouping and negative integer folding erase syntax needed by
exponentiation and nullish-mixing diagnostics. Explicit type arguments are published
at the same completed-call span. No nesting guard is increased or retired yet.

The first implementation still entered the driver recursively through arrow block
bodies. Normal-stack discovery caught a parser overflow at closure-body size 32;
LLDB identified the driver/closure-body/block/return/expression cycle. Arrow bodies,
ordinary nested blocks and return/throw/expression statements now yield to the
same work stack. Block recovery checkpoints retain token position, pending local
statements, explicit type applications, pattern prologues and completed-value
stack boundaries. Failed expression-bodied arrows restore their outer pending
statements. Four focused grammar/scope witnesses pass, and closure-body discovery
again completes sizes 8, 16 and 32 before the existing guard at 64. Inline-call
arguments complete through 64 before their parser guard at 128.

This exposes unfinished work rather than a new accepted boundary. Named declarations,
control statements, anonymous/named function bodies, parameter defaults, type syntax
and constructor-specific parsing still have synchronous compatibility paths. They
must join the scheduler with their own scope and recovery state. The current work
is not parser acceptance, and no supported depth follows from these observations.

The new mixed-literals discovery family combines grouping, record and array
construction, field/index access and calls. Size 8 completes the pipeline; size 16
parses but overflows during analysis. LLDB captures repeated checker-driver entry
through `record_literal`, `temporary_field` and `temporary_array_index`. Separate
binary and array discoveries also overflow during analysis at size 64. The
`assignment_values` integration suite aborts in its retained depth/diagnostic
witness; this failure remains open and must not be counted as passed or bypassed
with a new source guard. Preserve the completed array-literal-spread suite and
resume unrun suites. Parser migration and the remaining checker expression frames
must converge before whole-pipeline acceptance. Both debugger children are killed
explicitly after backtrace capture.

The resumed batch completes six suites (35 tests): call spread, comma expressions,
`in`, nesting continuations, numeric literals and optional chaining. Another 24
individual tests pass before four suites abort in their depth witnesses:
`comma_contexts::indexed_depth_canary_keeps_analysis_and_emission_bounded`,
`nullish_coalescing::nullish_parser_type_borrow_and_normal_depth_diagnostics_are_precise`,
`operators::unary_and_binary_nesting_are_bounded`, and
`templates::template_types_transfers_incoming_loans_and_normal_stack_are_diagnosed`.
Together with the earlier assignment witness, these are five unresolved integration
failures, not accepted exceptions. The completed checks remain preserved; tests
after an aborted witness still need execution after repair. Do not rerun passed
unaffected groups simply to produce a single green log.

Twelve representative valid/invalid inputs retain byte-identical generated output,
diagnostic text and exit status against `3f435e18`, using the same temporary source
path. Declared/inline matched check, emit and Node-tsc validations pass at 8 and 32
without timing. Whole-project formatting changes zero files/fingerprints. SQLite
records under `tsrust-nesting-parser-expression` and `tsrust-nesting-parser-body`
retain the original failures, phase observations, backtraces, corrected unit checks,
partial integration runs, output comparison and untimed benchmark validation.
This source checkpoint preserves unfinished migration; it does not accept the
parser or checker, refresh public consumers, or close the architecture requirement.

## Aggregate and access continuation source checkpoint

The checker now schedules ordinary array, record and template children, field
receivers and index operands on the existing expression work stack. Record
completion retains contextual callable conversion and native/scoped field
specialization. Field completion clears a parent projection on failure before
propagating the diagnostic. Index completion retains receiver loans and restores
partially checked child nodes before reporting errors. Spread construction,
collection properties and other compatibility dispatch remain migration debt.

The emitter now schedules aggregate children, temporary receivers, named-storage
index operands and comma-prefix effects on its expression work stack. Template
interpolations and discarded prefixes retain their individual temporary cleanup
boundaries. Temporary scalar-array specialization schedules literal elements
without adding an expression boundary to the fixed-storage literal. Storage path
collection handles sequence prefixes iteratively.

Three phase-isolated emitter tests pass on normal test-worker stacks, including
4,096 nested mixed call/record/array/access layers and 4,096 named index operands.
These sizes are witnesses, not supported maxima. Full-source discovery now passes
array sizes 8, 16, 32 and 64 and mixed-literal sizes 8 and 16. Larger sampled inputs
still encounter the old parser guard. That guard is unfinished migration, not a
resource policy or an accepted language limit.

The first affected integration batch exposed an emission overflow in the existing
100-level named-index witness. A phase probe confirmed that parsing and analysis
finished before emission overflowed. Named-storage operand scheduling repairs that
witness; its original regression now passes. All 26 tests in the arrays, comma-contexts and temporary-array-index suites now
pass after that repair, including the previously failing comma depth witness.
All four template tests also pass, including the formerly overflowing depth
witness. Earlier operator, assignment and nullish depth failures remain unresolved.
No architecture acceptance, consumer refresh or new formal scaling measurements
are claimed.

SQLite preserves the original interface failures, invalid-fixture test failure,
corrected tests, discovery results and named-index crash under
`tsrust-nesting-literal-*` and `tsrust-nesting-aggregate-*`. Closed records are imported with byte-exact restoration verification and a SQLite
integrity check before loose copies are removed. Source hashes distinguish the
aggregate/temporary access implementation from the subsequent named-storage repair. The invalid test
fixture used an untyped record outside its allowed context; correcting that
fixture did not change the language contract.

The resumed integration batch completed 12 nested-array, 14 nested-record and 42
scoped-callable-record tests. Its longer scoped-record-composition run reported
197 passes and five nullable-record parity failures. That run straddled the
named-storage CLI rebuild, so it is retained as mixed-version diagnostic evidence,
not a clean final-source acceptance run. The parity difference exposed a real
scheduling error: named field emission bypassed the nullable record parameter
bridge and accessed a concrete field instead of its readonly accessor. The bridge
now has its own receiver continuation, preserving accessor dispatch precedence
without recursively emitting its receiver. All 26 nullable-filtered composition
tests pass against matching final-source CLI/library builds, including all five
failed tests. Passing unrelated cases were not restarted.

Whole-project formatting changes zero files/fingerprints. This checkpoint remains
an unfinished architecture migration. Parser/type-syntax work is assigned through
`docs/handoffs/2026-10-13-claude-parser-context.md`; Codex retains checker and emitter
work and the integration/evidence writer role. No parser changes are included in
this aggregate/access section.
