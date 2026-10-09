# B7 nested hidden-source installers checkpoint

B7/gate 183 remains open. D2 discriminated unions remain a dedicated future feature. No owner question is pending. Read AGENTS.md, docs/development-workflow.md and docs/caller-borrowed-record-stores-plan.md first.

## Implementation

A locally constructed closure installer can call another locally constructed installer that stores a callback borrowing a captured input. The retained source maps through each construction context. Native witnesses cover nullable/plain readonly fields, independent-owner replacement, exact side effects and three installer layers. Source and editor checks reject conflicting access to the replacement owner and retain the previous owner across conditional replacement.

`record_parameter_borrows.rs` collects local closure origins during the existing call-edge walk. Explicit sources map through argument slots; captured sources map from the callee's inner binding to its outer construction binding only when the caller constructs that closure. Capture IDs from foreign contexts are not mapped. The new source red passed after this repair, but native replacement then exposed old-owner retention. `record_call_capture_release.rs` now checks capture eligibility against the callable's full checked inputs, including captures, rather than only written parameters. Existing signature and owner transport then execute the nested native witness without an emitter change.

This checkpoint does not establish calls through foreign captured installer receivers. Such calls need a separate environment/projection interface. Mutable retained captures still follow the receiver lifetime rule from the previous checkpoint; nesting does not allow an inner reusable mutable receiver to lend beyond its scope. Owning Once/Owned transfer retains its accepted contracts.

## Verification and storage

Eight affected integration suites total 110 passes. Eight owner-provenance units, 27 capture-loan units, one editor witness and eight depth canaries pass. All added executable sources exercise formatting and native assertions. The matched workload adds nested initial callback, released source, replacement callback and exact effects, bringing each iteration to 68 observations. Independent oracle, source contracts, mutation controls and registration pass. Five native and Node-after-tsc cases verify untimed.

Whole-project formatting changes one workload/fingerprint, then zero files. SQLite preserves `runtime/results/2026-10-09T22-44-14-386Z-verify.json` and 652 exact source/text/log entries under `runtime/source-evidence/nested-hidden-installers/`. Compiler and runtime input hashes match. Raw CLI emission, trimmed harness emission and instrumented prefix are verified. Exact readback and SQLite quick_check pass. Five disposable outputs totaling 1,126,667 bytes are removed. The shared Cargo target is 2.2 GiB; incremental directories contain no entries. No full regression or timing run occurs.

## Next work

Continue the B7 plan with captured installer receivers and their hidden source projections. Do not interpret the original closure's capture outer IDs in an arbitrary caller. Establish source borrow retention, complete replacement release, native signatures and concrete owner vectors together. Keep foreign/abstract uses conservative until the actual environment interface exists.

Captured record targets, callback capture vectors, abstract calls, aliases, consuming wrappers retaining borrowed sources, methods/defaults, field/index sources, computed owners, retained control flow and recursive stores remain pending. Family acceptance still requires complete regressions, accumulated matched measurements and consumer refresh. Five installed consumers remain B5.

Fifteen unrelated fixture edits remain untouched and uncommitted. Use explicit Git paths, the shared Cargo target, incremental off, Node 24.11.1 and Homebrew Python. Keep the full compiler goal active and D2 separate.

## Continuation prompt

Continue Tsrust from this checkpoint. Locally constructed nested installer calls now transport hidden readonly sources through three layers and release overwritten owners. Conditional replacement keeps the previous owner. Next implement the foreign captured installer receiver interface with explicit hidden source projections, using focused red witnesses, compiler/native/editor parity, matched workloads and exact SQLite evidence. Preserve source invocation contracts, conservative guards, unrelated fixtures and B5 consumers. Do not treat this checkpoint as B7 family acceptance.
