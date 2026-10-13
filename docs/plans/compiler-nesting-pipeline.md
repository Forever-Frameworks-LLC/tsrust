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

## Operator continuation source checkpoint

`analysis/continuations/operators.rs` now retains operand modes, contextual types,
string loans, observer places, capture-loan boundaries and comparison guard
snapshots across the shared checker work stack. Arithmetic, ordering, ordinary
strict equality, typeof guards, absent-sentinel equality and primitive-union
equality schedule their left and right operands. Unary arithmetic, negation,
truthiness, inspection, variant tests and non-null assertions schedule their
operands while preserving diagnostic locations and narrowing completion.

The emitter schedules ordinary unary/binary operands on its expression work
stack. Conditional-arm preparation and completion are separate, preserving
materialized temporary declarations and borrowed-union view conversion for typeof
and truthiness. Equality still binds operands in its match scrutinee, preserving
operand temporary lifetimes; checked arithmetic and special equality retain their
existing access-wrapper policy. Non-null projection emission and synthesized
wrappers remain compatibility paths and are not claimed stackless here.

All 58 tests across operators, int operators, number operators, string operators,
truthiness, data unions, nullable narrowing, non-null assertions and JSON typeof
pass against matching CLI/library builds. This includes the formerly overflowing
unary/binary nesting witness. A phase-isolated mixed unary/binary emission witness
passes at 4,096 layers on a normal test-worker stack with exact output and empty
temporary bookkeeping. The original operand-filtered unit run preserves three
passes and a failed new cleanup witness; the corrected expression-only witness
passes separately. No passing unrelated groups were restarted.

The new witness initially assumed the wrong arithmetic diagnostic span (the
existing contract points at `+`, not its boolean operand). It also expected a
failed block to restore the analyzer's outer block depth. Existing block error
handling returns without its successful scope-exit cleanup, so that assumption
was outside the operator change. The corrected witness exercises the expression
driver directly and proves observer, comparison-marker and expression-depth
restoration while retaining the failed tree. Error restoration for suspended
block scopes remains a concrete audit item; it must not be inferred from these
expression checks.

Full-source binary discovery completes samples 8, 16, 32 and 64 through disposal;
size 128 still encounters the retained parser guard. These are diagnostic
observations, not a support ceiling or formal scaling acceptance. The pending
original depth failures are now assignment values and nullish coalescing. Logical
branch checking, conditional/assignment state, synthesized wrappers, generic
requests and other compatibility paths still require connected continuations.
Parser/type-syntax work remains separate under the Claude handoff.

Whole-project formatting changes zero files/fingerprints. SQLite records under
`tsrust-nesting-operators-*` preserve the original interface output, both failed
witness runs, the corrected witness, complete affected integration results,
discovery sources/events, formatting and source/executable identity. Formal
matched timing protocols and measurement status are unchanged. This checkpoint
does not accept the full architecture or refresh installed consumers.


### Conditional branch continuation checkpoint

Conditional checking now suspends the predicate and both arms on the expression
scheduler. Existing branch planning, stored-callable conversion, availability,
flow and capture-loan joins retain their ordering. A failed-arm witness verifies
restoration of both AST children, the original diagnostic span and expression
depth. This does not prove outer block error cleanup.

Emission schedules conditional predicates and arms, including borrowed-union
injection. Explicit carrier-spine state preserves the existing callable carrier
inference boundary: inference propagates through conditional branches and view
injection, without leaking into nested call arguments. Ordinary expression
wrappers and conditional temporary lifetimes retain their previous boundaries.

Five emission unit witnesses pass, including exact output at 4,096 nested
conditional arms on the normal test-worker stack. The diagnostic restoration
witness passes. All 56 integration tests across conditional expressions, nullable
callables and nullable unions pass, including native ownership, callback failures,
branch laziness and disposal assertions.

Discovery now includes conditional and mixed conditional/call/record/array/access
families. Conditional samples 8/16/32 and mixed samples 8/16 complete the full
pipeline through disposal. Their next geometric samples encounter the existing
parser 128-level guard. These samples are observations, not support ceilings;
no formal scaling measurement or full architecture acceptance is claimed.

Whole-project formatting changes zero files and zero benchmark fingerprints.
SQLite records under `tsrust-nesting-branches-*` preserve interface checks,
unit/integration results, discovery sources/events, formatting and source identity.
Coalescing, logical checking, assignment, synthesized wrappers and remaining
recursive auxiliary consumers still need connected continuation work. Parser work
remains separately owned. Installed consumers remain at the accepted public
checkpoint; S12–S21 and optimization remain pending.


