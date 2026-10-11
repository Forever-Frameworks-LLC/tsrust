# Handoff: typed named bodies before capture lowering

/caveman Continue tsrust at /Users/seancotter/Desktop/dev/learning/tsrust. Read AGENTS.md, docs/development-workflow.md, docs/plans/nested-function-declarations-plan.md and docs/milestones/171-nested-function-captures.md first; inspect current Git status/checkpoint.

Gate171 remains incomplete. Retention checkpoint542069e preserved capture bodies/plans/logical owners/statement anchors. The source-readiness checkpoint added local_readiness.rs at Analyzer::new. The current environment guard follows typed body checking, with unresolved-context and callable-value guards as described below. Required binding identities propagate once over condensed groups. Plan.owners maps logical owners to function names; Plan.uses maps variable expression starts to BindingId. Project linker qualifies owner names and rebases keys/identities/spans. The definite-assignment adapter recognizes callable aliases for each captured binding and preserves ordinary variable behavior. Only a whole-name assignment to the resolved binding initializes it; shadow/indexed writes do not. Branch/return/loop/catch/finally behavior reuses existing flow.

The original early-call diagnostic now passes, marking the named-function use with a related captured-binding location. Transitive forwarder and shadow-assignment error witnesses pass too. Existing arrow borrowing native reference passes. Four positive native named-capture cases still fail at the environment guard. A fifth pending witness, checked_call_failure_keeps_capture_unassigned_on_the_catch_path, requires typed call-failure effects (fail() throws before value assignment then catch calls read). It remains genuinely red. Source precheck cannot replace final typed readiness/effect/control-target checks. Never turn the positive red witnesses into accepted unsupported contracts.

Source closures currently share their enclosing logical owner in lexical resolution. The precheck locates the innermost source closure containing the binding and checks its invocation body with parameter readiness, rather than treating local initialization as outer creation. This uses temporary compiler AST copies only. Typed context integration should avoid repeated copies and use actual checked lexical binding identities. No runtime owner clone or shared capture cell is introduced. Existing owner policy remains borrow locally, transfer supported owned escape later, scalar copy semantics.

Next implement typed local-function contexts and finite borrowed group environments. Resolve logical IDs/source anchors against typed defining contexts, partition group-external captures from invocation-local slots, infer existing access/storage contracts and run final readiness after typed call/error effects and control destinations are available. Then materialize only at valid use points with current loan checks; recursive peers use one finite environment and static/view dispatch, not fields containing each other. Preserve defaults/generics/source/editor identity and fresh loop/block environments. Owned escape gate172; result inference gate173. Keep the checked guard until this works, rather than allowing lifted bodies to bind unrelated globals or eagerly lowering hoisting to const-arrow declarations.

Use final milestone counts and SQLite records. Preserve four unstaged fixture edits: comma-contexts/guards.tsr, map-for-each/values.tsr, map-group-by/documented.tsr, interfaces/shapes.tsr. One agent, explicit staging paths only. Same warm Cargo target /private/tmp/tsrust-gate100-editor-target and CARGO_INCREMENTAL=0; pinned Node /Users/seancotter/.nvm/versions/node/v24.11.1/bin/node. No routine full compiler regression/timing/archives. Five consumers refreshed atomically; existing running editor server needs restart to load the new binary. Goal remains active and hourly checks remain paused.

Previous readiness checkpoint evidence: library18, lexical5, project1, canaries63, editor273 pass. Compiler
capture suite4 passes/5 pending failures as described above. Five matched lexical
cases pass untimed. SQLite runtime-result path:
benchmarks/runtime/results/2026-10-06T17-40-26-459Z-verify.json; exact red/build/pass logs
under target/editor/nested-readiness-* verified before loose-copy removal. Outputs
pruned; retain libraries and five consumers. No gate171 acceptance yet.

When investigating native materialization, keep environment formation separate from
body/type checking. Unused declarations must not borrow or transfer captures. A
borrowed environment that stays live must preserve current scope loans rather than
silently release them after each direct call. Prove conditional placement and Rust
drop order with native witnesses; a slot declared before a captured owner can have
an invalid drop lifetime even if initialized later. Do not eagerly create references
before captures are ready. The retained statement/enclosing-owner anchors are needed
for placement, and typed closure contexts should replace source-body copying.


## Editor follow-up

