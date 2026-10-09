# B7 closure source summaries checkpoint

B7/gate 183 remains open. This checkpoint fixes the retained-new-owner source diagnostic through ordinary closure-body installers. It does not accept native closure transport. D2 discriminated unions remain a separate future feature. No owner question is pending.

## Changes

`record_callable_identity.rs` separates named function identities from closure identities and gives each a parameter binding list. Closures retain outer bindings before their actual parameters. An outer binding may still have its parameter flag, so neither that flag nor the closure arity identifies a closure argument. Record views now consult explicit closure and method parameter lists while retaining ordinary named parameter handling.

Retained-borrow summaries and replacement replay share callable identities. Their internal places use actual binding IDs; call edges map those IDs to argument slots. Exported summaries use source slots. Ordinary closure invocation applies replacement release and retained-new-owner attachment in that order. Named relays can follow ordinary closure edges. Conditional replacement retains the old owner. The source ownership model and native owner namespaces remain unchanged.

## Evidence

The new source control previously accepted mutation of the newly installed owner. It now reports `capture-conflict`. Controls cover direct calls, named relays, conditional replacement and formatted-source idempotence. The new editor witness checks clean diagnostics, callback hover and the retained-owner diagnostic. Ninety-six affected integration cases pass with the existing native closure installer excluded. Forty borrow units, sixteen record-view units and eight depth canaries pass. Whole-project formatting twice changes zero files and fingerprints. No full regression or timing runs.

The original native installer remains a kept red. Its native method needs a record-view parameter, a retained-input lifetime and replacement conversion bounds; it also needs concrete closure owner transport and caller reification. Do not present source acceptance as native support. No new workload observations were added before native support exists. Existing matched workload support and measurement status remain unchanged. Closed command logs with prefix `tsrust-b7-closure-source-` are preserved and read back exactly from SQLite.

## Next work

Extend the shared native function signature builder to checked closure parameters and their distinct binding layout. Build closure replacement obligations and retained-input lifetimes from explicit callable summaries. Complete concrete owner planning across closure parameters and captures without encoding closure-local storage as a named-function owner ID. Preserve conservative guards for abstract uses, aliases, consuming wrappers and unsupported captures. Make the native installer witness pass, add native retained-new-owner and released-old-owner controls, then extend matched workload support and its oracle/contracts.

Capture-sourced stores, closure methods, abstract invocations and consuming wrappers are still outside this checkpoint. Follow the remaining B7 plan rows after concrete closure transport. Family acceptance still requires full regressions, matched timing review and consumer refresh. Five installed consumers remain B5. Fifteen unrelated fixtures remain uncommitted and untouched. Reuse the warm target with incremental compilation disabled. Use Node 24.11.1 and Homebrew Python for SQLite.

## Continuation prompt

Continue B7 from this checkpoint. Read AGENTS.md, docs/development-workflow.md and docs/caller-borrowed-record-stores-plan.md. Source retained-borrow and replacement summaries now follow ordinary closure calls using distinct callable identities and explicit parameter binding lists. The source and editor retained-owner control passes; native closure installer transport remains red. Extend shared native signatures, replacement bounds, retained lifetimes, concrete closure owner transport and caller reification. Keep fifteen unrelated fixtures, B5 consumers, conservative abstract-use guards and SQLite evidence. Keep D2 separate and the full compiler goal active.