### Coalescing operand continuation checkpoint

Coalescing and destructuring defaults now schedule both checked operands on the
shared expression work stack. The left frame retains pre-evaluation availability
and flow; the right frame distinguishes always-left, always-fallback and joined
branches. Unreachable fallbacks use a scheduler-owned child context and still
produce their original diagnostics. Joined branches preserve absent-source
availability, fallback loans, contextual types and capture-owner/flow joins.
The selected-left rewrite transfers its subtree directly instead of cloning it
as a disposable placeholder.

Emission separates operand planning from branch output, including physical-tag
selection for narrowed union sources. Each operand uses the expression work
stack with its original access mode. Pattern-default owner allocation and
conditional temporaries retain their preparation/completion order. Selected
coalescing, optional-selected and rest-pack wrappers also schedule their child.
Other optional-chain and assignment traversal remains migration work.

Two focused unit tests pass: reachable/unreachable fallback errors restore child
nodes and diagnostic spans, and absent/dynamic/selected coalescing emission
matches exact expected output at 4,096 layers on normal test-worker stacks.
All 24 integration tests across nullish coalescing, destructuring defaults and
nested rest patterns pass. The previously overflowing nullish normal-depth
witness now passes, alongside laziness, ownership, source-loan and disposal
checks. Assignment-value depth failure remains pending.

Geometric discovery completes coalescing samples 8/16/32 and mixed
coalescing/conditional/call/record/array/access samples 8/16 through disposal.
The next samples encounter the existing parser guard. No sample count becomes
a support ceiling, and discovery is not formal matched scaling acceptance.
Whole-project formatting changes zero files/fingerprints. Closed evidence and
source/executable hashes are retained under `tsrust-nesting-coalesce-*` in SQLite.

Logical values and boolean guard checking remain recursive operand consumers.
Their continuation must preserve skipped/reached flow, borrowed selected-left
loans and separate contexts for unreachable-right validation. These are next
connected branch paths, followed by assignment/control-flow and auxiliary
consumers. The full architecture, optimization and S12–S21 remain unaccepted;
installed consumers are unchanged.


### Logical branch continuation checkpoint

Boolean logical guards and value-returning logical selection now schedule both
operands on the checker work stack. Boolean frames preserve selected/skipped
condition outcomes and availability joins. Value frames preserve selected-left
loans, contextual right types, unreachable-right child contexts, borrowed-result
restrictions and capture-owner joins. Original helpers for type hints and union
selection remain shared with their existing consumers.

Logical value emission separates access planning from final branch construction.
Reachable operands use the emission work stack with their original conditional
temporary policy; statically unreachable right operands are not emitted. Boolean
logical emission already uses the binary operand work path. This change does not
claim auxiliary type-hint or union-payload traversals are fully migrated.

Two focused unit tests pass: exact emission through 4,096-layer boolean and value
logical chains, and diagnostic/tree/depth restoration for boolean, value and
unreachable-right checker failures. All 18 integration tests across logical
values, logical assignment, nullable narrowing and truthiness pass, including
native short-circuit behavior and ownership checks.

Full-pipeline discovery completes value and boolean logical samples 8/16/32 and
a mixed logical/coalescing/conditional/call/record/array/access sample at eight
layers. The next samples encounter existing parser nesting guards. These are
observations, not support ceilings or formal scaling measurements. Whole-project
formatting changes zero files/fingerprints. SQLite records under
`tsrust-nesting-logical-*` retain checks, discovery and source/executable identity.

The next assignment section must cover target selection, mutation bookkeeping,
scalar/compound/logical/nullish assignment, update operations and statement
reassignment together. `mutation_target` temporarily modifies mutation_place,
keep_flow and suppressed_flow around its synchronous expression call; scalar
assignment also restores replacement state before checking its right operand.
The continuation must retain and restore these boundaries on errors. Shared
assignment discovery currently calls scalar_assignment directly, so migrating
only expression dispatch would leave a recursive path. Assignment emission,
control statements, synthesized wrappers and auxiliary traversals remain pending.
No full architecture acceptance, formal timing or installed consumer refresh is
claimed. Optimization and S12–S21 remain required.


### Assignment operand continuation checkpoint

