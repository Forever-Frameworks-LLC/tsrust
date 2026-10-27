# B7 native ordinary closure installers checkpoint

B7/gate 183 remains open. The previous native closure-body installer red now passes. D2 discriminated unions remain a dedicated future feature. No owner decision is pending.

## Implementation

`native_function_signature_emit.rs` now renders a `NativeSignaturePlan` with explicit function and parameter bindings, retained lifetimes, record bounds, owner markers and replacement obligations. Named functions and forwarded methods retain their prior output. Before extending the workload, its 41-observation emitted CLI output remains exactly 345,109 bytes with SHA256 `052c93aabdf8c4bc731c1e392fb2f911f8cddbf1451e78e6bfb26103b450d697`.

`closure_record_signature_emit.rs` supplies ordinary closure signatures and invocation adapter bounds. Input slots map to checked binding IDs. Callback field conversions retain explicit input lifetimes. Adapter markers are neutral only with matching conversion bounds. Actual direct invocation passes named caller owner markers.

Native owner planning uses separate function/closure scopes and follows concrete closure call contexts. Owners still identify named caller storage. An unresolved closure-local owner does not become a fabricated named function ID. `closure_owner_uses.rs` preserves conservative guards for captured, escaping, polymorphic, stored and aliased installers. Supported readonly aliases inside a closure resolve to their input marker.

Checked closure borrow and release summaries are exported for emission. Shared record reification handles named and closure invocations, runs after successful invocation, and narrows the field carrier to the replacement owner without a clone or lifetime cast. Typed results retain their existing failure conversion.

## Verification and evidence

The eight affected integration suites total 98 passing cases. The first batch preserved three passing suites before an invalid mutable-alias test stopped the batch. The corrected failed suite and four unrun suites pass in the resumed record. The native installer case executes six readonly/mutable, plain/nullable and readonly-alias variants. A separate witness checks a typed throw after storage, the retained callback value and one installer effect. It does not prove old-owner release on a throwing path.

Forty existing borrow unit checks pass. The new planner unit independently verifies two distinct named caller owners and conservative alias-use suppression; it passes after fixing a test that reused a transferred callable. Exploratory mutable local aliases are rejected under existing data access rules, so no local data access syntax was added. The editor retained-borrow witness and eight depth canaries pass. Whole-project formatting changes one workload/fingerprint, then zero. No full regression or timing runs.

The matched workload contains 45 observations. Four new columns exercise a closure-body wrapper calling the saved conditional-family installer, the released old owner, the replacement value and exact effects. Five cases pass on native and Node-after-tsc. Independent oracles, source contracts, mutation guards, registrations and hashes are current. Measurement and family acceptance remain pending.

SQLite preserves `runtime/results/2026-10-09T21-34-16-612Z-verify.json`, 622 exact source/input/generated-text records under `runtime/source-evidence/closure-owner-transport/`, and 19 closed command logs. Every record is read back exactly before loose deletion. The runtime harness trims CLI stdout before hashing; both raw CLI emission and exact harness emission are preserved, and the latter is verified as the instrumented source prefix. Five disposable generated outputs totaling 937,758 bytes are removed. No process remains live at this checkpoint.

## Remaining work

Continue captured installers and complete closure-local owner identities rather than fabricating named storage. Concrete callback capture-type vectors through closure parameters still need their full transport. Preserve guards while completing aliases, abstract calls, consuming wrappers, callback inputs, method bodies, defaults and hidden captures. Continue field/index inputs, computed owners, retained control flow, recursive stores and the remaining B7 rows.

Source and native callable identities currently have separate view adapters because their function containers differ. Keep argument slots distinct from binding IDs in both adapters. The remaining closure owner namespace must be explicit. Family acceptance still requires complete regressions, matched timing review and consumer refresh. Five installed consumers remain B5. Fifteen unrelated fixtures remain untouched and uncommitted. Reuse the warm target, disable incremental compilation, keep Node 24.11.1 first on PATH and use Homebrew Python for SQLite.

## Continuation prompt

Continue Tsrust from this checkpoint. Read AGENTS.md, docs/development-workflow.md and docs/plans/caller-borrowed-record-stores-plan.md. Ordinary capture-free closure-body installers now execute with shared native record signatures, explicit parameter IDs, retained lifetimes, concrete named caller owner markers and replacement reification. The former kept native witness and typed-failure witness pass; source/editor checks agree. Complete captured installers, closure-local owner identities and concrete callback capture vectors, then the remaining B7 interfaces and plan rows. Preserve conservative guards, fifteen unrelated fixtures, B5 consumers and SQLite evidence. Keep D2 separate and the full compiler goal active.
