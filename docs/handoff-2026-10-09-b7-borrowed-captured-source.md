# B7 borrowed captured-source native checkpoint

B7/gate 183 remains open. D2 discriminated unions remain a dedicated future feature. No owner question is pending. Read AGENTS.md, docs/development-workflow.md and docs/caller-borrowed-record-stores-plan.md before continuing.

## Native progress

An ordinary installer closure can now retain a borrowed array captured from its enclosing scope in a callback field. The callback remains usable after the installer leaves its block. A named clear helper releases the source loan. Native witnesses cover nullable fields, plain fields, and two independently captured arrays used by the same callback. A plain local alias that transfers array ownership into the shorter block remains an escape error; it is not a borrowing alias.

`CapturedBorrow` now retains the installed callback capture index. `closure_record_signature_emit.rs` combines ordinary parameter summaries with captured-source summaries while keeping their binding namespaces distinct. Explicit inputs use their record-input lifetime; hidden borrowed inputs use their existing closure capture lifetime. Record view parameters and callback conversion bounds are emitted together. Generic invocation adapters preserve hidden capture lifetime obligations rather than inventing explicit argument slots. Empty summaries return before capture layout work.

Captured-source origins remain excluded from concrete owner-marker planning. This checkpoint establishes borrowed capture lifetime transport, not independent captured-owner replacement or an explicit closure-local owner namespace. Ownership-taking captures still need a receiver storage lifetime. Existing guards remain in place.

## Verification

The original native witness failed on missing record views and `From<Closure>` bounds; it now passes. Eight affected compiler integration suites total 101 passing tests, and the existing captured-source editor lifetime/conflict/clear witness passes. After those suites, the same native witness was expanded with the two-array case and rerun successfully. Eight depth canaries already passed at the immediately preceding source-analysis checkpoint; this checkpoint does not change shared parser/analyzer dispatch. No complete regression or timing run was performed.

The matched workload now observes 51 values. Its new scenario retains a captured array through installer disposal, observes the callback, clears through a named helper and mutates the released source. Five native and Node-after-tsc cases pass after formatting. Source hashes, independent oracle, registration and mutation guards are current. Adding a second clear helper exposed a previously insufficient existence-only guard; the contract now requires both clear operations. Formatting changes one workload/fingerprint, then zero. The final formatted registration check passes.

SQLite preserves final result `runtime/results/2026-10-09T22-02-06-995Z-verify.json`. Its compiler/runtime input hashes are checked against exact current bytes before storage; generated CLI output, trimmed harness output and instrumented prefix are verified. There are 624 exact source/text/log entries under `runtime/source-evidence/hidden-captured-source/` for the final run. Earlier failures and exploratory logs are preserved under `compiler-checks/hidden-captured-source/`. Preliminary successful result `runtime/results/2026-10-09T22-00-31-854Z-verify.json` is retained but superseded: source and guard edits overlapped that preliminary build, so use the final immutable-input run for acceptance. Loose records are removed after exact readback. Five disposable outputs totaling 984,126 bytes are removed.

## Next interfaces

Complete explicit owner identities for closure-local storage and hidden captures. Add a red witness replacing one captured installer's retained input with another installer's independent input, then access the old input while retaining the replacement. Keep capture indices, actual binding IDs and written argument slots distinct. Do not remove suppression before native owner markers and signature/lifetime transport exist together.

Continue hidden source propagation through additional closure-call edges, captured record targets, concrete callback capture vectors, abstract calls, aliases, consuming wrappers, methods, defaults, field/index inputs, computed owners, retained control flow, recursive stores and the remaining B7 plan rows. Owned capture storage must not be treated as an independent borrowed input. No implicit clone, shared cells or lifetime casts.

Five executable consumers remain B5 until family acceptance. Fifteen unrelated fixture edits remain untouched and uncommitted. Use explicit Git paths, the shared warm Cargo target, incremental compilation disabled, Node 24.11.1 first on PATH and Homebrew Python for SQLite.

## Continuation prompt

Continue Tsrust from this checkpoint. Borrowed captured arrays now survive installer disposal in native execution with nullable/plain fields and a two-array capture case. Source/editor lifetime checks and named clearing agree. Finish captured-owner replacement and the explicit closure-local owner namespace, then continue the complete B7 plan. Preserve conservative guards until corresponding interfaces exist, all unrelated fixtures, B5 consumers and exact SQLite evidence. Keep D2 separate. The full compiler objective remains active.
