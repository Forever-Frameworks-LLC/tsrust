# B7 independent captured installer owners checkpoint

B7/gate 183 remains open. D2 discriminated unions remain a dedicated future feature. No owner question is pending. Read AGENTS.md, docs/development-workflow.md and docs/caller-borrowed-record-stores-plan.md first.

## Implemented behavior

Two ordinary local installers can capture different caller-owned arrays and replace the same stored callback. After both installer blocks end, the old array can be mutated while the holder retains the replacement. Native witnesses cover nullable and plain readonly callback fields. Source and editor controls reject mutation of the new retained source. The original native witness failed with E0502 because both owners shared a callback payload lifetime; it now passes.

`borrowed_owner_emit.rs` includes hidden retained capture bindings in closure owner inputs, renders their call markers and resolves their source binding for replacement. `Scope::retained_borrows` combines written source slots and hidden capture indices for replacement selection. Hidden entries use the explicit parameter count plus capture index and are decoded only by the retained-source helper; do not interpret them as written argument slots or checked binding IDs. `record_callback_reification_emit.rs` matches the hidden source marker against the checked concrete payload and keeps explicit argument type facts unchanged. Plain fields and nullable fields use the same retained-source selection. The shared emission dispatch delegates marker construction to its helper, keeping temporary state out of recursive dispatch.

Closure classification no longer suppresses every captured retained input. It still suppresses owned capture storage, consuming wrappers, captured record storage, escaping/polymorphic/stored/aliased uses and invocation through captured or parameter receivers. Those indirect receivers cannot use declaration-context capture binding IDs as caller binding IDs. No marker is invented for an unknown owner.

## Verification and evidence

Eight affected integration suites total 102 passing cases. Seven owner-provenance unit tests, the new editor replacement-owner diagnostic witness and eight depth canaries pass. The focused native replacement case is rerun after the final conservative receiver guard and passes. No full regression or timing run was performed.

The workload now observes 54 values per iteration. Three new observations cover installation through the first captured source, mutation of the released source and invocation through the replacement. Five native and Node-after-tsc cases pass on the formatted source. Independent oracle, mutation guards, registration and fingerprints are current. Project formatting changes one workload/fingerprint, then zero files.

SQLite preserves `runtime/results/2026-10-09T22-09-35-868Z-verify.json` and content-addressed source/text/log evidence under `runtime/source-evidence/independent-captured-installers/`. Current compiler/runtime inputs match their recorded hashes. Raw CLI output and trimmed harness output are both preserved; the trimmed hash and instrumented prefix match the runtime record. Exact readback precedes loose deletion. Disposable workload outputs are removed.

## Remaining interfaces

Owners still identify named-function caller storage through `Owner.function` and `Owner.binding`. `resolve` does not fabricate named IDs for a closure-local owner. Next add a native witness with independent arrays declared inside an ordinary closure, then introduce an explicit function/closure owner scope across provenance, type lookup, marker names and replacement planning. Keep named marker spelling stable when adding closure markers.

Captured mutable source storage, ownership-taking captures, hidden source propagation through additional closure calls, captured record targets, concrete callback capture vectors, abstract calls, aliases, consuming wrappers, methods, defaults, field/index inputs, computed owners, retained control flow, recursive stores and remaining B7 rows still require their complete interfaces. The readonly replacement witness does not establish mutable capture acceptance. Preserve current guards until each interface is verified.

Five executable consumers remain B5 until family acceptance. Fifteen unrelated fixture edits remain untouched and uncommitted. Keep the whole compiler goal active. Reuse the warm Cargo target with incremental compilation disabled, Node 24.11.1 first on PATH and Homebrew Python for SQLite.

## Continuation prompt

Continue Tsrust from this checkpoint. Direct readonly captured-source installers now transport owner markers and release an overwritten source after both installer scopes end. Native nullable/plain fields, replacement-owner diagnostics and the 54-value matched workload pass. Implement explicit closure-local owner identities with a focused red native witness, then continue the full B7 plan, including mutable/owned capture interfaces and hidden composition. Preserve guards, unrelated fixtures, B5 consumers and exact SQLite evidence. Keep D2 separate.
