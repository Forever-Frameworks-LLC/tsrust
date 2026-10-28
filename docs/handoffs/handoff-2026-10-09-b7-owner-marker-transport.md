# B7 native owner-marker transport checkpoint

B7/gate 183 remains open. This checkpoint implements the previously red effectful same-origin callback installer. D2 discriminated unions remain a separate feature, intended to support later schema validation. No owner question is pending.

## Implemented behavior

Two independent instances from the same callback factory can replace one another through an ordinary effectful installer. The overwritten callback becomes available while the holder remains live. Installer effects execute exactly once. Named relays preserve the caller owner through a stable borrowed alias. Replacing the holder again can return to the original instance without resetting its state. A closure can capture the resulting holder and invoke its current callback.

The existing direct data-input replacement cases still work. A local forwarding closure keeps its original type until it is stored. Its storage conversion attaches the native marker, avoiding the generated-type mismatch exposed during this work.

## Representation and modules

`borrowed_owner_provenance.rs` is now used in production. It grounds concrete named storage owners and propagates whole correlated vectors through named calls, including recursive relays. It never invents another owner for a repeated call site. Incomplete grounded contexts, closure-body callers and installer function values suppress marker specialization. An empty retained-store summary exits immediately.

`borrowed_owner_emit.rs` selects callback-type collisions, emits zero-sized marker types and phantom marker parameters, propagates caller markers, wraps retained payloads and supplies expanded native lifetime alternatives. Its planning helper is outside recursive dispatch.

`borrowed_payload_emit.rs` correlates owner vectors with concrete callback vectors. `borrowed_callable_emit.rs` dispatches through the wrapper, including consuming calls. `record_parameter_borrow_emit.rs` names the corresponding generic tagged replacement types. Nullable and plain record storage conversions attach markers at the store. `record_callback_reification_emit.rs` uses both actual owner and callback type after the existing all-normal-exits release proof.

`scoped_record_emit.rs` preserves tagged payload types inside captured records. `scoped_record_capture_views.rs` uses the expanded lifetime alternatives. The nested-holder witness originally panicked because those slots were counted using only callback types; both counts now agree.

There is no source ownership change, implicit clone, shared cell, dynamic dispatch or lifetime cast. Five installed consumers remain at B5 until family acceptance.

## Verification and durable evidence

The affected integration evidence covers 80 cases across eight suites. The final storage/conversion batch reruns 32 cases after the local-forwarding repair; the other 48 passed in the preceding batch and are unaffected by that repair. All 38 borrowed/provenance unit checks pass. The independent factory editor witness now includes installer effects and passes hover, diagnostics and old-input conflict checks. All eight final depth canaries pass. No whole-crate regression or timing ran.

The workload now has 32 observations. Its new result observes the two effectful relay installations. Independent arithmetic oracle, source contract, mutation rejection, registrations and manifest hashes are current. Whole-project formatting runs twice with zero changes. `runtime/results/2026-10-09T20-02-23-127Z-verify.json` verifies five cases on native and Node-after-tsc, with no timing samples. SQLite preserves exact compiler/workload inputs and generated text under `runtime/source-evidence/owner-marker-transport/`; the result is imported and read back byte for byte. Logs use `tsrust-b7-owner-transport-*`, with the local forwarding red retained separately as `tsrust-b7-local-owner-wrapper-red.log`.

## Open work

Markers currently cover named callback inputs whose native callback types collide. Equal-type direct array/record captures remain on the previous direct-assignment representation; effectful data installers still need native singleton-family transport. Field/index inputs, computed temporaries, closure-body caller contexts and installer function values need complete provenance before enabling markers. Do not delete these guards to make a witness compile.

Next add effectful equal-type data replacement reds, then implement their native family representation. Keep old-input reuse while the holder remains live and verify body effects. Continue with closure-caller provenance, retained branch/loop/early-exit behavior, disposal, recursive stores and nested generic environments from the B7 plan. B7 family acceptance still requires complete regressions, matched timing review and consumer refresh. The full compiler plan remains active.

Preserve the fifteen unrelated fixture edits. Use explicit Git paths, one warm Cargo target, incremental compilation disabled and Node 24.11.1 first on PATH. Use Homebrew Python for SQLite. Preserve passed evidence instead of restarting full batches. Rebuild installed consumers only at family acceptance.

## Continuation prompt

Continue Tsrust from this checkpoint. Read AGENTS.md, docs/development-workflow.md and docs/plans/caller-borrowed-record-stores-plan.md. B7 owner markers now support effectful equal-type callback factories through named relays and nested holder captures. Implement the remaining retained data and closure-caller composition, beginning with focused failing native witnesses. Preserve ownership assertions, effects, SQLite evidence and unrelated fixtures. Keep D2 discriminated unions separate. Do not treat this checkpoint as B7 or compiler completion.
