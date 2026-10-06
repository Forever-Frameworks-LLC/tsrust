# Handoff: nested function declaration plan and red checkpoint

/caveman Continue tsrust in /Users/seancotter/Desktop/dev/learning/tsrust. Read AGENTS.md, docs/development-workflow.md, docs/handoff-2026-10-06-gate169.md and docs/nested-function-declarations-plan.md first. Inspect git status and log for the current checkpoint.

Gate 169 remains the latest accepted implementation. Gate 170 is planned and red, not accepted. The owner explicitly selected existing ownership rules for nested named captures: borrow local captures and transfer supported captures when an owned callable escapes. Do not introduce shared JavaScript capture cells or implicit cloning. This decision is settled; do not ask again.

Three std:testing fixtures in crates/tsrust-compiler/tests/fixtures/nested-function-declarations cover hoisting/self and mutual recursion/shadowing/callback use, later initialized and mutable captures/returned counters, and generic/default/rest/deep signatures. All three pinned Node reference runs pass. The focused nested_function_declarations compiler test fails all three at the existing nested-declaration syntax diagnostic. The red checkpoint had no compiler source changes. A subsequent preparatory chunk extracted top-level named header/body parsing into named_function_syntax.rs without accepting nested declarations. Detailed red evidence is retained under target/editor/nested-function-declarations-red.log as a gate-log record in benchmarks/records.sqlite; its loose copy was removed after exact read-back verification.

Start implementation with gate 170 lexical representation, scope registration and capture-free execution. Inventory exhaustive statement visitors before adding an AST declaration variant. Preserve lexical symbols and source identities; do not replace declarations with textual arrow bindings or a global name map. Use a dedicated local_functions module and small non-inlined parser helpers. Follow the complete physical plan for subsequent capture readiness, finite recursive environments, ownership escape and signature gates. Captured and signature witnesses remain pending red until their stages land; do not turn them into expected unsupported tests to claim full completion.

Gate acceptance requires editor navigation/formatting/diagnostics, all eight depth canaries after parser/analyzer dispatch edits, the whole editor crate, guide and complete matched benchmark support with untimed verification. Update milestone, coverage and roadmap, import/read back records, refresh five consumers atomically, then commit explicit paths. No routine full regressions or timing. The broader standard-library goal remains active.

Preserve the three unrelated owner fixture edits: comma-contexts/guards.tsr, map-for-each/values.tsr and map-group-by/documented.tsr. Never commit or reset them. One agent only. Use CARGO_INCREMENTAL=0 and CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target. Node is /Users/seancotter/.nvm/versions/node/v24.11.1/bin/node. Consumers now include the verified preparatory parser and stack repair. Library caches remain warm, but gate 169 cleanup removed obsolete test executables, so checks may relink. No archives or source snapshots.

The preparatory chunk also discovered the existing rest_parameters 100-deep call stack witness aborting during analysis. LLDB identified recursive ordinary-call frames and generic preparation, not parsing. The analyzer now keeps conditional-call construction in a dedicated helper, splits heavy generic specialization from its early dispatch checks, and routes direct nongeneric declared functions through a thin call dispatcher. The narrow rest witness now passes at the unchanged worker stack and source-depth limits. Completed failure and debugger evidence must remain in SQLite alongside the repair evidence. Gate 170 remains unaccepted until actual nested lexical declarations and the complete stage acceptance land.

Preparatory checks pass: 137 affected compiler checks, including all eight depth
canaries and generic/optional/rest contracts, and all 267 editor checks. The direct
declared-call dispatch frame is now 288 bytes versus its former 4,672 bytes. Detailed
failure, LLDB, compiled-frame and successful check logs are retained in SQLite under
target/editor/nested-function-parser-* as gate-log records, verified by exact
read-back. Five consumers are atomically refreshed and hash-verified. Generated
example and scratch test executables are pruned; libraries remain cached and tests
will relink. No timing or full compiler regression was performed. Nested declaration
implementation and its matched benchmarks remain pending; gate 170 is not accepted.
