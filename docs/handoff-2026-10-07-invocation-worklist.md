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

Next connect eligible root validation to the converged engine facts. First handle
unsupported/opaque dependencies conservatively: a known opaque body must not become
an effect-free call or silently lend an unsafe proof to its parent. Default/rest
arguments must be materialized in order before contextual body entry; current
invoke helpers require that precondition but Engine does not implement it. Unknown
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
