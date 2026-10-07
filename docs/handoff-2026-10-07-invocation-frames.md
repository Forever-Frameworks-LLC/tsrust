# Handoff: contextual invocation frames WIP

Read AGENTS.md, docs/development-workflow.md, docs/captured-call-effects-plan.md
and docs/handoff-2026-10-07-summary-completions.md first. This checkpoint adds
formal parameter metadata and contextual frames around standalone summary flow.
It does not wire actual call-site dispatch or accept Gate173. Copied consumers
remain the accepted 399a1c4 image; the warm Cargo binaries may contain newer WIP.

Index.parameters is keyed by the same original declaration span as body lookup.
It retains Binding slots, initializers and rest flags for named functions and real
arrows, including default-arrow bodies. Synthetic named-value forwarding wrappers
remain excluded. Formal contracts and actual callback bodies stay independent.

invocation_frame::Frame masks the callee's parameter/local BindingIds from incoming
state, installs supplied argument origins and signature targets, and initializes
locals as unknown. Its projection restores suspended caller-owned slots and copies
only outward binding effects by identity. It indexes outgoing slots once. Tests
cover recursive re-entry with different callback bodies, matching local names in
caller and callee, unknown omitted bodies, arrow parameter metadata and rest slots
remaining unknown rather than pretending their first argument is the whole array.

summary_flow::invoke connects one contextual frame to the standalone summary runner
and projects normal, return and throw states separately. A supplied callback body
flows through its parameter into a captured caller assignment. The helper requires
already evaluated defaults; it does not execute initializers or pack rest arrays.
Unknown optional/default/rest cases must remain required dispatcher work, not be
silently treated as effect-free. Return-value completion snapshots also remain open.

Twenty-four focused tests and 187 bounded compiler tests pass, including all eight
unfiltered depth canaries. The compiler command explicitly excludes
invoked_capture_writes_reject_invalid_reachable_calls and
literal_guard_native_callable_replacement_has_common_carrier. The conditional-arrow
common-carrier red remains required too. Affected editor intelligence and stdio
suites pass 197 tests; full editor was not rerun for this metadata-only checkpoint.
The preceding whole-editor 329-test record remains in SQLite. Run full editor before
accepting the actual invocation implementation. Formatting changes zero sources and
fingerprints. Closed logs are imported/read back in SQLite, then deleted. Completed
affected test executables are pruned. No new observation, timing or consumer refresh
is claimed. All fifteen owner fixture edits stay uncommitted.

Next implement explicit finite inter-call worklists. Save callee Values before
arguments, retain supplied argument Values, prepare ordered default/rest contexts,
then apply known normal and exceptional summaries after arguments. Carry callee
signature facts back to source validation. Do not recurse through Rust evaluator
calls or join all textual writes. Preserve caller frames, restoration and cleanup
routes. Contextual capture instances, return values across finally, optional call
regions, stored callable fields and collection cardinality remain required coverage.
Closures and latent writes still select conservative root fallback. Repair retained
native and conditional carrier failures before broad callable acceptance.

Use explicit Git paths, one warm Cargo target, CARGO_INCREMENTAL=0 and offline
locked Cargo. Preserve completed evidence and supplement only affected checks.
Keep the full language/editor plan active.
