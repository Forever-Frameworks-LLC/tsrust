# B7 direct input-owner replacement checkpoint

B7/gate 183 remains open. D2 discriminated unions are a separate feature, with a dedicated plan and future schema-validation integration. No source ownership decision is pending.

## Implemented

Independent factory instances with the same callback origin now release the overwritten input in recognized direct record installers. Nullable and non-nullable fields work. Direct array and record capture replacement also permit old-input reuse while the holder remains live.

The isolated `direct_record_store_emit.rs` module exposes a checked, single assignment at the caller. It constructs the original closure using the existing capture modes and final native carrier. This lets Rust see the overwrite even when the previous and replacement owners have the same native type. There is no cloning, shared cell, lifetime cast or source ownership change.

The proof deliberately excludes optional/default/rest parameters, fallible functions, hidden captures, effectful actual arguments, consumed non-copy arguments, unions in captures, local capture aliases, polymorphic closures and bodies with additional statements. Those calls retain ordinary lowering. Consumed non-copy arguments remain excluded even if unused, preserving destruction scope. Existing argument and body effect controls execute exactly once.

## Evidence

Five focused integration tests cover six native programs, including both nullable and non-nullable factory variants. The complete affected batch passes 77 integration cases across eight suites. The independent-factory editor witness passes, checking diagnostics, nullable hover and rejection before replacement. All eight depth canaries pass. No full regression or timing ran.

The matched workload now has 31 observations, with independent arithmetic oracle, mutation guards, source contract and registration. The formatted-source registration passes. `runtime/results/2026-10-09T19-37-10-905Z-verify.json` verifies five cases on native and Node-after-tsc. All timing arrays are empty. The result and `tsrust-b7-input-instances-*` logs are imported into SQLite and read back byte for byte. Compiler and workload source bytes are preserved under `runtime/source-evidence/input-owner-instances/`. Whole-project formatting changes one workload, then zero files; token-preserving hashes are current.

## Remaining work

The direct shape is not a complete solution for independent owner instances. Next add a failing same-origin factory witness whose installer increments a counter before storing its callback. Keep old-input reuse while the holder remains live and assert both installer effects. Then cover aliases, relays and broader control flow. Native callback type equality cannot prove owner identity. Do not weaken the source ownership assertions or skip installer effects.

The broader B7 plan also retains branch joins, loops, early exits, disposal, recursive retained-store summaries and nested generic environments. Finish that family before full regressions, matched timing and consumer refresh. The five installed consumers remain at B5. Fifteen unrelated fixture edits remain unstaged. Use explicit Git paths and the existing warm Cargo target with incremental compilation disabled.

## Continuation

Read AGENTS.md, docs/development-workflow.md and docs/plans/caller-borrowed-record-stores-plan.md. Continue B7 from the effectful same-origin installer red, then generalize native ownership transport without changing source rules. Keep D2 separate. Preserve existing passing evidence and use bounded affected checks.
