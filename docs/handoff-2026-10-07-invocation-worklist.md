# Handoff: finite inter-call worklist WIP

Read AGENTS.md, docs/development-workflow.md, docs/captured-call-effects-plan.md
and docs/handoff-2026-10-07-invocation-frames.md first. This checkpoint adds dispatch
hooks and a standalone finite inter-call engine. Compiler validation still uses
UnknownDispatch and the existing branch/conservative paths. Gate173 remains open.
Copied consumers remain 399a1c4; warm Cargo binaries can contain newer WIP.

expression_flow::Dispatch receives the saved callee after argument evaluation,
actual argument Values and the resulting caller state. Invocation carries optional
normal state, throwing states, result Value and nested signature facts. Pending or
definitely throwing known calls have no normal exit; evaluation stops remaining
expression tasks and statement commits. Node/summary evaluators expose dispatch
variants while retaining their existing UnknownDispatch wrappers.

invocation_worklist::Engine keys requests by original declaration, caller bindings
and argument Values. It uses a hash lookup, cached finite outcomes and reverse
request dependencies. A request starts at bottom, not an invented effect-free exit.
solve iterates requests; Dispatch::invoke only queries/enqueues, so recursive calls
do not recurse through Rust evaluator frames. Distinct completion states and source
signature facts join monotonically. Standalone tests cover seven original scenarios
(named, arrow, transitive, recursive, conditional, caught throw and argument write),
cache reuse and a supplied-false parameter excluding a guarded captured write.

Selection facts (truth/nullish/undefined) now accompany binding provenance. Frame
parameters receive argument selections, projection preserves captured changes and
joins discard disagreements. Unknown dispatch invalidates mutable selections. Scope
and origin/contract metadata gained structural equality/hash support for cache keys;
Span gained Hash without changing its layout or equality behavior.

Twenty-eight focused tests, 187 bounded compiler tests and 197 affected editor
intelligence/stdio tests pass. All eight depth canaries ran unfiltered. The compiler
command explicitly excludes invoked_capture_writes_reject_invalid_reachable_calls
and literal_guard_native_callable_replacement_has_common_carrier. Neither is fixed
in the actual compiler path. The conditional-arrow common-carrier red remains too.
Full editor was not rerun for this WIP; the earlier 329-test record is preserved and
full editor must run before actual invocation acceptance. Formatting changes zero
files and fingerprints. Closed red/corrected/final logs are imported/read back in
SQLite, then removed. Completed affected test executables are pruned.

A follow-up fixes opaque propagation. Missing known bodies and unknown callee
alternatives mark the current request opaque. Reverse dependencies propagate this
status transitively, including cached roots. The status before body evaluation is
retained so a dispatch-side change schedules dependents. A focused red witness
now passes for a missing callback body through relay and outer functions, repeated
cache lookup and an unknown callback. All 29 focused tests and all eight depth
canaries (63 tests) pass. No editor behavior or compiler-path activation is claimed.

Next connect eligible root validation to the converged engine facts. Continue handling
unsupported regions conservatively; opacity currently rejects the summary proof. Default entry now lives in generic_call_parameter_entry.rs. It resets formal slots,
installs supplied values in order, executes omitted/undefined defaults, joins both
branches when undefined status is uncertain and carries failures before body entry.
A missing required argument retains undefined for source diagnostics; ordinary
arity validation remains responsible for rejecting the call. Rest parameters now pack trailing argument snapshots into flat element provenance.
Constant integer indexing preserves the selected callback. Arrays and aliases carry
these snapshots through bindings and frame projection. Unknown indices, nested
aggregate elements and element writes remain conservative; array aliases have no
storage identity yet. Mutation invalidates all element snapshots. Implement precise
mutation, spread, iteration and return snapshots before full aggregate acceptance. Unknown
callee alternatives and callbacks passed to unknown/built-in callees still need
proper effects/cardinality handling. Do not activate these cases by pretending they
have no effects. Collection callbacks, optional regions and stored fields remain
required work. Return result Values are currently unknown; preserve evaluation
snapshots across finally before claiming factory/returned-callback consistency.

The actual root validator still rejects closures/latent writes on its precise path,
then uses conservative fallback. Integration should repair the original retained
reds while preserving restoration, saved callees, shadows and uncalled declarations.
Native carrier failures must also be repaired before broad callable acceptance.
Preserve all fifteen owner fixture edits, use explicit Git paths and one warm
non-incremental offline locked Cargo target. No new observations, timings or
consumer refresh is claimed. Keep the full plan active.

## Ordered default entry WIP

Thirty focused tests pass, including omitted defaults, explicit undefined, supplied
values bypassing defaults, references to earlier parameters and throwing defaults
that modify a captured callable before catch. Pre-analysis undefined names now
retain undefined selection when unshadowed. All eight unfiltered depth canaries
pass (63 tests). The first implementation exposed the need to preserve diagnostic
facts for invalid calls with missing required arguments; corrected evidence is
retained alongside the original red. No editor acceptance, native parity, root
activation, timing or consumer refresh is claimed. Rest arrays, return snapshots
and original integration/carrier reds remain open.

## Flat rest/array element provenance WIP

The finite domain stores a fixed vector of flat element facts, with no recursive
aggregate shape. Equal-length joins merge each position; differing shapes become
unknown. The vectors come from source array literals or source call argument lists.
Rest packing and constant indexing preserve captured callback bodies. Witnesses
cover rest invocation, literal arrays, aliases, separate indices and mutation
invalidating aliases. Dynamic indices, nested aggregate elements, spread and
collection iteration remain unsupported by this proof. No compiler-path acceptance
or editor/native parity is claimed. Thirty-one focused tests and all eight depth
canaries (63 tests) pass; exact red and passing evidence is retained in SQLite.
