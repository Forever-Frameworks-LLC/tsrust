# Handoff: B7 captured installer receivers, native

Date: 2026-10-10. Continues `docs/handoff-2026-10-09-b7-captured-receiver-source.md`.
Plan: `docs/plans/caller-borrowed-record-stores-plan.md`. Method:
`docs/handoff-2026-10-09-b5-continuation.md` section 4.

**B7 (gate 183) is not complete.** This checkpoint makes one form native: an installer
closure called through another closure that captured it. Other B7 plan rows and D2
remain open. No timing run occurred.

## What changed

The witness `captured_installer_receivers_preserve_native_hidden_sources` is green.

1. **Projected data lifetime.** `closure_record_signature_emit.rs`
   (`projected_capture_lifetime`) walks the capture path through the nested capture
   layouts. A borrowed capture spends its first lifetime slot on the reference; the
   captured environment's own slots follow in its layout order. The outer closure gets
   the same record-view signature and `C: From<…>` bound as the installer it calls.
   Only a shared data reference at the end of the path is resolved. Any other shape
   returns no signature and keeps the conservative path.
2. **Distinct owner inputs.** `borrowed_owner_emit.rs` gives every projected source its
   own owner input (`projected_owner_key`, keyed by the position of its captured
   borrow, outside binding IDs). `receiver_owner_marker` names the owner of what a
   closure value retains: through the construction context when the value is local, or
   through the current closure's own owner input when the value is held.
3. **Owner provenance through held values.** `borrowed_owner_provenance.rs` builds an
   `Environment` for a closure where it is constructed and passes it to the callee
   that receives the value. A capture's `outer` binding is never resolved in a foreign
   function. A call through a held value with no known environment marks every origin
   that callee installs as incomplete, so no owner marker is invented.
4. **Static guard.** `closure_owner_uses.rs` now treats only a written parameter as an
   unknown receiver. Capture bindings carry the `parameter` flag too; the guard uses
   the closure's parameter list instead.
5. **Reification.** `record_callback_reification_emit.rs` follows the projection in
   the caller before it compares owner markers.

## Defects found and fixed

All of these existed at `4c120ea5`. Earlier checkpoints ran subsets and missed them.

- **Unsound release for nullable callback records.** After a store or a clear through
  a `Mutable<Record>` parameter, source analysis released the caller's captures for a
  field such as `saved: Holder | undefined`, but generated Rust kept the borrow
  ("Rust rejected generated code"). `record_call_capture_release.rs` no longer descends
  into a record member of a union. Nullable callable fields keep their release.
- **Two sources through one installer closure.** The callback trait implementations
  used the unit owner for every owner input, so two replacement bounds differed only
  in a lifetime and were ambiguous. `unnamed_owner_markers` gives several owners
  distinct markers.
- **Unbounded instantiation.** A recursive call that wrapped its own borrowed callback
  parameter in a new closure was accepted and Rust reached its recursion limit. The
  call is rejected again with `callback-escape`.
- **Unstable diagnostic.** The view conflict for a callback alias named whichever alias
  a hash map yielded first, so the library and the command line disagreed. The earliest
  holder is named, and the message now describes a callback alias instead of an
  optional element view.
- **Stale rejection rows** in `callback_moves`, `callbacks` and `nested_closures`: a
  local closure can now borrow a callback parameter, directly, through a stable alias
  or from a nested closure. Each form was checked natively and against Node and became
  a positive witness.

## Evidence

- Whole compiler crate: 2,696 passed, 2 failed, 3 ignored
  (`tsrust-b7-projected-owners-whole-compiler-final.log` in SQLite). Both failures came
  from the new recursion guard also firing for owned callbacks, where an existing check
  reports `unsupported`. The guard was narrowed to borrowed callbacks and the two
  suites (`recursive_owned_callbacks`, `recursive_callable_results`) and the four
  callback suites were rerun and pass. The whole crate was not rerun after that
  one-condition change.
- Whole editor crate: 451 passed, 0 failed
  (`tsrust-b7-projected-owners-whole-editor-final.log`).
- The first whole run of this session, with five failures that all reproduce at
  `4c120ea5`, is kept as `tsrust-b7-projected-owners-whole-compiler.log`.
- New compiler witnesses in `tests/caller_input_instance_replacement.rs`:
  `captured_installer_receivers_cover_plain_fields_and_three_layers`,
  `one_installer_receiver_retains_two_independent_hidden_sources`,
  `captured_installer_receivers_keep_live_sources_borrowed`.
- Editor witness: `captured_installer_receivers_release_replaced_hidden_sources_in_the_editor`.
- Workload `caller_borrowed_record_stores`: 72 observations per iteration (was 68).
  The new scenario installs through a held installer closure twice and observes the
  first value, the released source, the replacement and the effect count. Oracle,
  source contract, registration controls and manifest hashes are updated; native and
  Node-after-tsc verification pass, untimed.
- Guide: "Installers called through another closure" in
  `docs/scoped-callable-transport.md`. The example runs natively and on Node.

## Still open in this form

- A mutable captured source through a held receiver: `projected_capture_lifetime`
  resolves only a shared reference. It needs the receiver reborrow lifetime, as direct
  mutable captures use `'record_receiver`.
- A projected source reached through a *parameter* of the closure
  (`Borrow::source_projection` on a written parameter) still has no native signature.
- A closure value constructed in one function and held in another through a parameter
  has no environment; its installed origins stay without owner markers.
- Opaque projections keep conservative receiver retention.

## Not done

- No timing. The workload keeps `measured: false`.
- B7 rows listed in the plan: recursive relay summaries, branch and loop stores, early
  exits, failure paths beyond the covered ones.
- The five installed consumers stay at the accepted B5 build (`00214a11`), as the
  previous handoff requires. They were refreshed from this unaccepted checkpoint by
  mistake and then rebuilt from `00214a11` and restored. Refresh them only at B7
  family acceptance.
