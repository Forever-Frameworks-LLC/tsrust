# B7 closure-local owners checkpoint

B7/gate 183 remains open. D2 discriminated unions remain a dedicated future feature. No owner question is pending. Read AGENTS.md, docs/development-workflow.md and docs/plans/caller-borrowed-record-stores-plan.md first.

## Implementation

Owner identity now carries `Scope::Function` or `Scope::Closure` plus a binding ID. Named-function marker spelling remains `TsrOwner<function>_<binding>`; closure-local owners use `TsrClosureOwner<closure>_<binding>`. Provenance, marker rendering, payload alternatives and owner type queries use the explicit scope. Unresolved capture references do not become fabricated local owners. These are checked lexical storage identities, not a claim of arbitrary runtime instance identity.

An effectful installer can replace callbacks borrowing arrays declared inside an ordinary closure. The old array becomes available while the holder retains the replacement. Native variants cover readonly nullable/plain fields, mutable nullable fields, exact installer effects and nested local installers capturing those arrays.

The initial simple witness already passed through direct-store lowering. Adding an effect counter exposed the missing native owner namespace as E0502. The scoped identity fixes that real witness. Do not treat a passing optimized direct store as evidence for the ordinary call interface.

Nested installer composition exposed another real defect: storing a captured data input retained unrelated mutable effect-counter loans. Ordinary local bindings can have no initializer metadata, so initializer-based local-origin detection is insufficient. An attempted parser-ID/type refinement did not fix this. The final solution uses existing capture ownership paths: payload attachment records direct loans under each `$capture<inner>` field; a retained captured source selects that field, with a conservative whole-receiver fallback when no precise path exists. No initializer replay, shared capture cell, clone or lifetime cast is introduced. Temporary debug logging is removed.

## Verification

Eight affected integration suites total 103 passing tests. Eight owner-provenance tests, 27 capture-loan tests, the new editor witness and eight depth canaries pass. The native local-owner witness executes four variants, including nested captured installers. The editor accepts direct and captured local-owner composition and rejects mutation of the retained replacement. No complete regression or timing run was performed.

The matched workload now observes 58 values per iteration. Four new values come from nested effectful installers over closure-local arrays: initial callback, released old source, replacement callback and exact effects. Five native and Node-after-tsc cases pass on the formatted source. Independent oracle, registration, mutation guards and fingerprints are current. Project formatting changes one workload/fingerprint, then zero files.

SQLite preserves result `runtime/results/2026-10-09T22-19-34-378Z-verify.json` and 636 exact source/text/log entries under `runtime/source-evidence/closure-local-owners/`. Compiler/runtime inputs match their recorded hashes. Raw CLI emission and trimmed harness emission are preserved; the trimmed hash and instrumented prefix match the result. Logs include the initially passing simple case, genuine effectful red, failed composition and failed initializer hypothesis, diagnostic investigation and final passing checks. Loose records are deleted only after exact readback. Five disposable outputs totaling 1,059,583 bytes are removed.

## Next work

Continue retained mutable captures and ownership-taking captures with focused red witnesses. A mutable callback input passed directly to an installer now works from closure-local storage; this does not establish mutable data captured by the installer itself. Preserve source invocation contracts and the owner-selected ownership model when resolving receiver lifetime constraints.

Hidden sources through additional closure-call edges, captured record targets, concrete callback capture vectors, abstract calls, aliases, consuming wrappers, methods, defaults, field/index inputs, computed owners, retained control flow, recursive stores and remaining B7 rows still need complete interfaces. Current conservative guards remain until their corresponding call and lifetime interfaces exist. Complete family regressions, accumulated matched measurements and consumer refresh only at B7 family acceptance.

Five executable consumers remain B5. Fifteen unrelated fixture edits remain untouched and uncommitted. Reuse the shared Cargo target with incremental compilation disabled, Node 24.11.1 first on PATH and Homebrew Python for SQLite. Keep the full compiler goal active and D2 separate.

## Continuation prompt

Continue Tsrust from this checkpoint. Function and closure-local owner scopes are now explicit throughout provenance and native payload planning. Effectful replacement over closure-local arrays passes readonly/plain, readonly/nullable, mutable/nullable and nested captured-installer variants. Per-capture loan paths prevent unrelated effect counters from being retained. Implement the next full B7 interfaces with focused TDD, starting with mutable retained captures and ownership-taking captures, then hidden composition and the remaining plan rows. Preserve invocation contracts, conservative guards, unrelated fixtures, B5 consumers and SQLite evidence. Keep D2 separate.
