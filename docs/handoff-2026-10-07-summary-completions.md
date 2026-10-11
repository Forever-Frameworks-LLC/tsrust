# Handoff: separate callable summary completions WIP

Read AGENTS.md, docs/development-workflow.md, docs/plans/captured-call-effects-plan.md
and docs/handoff-2026-10-07-provenance-state.md first. This checkpoint implements
separate completion graphs and standalone ordered state summaries. It does not
apply them to actual call sites or accept Gate173. Copied consumers remain 399a1c4.

summary_graph preserves distinct normal, returning and throwing terminals, while
existing diagnostic graph keeps terminal zero. Cleanup routing passes controls for
return-to-throw and throw-to-return overrides and caught failures. Shared node_flow
applies operand effects before destination commits. Summary flow builds local slots,
walks finite state joins and retains separate exit states and call-signature facts.
It preserves saved callees before argument writes. Real invocation dispatch still
uses the old Task::Call behavior; transitive and recursive effects remain absent.

Normal-edge selection now uses known predicate truth, fixing dead literal-guard
writes entering source contract sets. Both uninstantiated generic guard controls
and editor diagnostics pass. Native specialization still rejects replacement between
generic named callable environments with callable-origin. Retain the new red
literal_guard_native_callable_replacement_has_common_carrier in
crates/tsrust-compiler/tests/generic_call_capture_effects.rs. Its expected native
result is 7 for both branches. The accepted-consumer source-flow red and original
native failure, harness correction and newly named red remain durable evidence.
The earlier unannotated conditional-arrow carrier red remains required too.

Twenty-one focused unit tests pass. The 187-test bounded compiler batch passes all
eight unfiltered canaries and existing successful ownership/lexical controls while
explicitly skipping invoked_capture_writes_reject_invalid_reachable_calls. It ran
before adding the native carrier red; after separating that red, the affected source
guard test passes again. Whole editor passes 329 tests. Node independently verifies
guard results. No end-to-end generic replacement acceptance is claimed. Formatting
changes zero source files and hashes. Closed logs are imported/read back in SQLite
and removed; completed affected test executables are pruned.

Next wire summary invocation after arguments using saved callee origins and supplied
argument Values. Index needs parameter/frame metadata; project only captured writes
back into caller frames and keep fresh recursive locals separate. Use finite
inter-call worklists and contextual provenance instead of recursive Rust dispatch
or textual write unions. Keep normal and exceptional exits distinct. Return values
must retain evaluation snapshots across finally; the current summary runner only
tracks binding effects and signature facts. Closures and latent body writes still
select conservative root fallback. Collection callback cardinality, optional calls,
stored fields and the common callable-carrier dependencies remain required work.

Preserve fifteen owner fixtures; use explicit Git paths. Reuse the warm target with
CARGO_INCREMENTAL=0 and offline locked Cargo. Preserve passed bounded evidence and
repair only affected groups. Keep the full plan active, without timing or consumer
refresh claims for this WIP.