Scalar, compound and boolean logical assignment, plus update operations, now use
an assignment task on the shared checker work stack. Legacy statement and scalar
discovery entry points invoke this task, preserving their existing outer checking
boundaries. Nested expression assignments suspend directly on the same scheduler.
Nullish assignment separately retains target/fallback state on that scheduler.

Mutation selection now has explicit prepare/finish boundaries. Target errors
restore mutation_place, keep_flow, suppressed_flow and scalar replacement state
before propagation. RHS frames retain selection loans, old-value access policy,
conditional availability and integer failure-point publication. Nullish frames
preserve prior narrowing, callback-origin validation and the existing unreachable
RHS context-restoration policy.

Emission queues storage index/discard operands before RHS values for scalar,
compound, logical and nullish assignments and updates. Shared rendering helpers
consume completed operands while preserving saved-index reacquisition, bounds,
old-value evaluation and conditional stores. Statically present nullish targets
still omit RHS emission. Owned void stores and sequence traversal remain explicit
compatibility paths; this checkpoint does not claim all assignment lowering is
stackless.

Two focused unit tests pass: mutation-scope/tree restoration on target and RHS
errors, and exact emission at 4,096 layers across scalar, compound, logical,
nullish and indexed-update shapes. All 32 integration tests pass across assignment
values, compound assignment/statements, compound values, logical assignment,
updates, nullish assignment/aliases, owned values and assignment callees. This
repairs the previously overflowing assignment-value depth witness. Both original
remaining depth failures (assignment values and nullish coalescing) now pass;
that fact alone does not prove the broader compiler architecture complete.

Discovery completes scalar and compound assignment samples 8/16/32 and a mixed
assignment/logical/coalescing/conditional/indexed sample at eight layers through
disposal. Next geometric samples hit existing parser guards. These observations
are not language ceilings or formal scaling acceptance. Whole-project formatting
changes zero files/fingerprints. SQLite records under
`tsrust-nesting-assignment-*` retain both intermediate interface checks, final
unit/integration evidence, discovery and exact source/executable hashes.

Next, sequence scheduling must retain per-prefix capture boundaries, projection
restoration, array-removal snapshots and discarded-value behavior. Discarded owned
assignments route through assign_statement_with_loans; string compound prefixes
allow owned statement behavior. Emission prefixes drain their own conditional
temporaries. These paths must migrate together with owned store completion,
followed by control statements and auxiliary consumers. Full architecture,
optimization, S12–S21, formal measurements and installed consumer refresh remain
pending.


### Sequence and owned-store continuation checkpoint

Sequences, discarded operands and owned stores now have tasks on the shared
checker scheduler. Sequence prefixes retain per-operand capture boundaries,
projection state and precise array-removal snapshots. Final operands retain
binding, callable-origin and condition-outcome propagation. Snapshot completion
distinguishes an unreached final operand from a reached operand with no removal,
so failure does not overwrite a caller's earlier snapshot prematurely.

Discarded named/field stores and string compound assignments schedule their
replacement operands. Named replacement and precise sequence reassignment use
the same store task as nested sequence prefixes. Field replacement suspends
mutation selection, stored-callable contextual conversion and ownership/native
view commit separately. Obsolete recursive mutation/discard helpers are removed.
Remaining semantic helper calls are not assumed stack-safe merely because these
primary operands now use the scheduler.

Emission schedules sequence prefixes/final values, preserving the sequence's
original expression-boundary policy and per-prefix conditional temporary drains.
Owned void assignments now queue their RHS and named-storage operands too;
shared render helpers preserve capture restoration and field replacement order.
The obsolete recursive sequence emitter is removed.

Three focused unit tests pass, including direct checker and exact emission
witnesses at 4,096 sequence layers on normal stacks and error restoration for
named stores, indexed stores and string compound prefixes. The initial test build
failed on an orphaned attribute left when deleting an obsolete helper; its log
is preserved and the corrected run passes. All 65 integration tests across comma
expressions/contexts, owned assignment values, assignment callees, array removal,
pop/shift, scoped record parameter replacement and caller-borrowed record stores
pass. These include removal snapshots, capture release, allocation/pointer and
native disposal assertions.

Discovery completes sequence and owned-store sequence samples 8/16/32/64 through
disposal. Sample 128 reaches the existing parser guard for both families. These
are observations, not nesting ceilings or formal timing acceptance. Whole-project
formatting changes zero files/fingerprints. SQLite records under
`tsrust-nesting-sequence-*` preserve interface output, original failure, repaired
unit run, integrations, discovery and source/executable identity.

