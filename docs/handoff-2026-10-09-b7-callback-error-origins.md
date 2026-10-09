# B7 completed caller error origins

B7/gate 183 remains in progress. This checkpoint follows `da3b006`. Read AGENTS.md,
docs/development-workflow.md, docs/caller-borrowed-record-stores-plan.md and
this handoff. D2 discriminated unions remain a dedicated required feature. No owner
question is pending.

## Repaired

The kept typed-error witness now passes with its bound catch and success assertion.
Partial catch inference included calls inside the try body but omitted installation
calls already checked outside it, because the complete caller body was not stored.
The isolated caller_error_origins module retains completed named-call callback
argument types in Context. Regional inference uses those facts, and nested closures
inherit them. Facts are deduplicated; no source types, native carriers, ownership
loans or runtime storage are changed.

Native cases cover typed string failure and success, conditional number/string
failure origins, and a catch in a closure capturing the installed holder. Missing
concrete origins still report error-effect. No fabricated pure or dynamic contract
is introduced. The editor now refreshes catch symbols from checked declaration facts;
previously declarations were built before those facts loaded and hover stayed untyped.

## Verification

SQLite keys start with `tsrust-b7-callback-errors-`. `origin.log` repairs the original
witness; `extended.log` exposes the nested-capture omission; `captured-repair.log`
passes all nine consuming/completion/error witnesses. `unknown-control.log` passes
the missing-origin rejection. `controls.log` passes twelve replacement checks,
seven direct-store checks and four fallible-record-callback checks. `depth.log` passes
all eight canaries. `editor-final.log` passes the new typed catch hover and two existing
catch cleanup checks. Failed intermediate editor checks remain preserved.

The matched workload has eighteen observations: the preceding seventeen plus a
callback-thrown numeric failure caught inside a capturing closure. The independent
arithmetic oracle, source contract, mutation guards and manifest hashes are current.
`runtime/results/2026-10-09T18-42-14-533Z-verify.json` verifies five cases on native
and Node-after-tsc without timing samples. Original source bytes are preserved under
content-addressed `runtime/source-evidence/caller-callback-errors/` keys. Whole-project
formatting and registration are checked separately; measurement and family acceptance
remain false. No complete regressions or timing ran.

## Continue

No kept red remains in caller_callback_store_failures.rs. Continue nullable stores,
mutable data aliases, exact replacement/clearing, branches, loops, early exits and
disposal. More complex recursive retained-store summaries and nested generic callback
environments remain pending. Do not equate this focused catch repair with acceptance
of every exception/lifetime path in B7.

Fifteen unrelated fixture edits remain uncommitted. Preserve semantics and use explicit
Git paths. Installed consumers remain at B5 until family acceptance. Reuse the single
Cargo target, disable incremental compilation, put Node 24.11.1 first on PATH, and use
Homebrew Python for SQLite. Keep source point types separate from final native storage.
