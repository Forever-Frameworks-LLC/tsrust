# B7 effectful data-owner transport checkpoint

B7/gate 183 remains open. Effectful named array and record installers now release overwritten equal-type owners while their holders remain live. D2 discriminated unions remain a separate feature. No owner question is pending.

## Behavior and representation

Nullable data stores retain a checked singleton borrowed callback family. Native owner markers distinguish independent data owners even when there is no abstract callback capture. Native lifetime alternatives include the singleton concrete closure environment for every tagged owner. This extends the existing callback owner representation without cloning data, changing source ownership or casting lifetimes.

The new focused cases cover nullable and plain mutable array fields, readonly array captures, effectful named relays, nested holder capture and mutable record inputs. They assert old-input reuse, the replacement's state and exact effect counts. Existing factory-instance and local-forwarding cases remain controls.

Broadening markers exposed two accepted relay regressions. A relay can install the same origin using different input parameters, so its native conversion bounds need all actual symbolic parameter vectors. `borrowed_owner_provenance::parameter_vectors` runs the finite checked call graph with formal parameters as symbolic owners. `record_borrow_replacement_types` preserves correlated vectors and filters them by the destination parameter and field path. This avoids both keeping only the first owner and attaching another destination's undeclared lifetime. The existing two-destination and repeated-store native witnesses pass again.

The relevant modules are `record_parameter_borrows.rs`, `borrowed_owner_emit.rs`, `borrowed_owner_provenance.rs`, `record_parameter_borrow_emit.rs`, `record_parameter_replacement_emit.rs` and `nullable_callback_replacement_emit.rs`. Source invocation contracts and the existing all-normal-exits release proof remain authoritative.

## Verification

All 82 affected integration cases pass across eight suites. The ten instance tests include multiple native variants for readonly arrays, plain fields and relay composition. All 38 borrowed/provenance unit checks pass. The new editor witness verifies nullable hover, released old-input access and retained new-input rejection. All eight depth canaries pass on the normal worker stack. No whole-crate regression or timing ran.

The matched workload has 33 observations. Its new data effect count is independently computed as two. Source contracts require both installer effect increments, and mutation tests reject deleting either observation or an increment. Whole-project formatting runs twice with zero changes; formatted registration passes. Native/Node result identity and durable storage are recorded below after verification.

## Remaining work and continuation

Continue with closure-body caller provenance, installer function values, field/index inputs and computed temporaries. These currently suppress marker specialization rather than fabricating owner identity. Retained branch/loop/early-exit behavior, disposal, recursive stores and nested generic environments remain in the B7 plan. In particular, probe repeated calls to installers that build conditional callback families; the generic conversion helper currently expands concrete closure payloads and must not be assumed complete for arbitrary family composition.

Keep old-input reuse while the holder remains live, new-input conflict assertions and observable installer effects. Use focused red tests before widening representations. Continue the original compiler plan after B7; do not close the goal from this checkpoint. Family acceptance still requires full regressions, matched timing review and consumer refresh. The five installed consumers remain at B5.

Preserve the fifteen unrelated fixture edits and stage explicit paths only. Use the existing Cargo target, CARGO_INCREMENTAL=0, Node 24.11.1 first on PATH and Homebrew Python for SQLite. Preserve original failed and passing evidence, import and read it back before deleting loose copies, and do not restart passed batches without a relevant change.

## Closed evidence

`runtime/results/2026-10-09T20-12-51-730Z-verify.json` verifies all five cases on native and Node-after-tsc, with empty timing arrays. The result, compiler inputs, workload sources, harness sources and generated text are imported into SQLite and read back byte for byte. Content-addressed source records use `runtime/source-evidence/data-owner-transport/`; logs use `tsrust-b7-data-owner-*`. The initial reds, failed relay bounds and repaired controls remain preserved. Disposable workload artifacts are removed after their text inputs are retained. No installed consumer is refreshed at this partial B7 checkpoint.