The block scheduler still directly handles only nested blocks and returns;
other statements enter compatibility checking. Declaration initializers,
conditions, loop bodies, exception paths and their scope cleanup must join the
scheduler. Block errors still require the previously recorded restoration audit.
Parser/type-syntax work remains separate. Full compiler architecture, formal
scaling, optimization, S12–S21 and installed consumer refresh remain pending.


### Declaration and branch checker continuations

Declaration initializers, expression statements and `if` conditions/arms now
suspend on the shared checker worklist. Declaration frames preserve contextual
callback types, view modes, precise removal snapshots and the module-literal
scope through initializer completion. Branch frames retain incoming availability,
flow and capture ownership, restore the alternate entry state, then apply the
existing reachable-arm joins and carrier/storage planning. Partial AST children
return to their parent slots before propagating errors.

Block scope cleanup now runs after both child failure and final carrier/borrow
validation failure. It restores lexical visibility, shadowed native slots,
capture boundaries, mutable view tracking and analysis depth. Failed labeled
blocks and abandoned compatibility-loop frames release their control targets.
The discarded-scalar classifier uses an explicit worklist too.

All 14 checker continuation unit tests pass, including new failures through both
branch arms, closure declaration initializers and labeled loop bodies. All 78 affected compiler integration tests and 19 editor tests pass, including
native ownership/control flow, comma removal contexts and project diagnostics.
No architecture acceptance is claimed.
Discovery completes `if-statements` samples 8/16/32/64 through disposal; 128
reaches the existing parser guard. `initializer-bodies` completes eight layers
but overflows the normal parser test-thread stack at sixteen, before the parse
finish event. Reproduce with the discovery driver's corresponding family; the
record includes exact generated source and its hash. This belongs to the separate
parser migration, not a new rejection policy. Discovery ran while the integration
build was active, so its time/memory observations are not isolated performance
measurements.

Next work still includes statement assignments, loop and exception continuations,
statement emission, recursive auxiliary consumers and retirement of obsolete
analysis guards after their connected paths are migrated. Existing emission
compatibility paths remain unchanged by this checker section. Formal scaling,
optimization, S12–S21 and installed consumer refresh remain pending.

Whole-project formatting changes zero files and source fingerprints. SQLite
records under `tsrust-nesting-statements-*` preserve the interface check, unit and
integration runs, editor checks, discovery, formatting and exact source/executable
identities. The recursive `assignments` and `record_field_move_carriers` visitors
in `analysis/record_branch_carriers.rs` remain explicit audit targets, along with
loop/exception checking and emission control/storage-epoch visitors.


### Loop and assignment checker continuations

The shared checker worklist now owns every ordinary loop operand and body:
`while` conditions, `for...of` iterable evaluation and element scope, and counted
or post-test loop bodies, updates and predicates. Tail loops keep a separate
scheduled context for syntactically present unreachable tails. That context is
removed on success and failure; its availability effects never enter reachable
exits. Loop frames preserve entry/zero-iteration state, continue and break owners,
back-edge validation, guard refinement and map-storage edge observations.

Statement assignments select the existing scalar/compound/store tasks directly.
Their target and replacement trees are restored before diagnostic propagation;
precise removal sequences keep their prior named-store behavior. Obsolete
recursive statement dispatch methods are removed. Iterator loans and names are
released when body checking or back-edge validation fails, as well as on success.

Focused failure witnesses cover nested iterators, reachable and unreachable tails,
post-test predicates, and scalar/compound/owned assignment initializers containing
loops. The first unit build found a stale test calling the removed `tail_loop`
entry and an incorrectly quoted test source. The witness now uses the production
block scheduler; its break/continue/return owner assertions remain intact. Both
original errors are preserved. All 271 analysis unit tests, 75 affected compiler
integrations and 18 editor tests pass. Whole-project formatting changes
zero files and fingerprints. No full architecture acceptance is claimed.

The remaining checker compatibility dispatch is exception propagation and nested
exception regions. Ordinary statement emission still recurses through control
bodies and storage-epoch helpers. Auxiliary owner/flow visitors, including
`loan_roots`, also require the connected traversal audit. Existing analysis guards
remain migration debt; no constants or stacks were increased. Parser work remains
separate, and formal scaling, optimization, S12–S21 and consumer refresh are pending.

