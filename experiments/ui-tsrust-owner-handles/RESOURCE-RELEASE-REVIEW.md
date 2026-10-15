# B2 merge review

Reviewed candidate4810f10d against L01–L26, H/R/I reconciliation, checked adapter discovery/emission and the async/worker direction. No merge-blocking runtime defect was found. Corrected the report's emission-source reference from nonexistent `emit/rust.rs` to `emit/rust_bridge.rs`.

## Matrix assessment

- L01–L03: baseline/pins and native observation boundary are explicit. L02 is design analysis, not a thread-safety proof. The audit stores only IDs; snapshot values do not retain the resource.
- L04–L07: native move/capture/replacement/close/scope controls and negative expectations calibrate actual Drop events. Checked Tsrust equivalents remain blocked. L06 does not falsely claim teardown of an owned audit handle: its audit is process-scoped.
- L08–L20: real UI state, captures, queued payloads, active callback safety, tree ordering, automatic destruction and resource balance remain blocked. Native1,000-cycle balance is not promoted to UI balance. Queue clear alone is correctly not applicable because that API does not exist.
- L21–L22: current/historical identity, B4 and logical lifetime witnesses remain in the full focused suite; the owner is unchanged. Formatting/token/idempotence checks and unmodified emitted native execution are retained.
- L23: exported Drop rejection is reproducible in both check and emit; paired plain-data control executes. This is an intentional missing capability, not a compiler crash or failed package download.
- L24–L26: all H/R/I dispositions reconcile, B1/B3/B4 remain resolved within their original contracts, and demo exit2 cannot be confused with full resource acceptance. Original isolated handoff restrictions remain historical; user now authorizes merge and roadmap priority.

## Blocker assessment and edge cases

`project/rust_data.rs` rejects the owned Drop boundary. `project/rust.rs::shape` and public-field requirements exclude an existing private opaque owner/returned closure substitute. `emit/rust_bridge.rs::rust_data_output` builds a new source record from native fields; `rust_input` reconstructs a new native record. For copyable fields, simply allowing Drop would destroy the original resource during conversion while the projected record survives. Moving non-Copy fields out of a Drop type is also not a general solution. The guard must remain.

An integer resource registry with explicit cleanup would mask automatic destruction. Private Rust-local guards only observe the native call's scope. Existing File operations cannot establish non-retaining release counts and ordered events. None provides an easy valid replacement. The compiler worker independently confirms S18's native SQL resources do not generalize this bridge and recommends the priority dependency rather than concurrent opaque-resource changes.

The observer's public ID field permits deliberate native fabrication, and its global log grows until process exit. Those are documented test-only limits, not UI capabilities. Tests use fresh bounded processes and only the allocator; no process/thread identity guarantee is inferred. Unexpected compiler/native failures still fail validation. Missing/changed pinned libraries are tool-provenance failures, not evidence of B2.

## Async/worker direction

The compatibility table preserves serialized owner mutation without requiring all application work to be single-threaded. It distinguishes retirement, active-use completion and physical release, rejects relying on callback return as a universal async safe point, and retains explicit late-completion, cancellation/shutdown, affinity and transport dependencies. Current module-scalar runtime allocation is not certified for concurrent construction. There is no claim that a release event or dropped future proves async backend cleanup completion.

The roadmap now makes checked resource lifetime support a compiler priority. UI pauses until that dependency is accepted. Automatic owner finalization/order and async/thread execution still need their own tests after the observer can cross the checked boundary. This review does not approve an untested synchronous-only foundation as a production UI architecture.

## Review checks and integration

Review rerun:16 tests passed,0 failed,0 skipped. Native demo exits2 for B2. An additional native field-projection illustration confirms the underlying resource is released while the projected scalar survives; it is explicitly a native representation illustration, not patched compiler output or UI acceptance. Source correction and priority documentation add no runtime behavior.

Whole-main formatting changed0 files/fingerprints after the compiler worker's formatting handoff. Dirty/untracked S18 and owner files plus their index entries were fingerprinted for exact preservation during the disjoint merge. No consumer refresh or S18 source checkpoint is included.

Original17 B2 evidence records and this review's closed evidence are integrated under `runtime/diagnostics/ui-resource-release/4810f10d1bb10d8de5c7d369a3f68e1a07b0ea6d/` in canonical `benchmarks/records.sqlite`. Verify imported bytes and database integrity while preserving prior records and analytical tables. Historical absolute record paths elsewhere in that database remain outside this merge's scope. UI is paused pending the owner-prioritized compiler dependency in the roadmap.