The interface type-reference highlighting repair is complete: annotations, aliases,
array elements and nested generic arguments have type scopes. Real TextMate tests
and extension compilation pass; the local VSIX is installed with exact source
verification. Reload the VS Code window to activate it. See the gate 166 milestone
for evidence. Resume the typed capture-context work described above; this editor
repair does not accept gate 171. Preserve all four unrelated fixture edits.


## Typed projection follow-up

Ordinary arrow closure checking now calls analysis::capture_context::project.
Context::source_bindings records declaration start to typed binding slot at bind;
projection carries only selected identities into its inner slot numbering. This
separates hidden source bindings from active lexical names without repurposing the
supplied-slot declaration_span field. Capture view roots use a lookup map instead
of repeated visible-vector searches. Existing outer call ABI and flow state are
still discarded exactly as the former closure setup required.

Use this seam when resolving named capture plans against typed defining contexts.
Keep the pending guard until checked bodies, final typed readiness and finite
borrowed layouts work. The current change does not execute captured declarations.
Do not lower them to eager arrows or change current lexical loan lifetimes.

Final typed projection checkpoint: library20, canaries63, editor273 and matched
lexical cases5 pass. Captures remain4 passing/5 pending; lexical foundation remains
5 passing with two pending future-stage tests. SQLite retains15 exact logs/results,
including 2026-10-06T17-57-50-610Z-verify.json. Five consumers refreshed; 31 example
directories and51 scratch executables pruned. Gate171 remains incomplete. Continue
by resolving planned named bodies against typed defining contexts, then final typed
readiness and finite group environments. Preserve all four unrelated fixture edits.


## Current typed named-body state

named_capture_context.rs now prepares typed direct-call contexts from planned source
binding identities. Contexts::new uses shared Plan::group_requirements for request
classification; actual used bodies select external slots on demand. Slots follow
reserved source parameters, and Context::named_capture_range identifies them for
existing access rules. Only direct capture names enter body visibility; transitive
slots remain addressable by source identity without replacing source parameters.
Read, mutable array, mutable scalar and captured-default contracts are checked in
focused unit scenarios. Public bad-field diagnostics pass through caller shadowing,
forwarders and linked private types with exact original source ranges.

Analyzer::new still performs the source readiness precheck, but no longer rejects
all captures before types exist. Analyzer::run now retains plans in a private provisional CheckedProgram. Public
analysis runs typed readiness before the environment guard returns an error. Missing defining contexts and captured
callable values still guard individually. No captured tree reaches emission, and
no declaration creates a runtime loan in this preparation. The source call ABI
remains the parameter_count prefix; captured Binding.parameter flags currently serve
access inference only. When exposing completed layouts, keep captures out of public
source parameter contracts and projections.

The next stage must retain typed capture/access metadata as finite recursive group
layouts, include capture coordinates in the recursive access lattice, and form
borrowed environments only at ready uses. Reuse existing lifetime/loan contracts;
prove mutable reentry, scope release, block/loop freshness and drop order. The final
typed readiness adapter must observe checked call failure effects and destinations;
source prechecks cannot satisfy the pending fail()/catch witness. Keep the emission
barrier until these pieces work. Captured callable transport is still pending, and
owned escape/result inference remain gates172/173.

Typed checking also exposed ordinary shadowing defects. Block entry keeps a narrow
journal of shadowed names and restores them at exit. callable_scope_depth prevents
function-body locals from redeclaring parameters. Arrow parameter reservation uses
its own name map, then restores inherited visibility for default checking. Source
slots and ownership remain independent. The matched lexical workload now has seven
observations, including shadow/restoration and arrow parameter independence; both
sources, independent oracle, guard, manifest fingerprints and coverage exports are
current. No measurement protocol or iteration count changed; no timing was run.

Current evidence: 21 focused library tests, five module/capture checks, 63 depth
canary checks, 274 editor tests, one registration test and five
seven-observation lexical cases pass untimed. The capture suite has six passes and five pending failures; the lexical suite has
five passes and two future-stage failures. Whole editor completion preserves original
passed groups, repairing stdio and resuming unrun suites. Canonical final result:
2026-10-06T18-23-58-402Z-verify.json; 107 current compiler input hashes verified. SQLite
retains 32 exact logs/results. Five consumers refreshed; 31 example directories,
52 scratch executables and runtime-comparisons outputs pruned. Libraries remain
warm, test executables will relink. No gate171 acceptance or timing. Preserve all
four fixture edits and keep hourly checks paused. Continue the full plan toward
finite borrowed group environments and final typed readiness.