Mixed-loop discovery completes samples 8/16/32/64 through disposal; 128 reaches
the inherited parser guard. The run overlapped integration compilation and is
not isolated performance evidence. SQLite records under
`tsrust-nesting-loop-work-*` preserve the original failed unit build, repaired
checks, discovery, formatting and final source/executable identities.

#### Owner correction: stop repeating known guard observations

The owner reiterated that arbitrary depth caps and incremental limit chasing
waste effort. No new depth caps or stack increases are authorized. The parser's
inherited internal counter guard is not a measured hardware limit and does not
map directly to source nesting depth. Retire it through the connected parser
migration; retire equivalent checker guards through the connected checker audit.
Do not repeatedly run geometric full-pipeline discovery merely to rediscover a
known parser guard. During implementation, batch focused semantic/error-restoration
checks across coherent subsystems. Use phase-isolated witnesses where they answer
a new architectural question. Resume broad deep-input and scaling measurements
once the relevant connected paths and obsolete guards have been removed, or when
new evidence warrants a specific probe. A successful fixed-depth witness never
establishes a product ceiling or completes architecture acceptance.


### Exception region and callable continuations

Exception regions now suspend their try body, catch body, canonical cleanup and
per-destination cleanup replay on the shared checker scheduler. Exit-frame setup
and completion are separate operations. Enclosing completion/error frames,
region-entry loans, loop depth, exception environment and catch-name visibility
are restored on diagnostic paths. Child source slots are restored before errors
propagate. Catch bookkeeping is published only after successful completion,
matching the previous speculative-check behavior.

Cleanup replay retains the original normal-first/destination ordering. Normal
cleanup preserves an incoming completion destination; abrupt cleanup replaces it.
Canonical cleanup uses an isolated context, while each ownership replay resets
the declaration prefix and source body. The canonical typed body/bindings are
published after replay, preserving existing closure identities and storage plans.
The callback-based completion helpers retained for tests share the same merge
operations as the production scheduler.

Exception-bearing closure bodies now suspend through a callable task, removing
the former compatibility call from the expression scheduler. Root callable
checking uses the same task. Thrown values schedule their expression evaluation
between the existing ownership checks and typed-completion recording.

The initial region implementation passes all 272 analysis tests. After integrating
exception-bearing closures and verifying handler bookkeeping order, all 158
affected exception unit tests pass. New witnesses cover failure in try/catch/
finally, nested regions, cleanup after control flow and exception-bearing closures;
they verify original diagnostic spans, restored source children, enclosing frames,
catch shadowing and analysis depth. All 65 native compiler integration tests and 22 editor tests pass. Whole-project
formatting changes zero files and fingerprints.
The interface-check log predates the final callable integration. These are
implementation checks, not full architecture or performance acceptance.

Result discovery, closure preparation/defaults and semantic effect helpers still
need their own connected audit. In particular, `check_pending_region_functions`
can invoke function checking while refining region contracts, and
`discover_checked_exception_result` enters checking from result discovery.
Statement emission and recursive auxiliary owner/type visitors remain pending.
No depth constants changed and no repeated known-guard discovery ran.

SQLite records under `tsrust-nesting-exception-*` preserve the intermediate
interface/analysis pass, final affected exception pass, native/editor runs,
formatting and source/executable identities. The 272-test initial pass predates
the callable integration; the 158-test affected pass covers that integration and
handler-bookkeeping restoration. A production-used region-body adapter remains
available outside test builds; retaining its availability does not change its
code under `cfg(test)`. Installed consumers remain at the last accepted public
checkpoint; full nesting architecture and measurements remain unaccepted.

## Ownership provenance helper worklists

The checker now collects loan roots and view sources with explicit expression
worklists. Root binding lookup uses a loop, and stable receiver classification
uses a worklist. Loan roots preserve first-encounter order and deduplication;
view sources retain sorted deduplication. Sequence prefixes and conditional
predicates remain excluded, and receiver access modes retain their prior rules.
The directly invoked field-path and named-storage helpers already use loops.

Two focused tests pass, including a parser-independent 16,384-layer checked
expression witness on the normal test-worker stack with normal disposal. The
analysis batch passes 274 tests. Whole-project formatting changes zero files or
fingerprints. Evidence uses the `tsrust-nesting-ownership-*` SQLite records.
The final source differs from the tested source only by indentation in root
binding lookup. No parser discovery, scaling measurement, public contract change,
or consumer refresh is claimed. Record carrier walks, result discovery and
statement emission remain among the connected migration work; compiler-wide
architecture acceptance remains open.

