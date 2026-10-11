# Handoff: callable provenance state WIP

Read AGENTS.md, docs/development-workflow.md, docs/plans/captured-call-effects-plan.md
and docs/handoff-2026-10-07-body-provenance.md first. This checkpoint wires actual
body origins through source binding and expression states. It does not accept
Gate173 or execute invocation summaries. Accepted copied consumers remain 399a1c4.

Binding.origins is separate from Binding.contract. Initializer collection records
real bodies even with callback annotations. Statement and expression assignments
update origins regardless of declared signature; fixed signatures remain fixed.
Expression Value carries origins, and assertions preserve evaluated snapshots.
Branch joins canonicalize finite origin sets including unknown alternatives.
Straight-line fallback snapshots aliases before later reassignment. Conservative
fallback accumulates matching writes by lexical identity without turning them into
executed effects. Parameters have unknown bodies until contextual arguments arrive.

body_origins::resolve follows the same BindingId lookup used by callable contracts.
Unknown captured frames cannot fall back to a global name. Its explicit enter/leave
stack detects cycles per path and retains unknown alongside known alternatives;
shared acyclic alias paths remain known. Unknown Value defaults never disappear
from a join merely because no signature is available.

Fifteen focused metadata tests pass, including annotated aliases, assignment through
assertions, saved snapshots, branch joins, cycles and straight-line typed writes.
The 186-test bounded compiler batch passes all eight unfiltered depth canaries and
successful lexical/ownership controls, explicitly skipping only the original
invoked-capture rejection test. The whole editor passes 328 tests. Both the original
eight invocation defects and the conditional common-carrier inference red remain
required work. Whole-project formatting changes zero sources and fingerprints.
Closed logs are imported/read back in SQLite, then removed; completed affected test
executables are pruned. All fifteen owner fixture edits stay uncommitted.

Next implement ordered normal and exceptional invocation summaries. Save callee
origins before arguments, retain supplied argument Values for callback provenance,
build callee-local frames with the existing Index identities, and apply only actual
known calls. Keep normal exits separate from throws and cleanup overrides. Recursive
and transitive calls need finite fixed-point summaries, not textual write unions.
The current expression Task::Call only records signature facts and returns unknown;
flow graphs still have shared terminal zero, which summaries must refine. Closures
and latent body writes can still select conservative fallback. Stored fields,
optional call regions and collection callback cardinality remain later required work.

Use explicit Git paths, one warm target, CARGO_INCREMENTAL=0 and offline locked
Cargo. Preserve completed bounded evidence and do not restart unaffected suites.
No benchmark observation, timing or consumer refresh is claimed for metadata WIP.
Keep the full plan active and repair the retained conditional carrier red as part
of callable value consistency before full acceptance.