## Typed exceptional readiness follow-up

The pending fail()/catch witness now reports `unassigned` at the named call, with
the captured declaration as related information. CheckedProgram retains capture
plans privately. Both public analysis paths finalize typed effects, ordinary
definite assignment and named-capture readiness before the environment barrier.
Actual checked functions, closures and methods are selected by physical declaration
spans; projected hidden slots cannot masquerade as defining declarations. No source
closure body copying occurs in this final pass. Exception emission validation is
deferred only for programs still blocked by missing capture environments.

Focused cases cover failure before assignment, failure in an assignment RHS and
failure inside a checked closure invocation. Catch assignment and finally assignment
reach the pending environment barrier rather than a false readiness error. The
editor diagnostic marks the exact named call and related captured binding. Four
positive native capture witnesses remain red; no runtime capture acceptance is
claimed. Next implement finite capture access coordinates and borrowed group
environments, preserving loans, recursion, loop freshness and drop order.

Final checkpoint evidence: matched lexical cases5 pass untimed; canonical result
2026-10-06T18-53-10-079Z-verify.json. All107 compiler source hashes are verified.
Ten exact logs/results are imported into SQLite and read back before removal.
Readiness library3, depth canaries63, editor275; captures7 pass/4 pending.


## Current recursive capture-access state

named_capture_access.rs owns Access coordinates (BindingId, hidden slot, Mode) and
a monotone Lattice keyed by function index plus binding identity. Contexts retains
the capture identities in projected slot order and extracts summaries after checking.
Static named edges propagate access and use metadata into typed caller capture or
parameter slots; ordinary local owners acquire no loan or transfer in this phase.
The recursive driver iterates capture and supplied-argument coordinates separately.
New capture coordinates extend its finite budget. The final private checked program
retains summaries for future group layout. No source arguments or public parameter
contracts include captures.

Forwarder mutation was genuinely red (Copy versus Mutate), and now passes. A mutual
recursive scalar-mutation case proves convergence and exact hidden slots. Runtime
environments remain absent and blocked at public finalization. Continue by deriving
finite borrowed group layouts from these settled summaries, then placement and native
dispatch with existing loan rules. Preserve recursion, lexical release, later
initialization, block/loop freshness, defaults and source identities. Do not replace
these pending requirements with ephemeral call borrows or eager arrow creation.

Capture-access checkpoint evidence: library25, canaries63, editor275 pass;
captures7 pass/4 pending. Matched lexical cases5 pass untimed. Canonical result
2026-10-06T18-58-52-478Z-verify.json has 108 verified compiler fingerprints
and no timing samples. 9 exact original records, including failed compile
evidence, are imported/read back in SQLite before loose removal. Five consumers
are refreshed atomically; 50 scratch executables and runtime comparison output
are pruned. Library caches remain; tests will relink. Gate171 is not accepted.


## Finite typed group layout checkpoint

named_capture_layout.rs derives deterministic fields and member-slot mappings from
settled capture access. Each recursive group deduplicates external BindingId values,
checks their typed slots agree and joins access modes. Read scalars use Copy storage,
mutable scalars use Mutate storage and owned values retain their read/mutate/consume
requirements. Recursive peers are static members, not fields containing each other.
Fields are ordered by defining identity, so shadowed spellings cannot merge.

Captures whose owner is a group member are explicitly Invocation sources rather than
shared fields. A recursive outer/child parameter witness verifies that a fresh
invocation-local value is not incorrectly hoisted into group storage. Another
witness checks two recursive peers share one mutable scalar field and one readonly
array field, with source parameter counts unchanged. Layout construction is called
only after final typed readiness and retained privately before the environment barrier.

This checkpoint does not instantiate native environments or remove the emission
barrier. Next implement ready-use placement and native group dispatch, with existing
lexical loans, mutable reentry and drop-order checks. Cross-group forwarding must
reborrow correctly without creating duplicate retained mutable references. Invocation
sources need explicit forwarding from their defining activation. The four positive
native witnesses remain pending. Owned escape remains gate172.

The next authoritative handoff is docs/handoff-2026-10-06-gate171-layout.md.