### Record ownership statement worklists

Record branch assignment collection and field-move carrier propagation now use
explicit mutable statement worklists. Child scheduling preserves source order,
including then/else, loop body/update, and try/catch/finally ordering. Distinct
owned-closure detection iterates record field pairs instead of recursing.
Control-store marking uses a body worklist while retaining each lexical body's
own declaration analysis. It omits descendant store scans for bodies without
bound declarations, where no local can be marked by that scan.

The analysis batch passes 276 tests. New witnesses check assignment traversal
order and 16,384 nested statement blocks, including disposal, on normal stacks.
The scoped callable record, nullable caller callback store and nullable callable
compaction integration suites pass 56 tests, preserving native results, ownership
rejections and formatting round trips. Whole-project formatting changes zero
files or fingerprints. SQLite records use `tsrust-nesting-record-walk-*` names;
a separate record identifies the final indentation-only source adjustment.

These changes do not retire the remaining compiler guards or establish a depth
ceiling. Recursive carrier conversion, carrier retargeting and record type joins
remain connected migration work. No new formal scaling measurement or consumer
refresh is claimed, and full architecture acceptance remains open.

### Carrier compatibility and nested field replacement

Carrier compatibility now uses continuation tasks for array elements, record
field conjunctions and union member/alternative searches. Failed union candidates
continue to later candidates; a failed required field or member short-circuits
the enclosing conjunction. Existing callable compatibility rules are unchanged.
Nested record field replacement retains parent fields explicitly and materializes
from the replaced leaf outward, preserving sibling types and original records.

The analysis batch passes 279 tests. New witnesses exercise matching and
mismatching record pairs at 16,384 layers on normal stacks, union alternatives,
and nested field replacement with a missing-field failure that materializes
nothing. Native integration suites pass 257 tests: scoped callable records 42,
nullable caller callback stores 13, and scoped record composition 202. The last
suite includes ownership rejection, native output, exception cleanup, map storage
and formatting checks and took 549.79 seconds. This is test execution evidence,
not a compilation scaling measurement. Whole-project formatting changed zero
files or fingerprints.

SQLite records use `tsrust-nesting-carrier-compatibility.log` and
`tsrust-nesting-carrier-path-*`. The earlier compatibility-only test run predates
the field replacement change; the 279-test analysis run and all three native
suites cover the final implementation. No public contract or IntelliSense change
is introduced, and consumers have not been refreshed. Full nesting acceptance
and formal scaling remain open. Remaining connected paths include carrier
retargeting, record type joins, recursive record-value carrier inference in
`record_value_carriers.rs`, result discovery and statement emission.

### Record-value carrier inference continuations

`record_value_carriers.rs` now uses explicit expression, record-field and wrapper
continuations. Each field completes before its parent layout is materialized;
wrappers retain their checked type and diagnostic span across child inference.
The original local-storage plan lookup, parameter/capture exclusions, unchecked
value result and last-sequence-value rules remain in place.

Three focused tests pass, including 16,384 checked carrier wrappers and ordinary
disposal on a normal test stack, concrete field replacement without source AST
mutation, and unchecked/sequence behavior. All 42 scoped callable record native
integration tests pass, including normalized storage propagation and field moves.
Whole-project formatting changed zero files or fingerprints. SQLite evidence is
stored under `tsrust-nesting-carrier-inference-*`.

This removes this helper's recursive expression traversal, not the remaining
record type joins that wrapper completion may invoke. Carrier retargeting, joins,
result discovery and statement emission remain open. There is no new public
contract, consumer refresh, formal scaling acceptance or compiler-wide nesting
acceptance in this checkpoint.

### Shared record and scoped-map type joins

`analysis/carrier_type_join.rs` now owns one continuation stack for record joins
and scoped-map value joins. This removes the mutual recursion between
`join_record_types` and `join_scoped_map_types`, including mixed record/map paths.
Record fields retain their original order and checked span. Direct record fields
keep the original left-type fallback when a child has no join; array record
fields still require a successful child join with matching rank. Map validation,
callable-family fallback and parent layout materialization retain their previous
order and conditions.

