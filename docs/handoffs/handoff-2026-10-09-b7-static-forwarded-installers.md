# B7 static forwarded installer checkpoint

B7/gate 183 remains open. Static invocation through a saved named installer now preserves independent caller owners and executes natively. The closure-body installer witness remains red. D2 discriminated unions remain a dedicated future feature. No owner question is pending.

## Implementation

`native_function_signature_emit.rs` extracts the named function's native parameters, generic record bounds, retained-input lifetimes, callback bounds, owner parameters and replacement obligations. Named functions and applicable capture-free forwarded methods now share that signature. The extraction preserves the previously accepted workload's emitted bytes exactly: SHA256 `73f7d598f3cf9ec754606f658054554b30a451cd38cb400fac7e3bbad9facdd3` before new workload scenarios are added.

`forwarded_record_signature_emit.rs` supplies forwarded method signatures and trait replacement obligations. Trait argument lifetimes map explicitly to the target's retained-input groups. When owner markers are enabled, generic trait conversions use a neutral marker with a required matching `From` bound; they cannot silently construct a concrete owner-tagged carrier. Typed-result adapters carry those bounds too. The native method and counted method transport actual owner parameters.

`forwarded_owner_uses.rs` isolates conservative classification. Bare named function materialization has no local binding and is not an abstract use; mistaking it for one initially suppressed every static forwarding context. Local values used as callback arguments, stored/returned values, aliases and unsupported captured/consuming wrappers retain conservative suppression. A planner test verifies that even grounded symbolic inputs cannot bypass an abstract-use guard. Closure-body call contexts remain suppressed until they have a complete interface.

Concrete owner and callback-capture planning follow checked `ForwardFunction` edges. Forwarded calls evaluate the receiver and arguments once, pass caller markers, preserve typed results and reify completely replaced record carriers after successful invocation. No source kind, clone rule, shared cell or lifetime cast was added.

## Verification

The eight affected integration suites pass 95 cases, excluding only `closure_body_installer_releases_independent_input_owners`. The former installer-function-value red now passes. New native checks cover a single input, independent readonly and mutable owners, nullable and plain fields, exact effects and typed failure after storage. The failure witness verifies the installed value and typed error; it does not prove old-owner release on a throwing path. Forty borrowed/provenance unit checks, the forwarded-call editor witness and eight depth canaries pass. The stronger symbolic guard test passes separately on final source. Its initial short exact filter selected zero tests; the full module-qualified filter is authoritative.

The matched workload has 41 observations. Four added results observe a saved conditional installer's first invocation, released old-owner state, replacement invocation and exact effects. This also composes the accepted saved-local family with forwarding. Independent oracles, source contracts, mutation guards, registrations and fingerprints are current. Whole-project formatting changes one workload/fingerprint, then zero; final passes change zero. No whole-crate regression or timing ran.

SQLite preserves both untimed verification records, `runtime/results/2026-10-09T21-02-11-214Z-verify.json` and `runtime/results/2026-10-09T21-03-58-996Z-verify.json`. Each verifies five cases on native and Node-after-tsc with empty timing arrays. The first compiler inputs were preserved before strengthening a test-only assertion; the second has the final input hashes. Exact sources, unmodified and instrumented emitted code and generated text use `runtime/source-evidence/forwarded-owner-transport/`. Closed logs use `tsrust-b7-forward-*` and `tsrust-b7-native-signature-*`. Every import is read back byte for byte; original failures remain preserved. Loose records and generated workload artifacts are removed after preservation.

## Remaining work

The closure-body red still fails with a missing `C0: From<Closure0<...>>` bound and concrete-record versus record-view argument mismatches. Extend the shared native signature machinery to checked closure parameters using `CheckedClosure.parameters`; their indices include a different capture/parameter layout. Add source retained-borrow and replacement summaries for those calls before claiming caller-owner release. Complete concrete provenance across closure captures and arguments without inventing ordinary-function owner IDs for closure-local storage.

Abstract installer calls, aliases, consuming wrappers, callback-input forwarding and closure methods remain conservative or unverified. Continue their complete native/source interfaces, then the field/index inputs, computed owners, retained control flow, recursive stores and other B7 plan rows. Consider caching per-function symbolic owner plans if profiling confirms repeated global use classification is costly; do not weaken the guards as an optimization.

Family acceptance still requires full regressions, matched timing review and consumer refresh. Five installed consumers remain at B5. Fifteen unrelated fixtures remain untouched and uncommitted. Reuse the single Cargo target, disable incremental compilation, keep Node 24.11.1 first on PATH and use Homebrew Python for SQLite. No process remains live at this checkpoint.

## Continuation prompt

Continue Tsrust from this checkpoint. Read AGENTS.md, docs/development-workflow.md and docs/plans/caller-borrowed-record-stores-plan.md. B7 static saved installers now execute with shared native record signatures and independent named-owner transport. One closure-body installer witness remains red. Extend the shared signature and source summaries to closure parameters/captures, then complete concrete closure provenance with retained-new-owner and released-old-owner controls. Preserve conservative abstract-use guards, fifteen unrelated fixtures, B5 consumers and SQLite evidence. Keep D2 separate and the full compiler goal active.