All 285 analysis tests pass. New witnesses cover 16,384 record layers on normal
stacks, left fallback, array-rank and child-join rejection, incompatible scalars,
and non-map inputs. Native validation passes all 42 scoped callable record tests
and 11 scoped record composition tests selected by `origins`, exercising distinct
callback origins through records, maps, arrays, loops and storage replacement.
Whole-project formatting changes zero files or fingerprints. Evidence is stored
in SQLite under `tsrust-nesting-carrier-join-*`.

The completed earlier 257-test native run remains preserved; this change reruns
affected suites rather than restarting that full composition pass. No formal
scaling measurement, public contract change or consumer refresh is claimed.
Expression carrier retargeting, literal record-value joins, result discovery and
statement emission remain connected work before full architecture acceptance.

### Mutable carrier retargeting continuations

`analysis/carrier_retarget.rs` replaces recursive nullable-store retargeting with
owned parent frames for injections, arrays, record fields and carrier wrappers.
Completed children are restored before parent metadata is published. Diagnostic
unwinds restore every extracted child without publishing unfinished parent tags
or types, while retaining completed prefix mutations as the original traversal
did. Wrapper removal preserves the inner expression and its span. Record fields
continue to use the iterative field replacement and carrier compatibility paths.

All 289 analysis tests pass. New witnesses exercise 16,384 wrappers on normal
stacks for both success and failure, exact diagnostic spans, complete parent
restoration, array prefix mutation before a later failure, and unchanged injection
tags on payload failure. Native validation passes 42 scoped callable record tests
and 26 nullable scoped-record composition tests. Whole-project formatting changes
zero files or fingerprints. SQLite evidence uses `tsrust-nesting-carrier-retarget-*`.

This is internal architecture work with no public contract or IntelliSense change.
Consumers remain unchanged. Literal record-value joins, result discovery and
statement emission still need connected migration; full compiler nesting and
formal scaling acceptance remain open.

### Paired literal record join continuations

`analysis/carrier_literal_join.rs` replaces recursive literal field joins with
paired owned parent frames. It retains declaration field order, original field
types for fallback selection, nominal rejection, native-view/map/callable
conversion order, parent materialization timing and diagnostic spans. Both
extracted children are restored before either normal completion or error unwind.
Completed prefix mutations remain visible on a later error, as before.

All 292 analysis tests pass. New witnesses cover 16,384 nested literal pairs with
normal disposal, a callback-contract error under 4,096 paired parents with exact
leaf diagnostic span and both trees restored, and incompatible nominal records.
Native validation passes 42 scoped callable record tests and 11 composition
`origins` cases. Whole-project formatting changes zero files or fingerprints.
SQLite evidence uses `tsrust-nesting-literal-join-*`.

The audit identifies a remaining connected cycle between nullable/sticky joins
and empty scoped record/map environment construction. Address those paths as one
subsystem, including their recursive environment helpers. Result discovery and
statement emission also remain open. No public contract change, consumer refresh,
formal scaling acceptance or full compiler architecture acceptance is claimed.

### Nullable joins and empty environments

Nullable and sticky record joins now share the carrier type-join stack. Empty
record and map environment construction uses one worklist, including union
members, preserving the distinct map-only and callback-environment traversal
rules. Absent callback construction also replaces recursive callback return-chain
expansion with parent frames and bottom-up cache publication. Field order,
nullish member selection, original declaration preference, materialization order
and spans retain their existing rules.

The initial subsystem version passes 295 analysis tests and 35 native composition
cases (26 `nullable`, nine `empty`). The follow-through audit then found and
migrated absent callback result recursion. Final source passes 296 analysis tests
and reruns all nine affected `empty` native cases. The initial 26-case nullable
run is preserved with its source identity; it is not relabeled as a final-source
run. Both whole-project formatter runs change zero files or fingerprints.

New normal-stack witnesses cover 16,384 empty record layers, callback expansion
through a nullable record, unchanged map-only callback traversal, preservation of
left nullish members, and 4,096 absent callback return contracts with cache reuse.
SQLite evidence uses `tsrust-nesting-nullable-environment-*`, with separate initial
and final source/check records. These are correctness witnesses, not measured
support ceilings or formal compilation scaling results.

No public contract or IntelliSense change is introduced; consumers remain
unchanged. The wider audit, result discovery and statement emission remain open
before obsolete checker guards can be retired. Full compiler architecture,
scaling, optimization and S12–S21 acceptance remain unfinished.

### Source-scope result discovery scheduler

`analysis/discovery_work.rs` replaces recursive discovery through blocks, branches,
loop bodies/updates, iterators and exception regions with explicit tasks. It owns
one cloned source body, moves ordinary nested bodies into tasks, and preserves
mutable root-context effects while nested scopes use independent context clones.
Leaf expression/assignment discovery retains its existing ownership checks and
pending-result restoration. Branches still choose the first concrete result and
retain absent fallbacks in the same order.

Exception tasks preserve cleanup-first exploration, abrupt-cleanup precedence,
catch payload refinement, the existing discovery restart after refinement, and
body/handler/cleanup candidate preference. Body and handler are both explored
where previously required, even if body discovery finds a concrete result.
The checked-region preparation helper now returns refined source instead of
recursively starting discovery itself.

All 298 analysis tests pass, including 16,384 source scopes on normal stacks and
a deep diagnostic witness that retains root declarations, excludes child bindings
and restores return permission and analyzer depth. The compiler integration batch
passes 80 tests: callable results six, exception handling 21, exception return
moves 34, named result inference three and recursive callable results 16. Seven
editor tests pass: exception handling six and nullable callables one. Whole-project
formatting changes zero files or fingerprints. SQLite evidence is recorded under
`tsrust-nesting-discovery-work-*`.

The inherited discovery entry guard remains for synchronous checker re-entry;
queued source scopes no longer consume recursive entries or increment that depth.
This is not final guard retirement. Expression/closure/default and region-check
re-entry must be audited together with the wider checker before removing it.
Parser integration, statement emission, full normal-stack/scaling acceptance,
optimization and S12–S21 remain unfinished. No new cap, stack increase, formal
scaling measurement or consumer refresh is claimed.

### Closure default and parameter-pattern suspension

Closure preparation now yields a child context before checking initializers.
The shared checker driver queues each initializer and lowered parameter pattern
in declaration order before entering the closure body. Parameter metadata,
nullable callback access, default availability/flow joins, capture restoration
and diagnostic spans retain their existing completion boundaries. Pattern tasks
restore extracted statements before propagating errors.

Scheduled expression and block frames no longer increment or reject the legacy
combined analysis depth. Those frames live on the explicit worklist, so their
nesting does not represent recursive checker entry. The initial analysis batch
passed 298 tests but two new witnesses hit that obsolete guard; the repaired
batch passes all 300, including 257 nested closures connected through defaults
and the corresponding deep error/unwind witness on normal stacks. Witness sizes
are not support ceilings. The affected compiler integration batch passes 53 tests
covering optional/default parameters, captured defaults, destructuring, callable
ownership, initializer facts and generic default scope.

Compatibility closure preparation and named-function context setup still have
synchronous adapters. Result discovery, pending region functions, emission and
parser integration require further connected work. This checkpoint does not
claim full architecture acceptance, formal scaling acceptance, optimization or
S12–S21 completion. No stack size or depth constant is increased. Evidence uses
`tsrust-nesting-closure-parameters-*` in SQLite; consumers remain unchanged.
Three editor tests also pass, including executable default-parameter examples,
signature/help contracts and visibility of earlier initialized parameters.
Whole-project formatting changes zero files or benchmark fingerprints.

### Contextual expression preparation

The checker driver now suspends mutation-footprint access, flow projection,
borrowed-union expectations, nullable injection and contextual stored-callable
conversion. The frame owns completion metadata while its child owns the entire
expression, restoring that expression before success or error completion.
Projection suppression and retained-flow boundaries unwind even when the child
fails. Conversion completion preserves physical tags, scoped record identities,
read-only callback environments, once wrappers and absent callable results.

The mixed witness connects nullable parameter expectations, default calls and
closure bodies through the same worklist. Optional-chain lowering, JSON-specific
preparation, generic staging and other synchronous helper entry points remain
open; this change does not establish complete expression preparation safety.
Full compiler architecture and formal compilation scaling acceptance remain
pending, as do optimization and S12–S21. No new cap or stack increase is used.

All 301 analysis tests pass on normal stacks. The affected compiler integration
batch passes 45 tests: nullable callables 24, nullable caller callback stores 13,
nullable narrowing four and nullable unions four. These include native ownership,
physical storage, allocation and disposal assertions; they are correctness checks,
not formal timing measurements. Evidence uses `tsrust-nesting-contextual-prelude-*`
in SQLite. Installed consumers remain unchanged while architecture acceptance is
incomplete.
The nullable-callable editor contract/example test passes. Whole-project
formatting changes zero files or benchmark fingerprints.
