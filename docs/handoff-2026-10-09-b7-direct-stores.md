# B7 direct callback forwarding checkpoint

B7/gate 183 remains in progress. This checkpoint follows red witnesses at `e69c69f`
and holder capture implementation at `12ffe9d`. Read AGENTS.md,
docs/development-workflow.md, docs/plans/caller-borrowed-record-stores-plan.md and
docs/b7-direct-callback-stores.md first. No owner question is pending. Discriminated
unions remain a dedicated D2 feature.

## Implemented

Direct readonly and mutable callback parameter stores are lowered to ordinary
checked forwarding closures by the isolated direct_callback_stores module. The
complete reconstructed callback annotation preserves parameter access, result,
optional/default shape and rest packs. Existing closure capture analysis, caller
borrow summaries, payload alternatives and native field conversions then apply.
Stable aliases and named relays preserve the original owner. No implicit clone,
shared cell, dynamic dispatch, lifetime cast or new source callable kind was added.

Seven direct-store checks pass: readonly release, mutable state/release, alias relay,
required arguments, optional/default calls, rest calls and retained mutable conflict.
Twelve replacement checks and seventeen existing caller-store checks pass. The editor
witness checks declared parameter help, hover without internal names, and independent
mutable invocation rejection. All eight depth canaries pass after the analyzer hook.

Closed evidence is in benchmarks/records.sqlite under `tsrust-b7-direct-store-`
keys. Initial compile failure and its repaired check remain preserved. `focused.log`
has the first five direct tests and existing suites; `optional.log` and `rest.log`
contain the supplemental cases. `editor.log`, `depth.log`, `registration.log` and
`format-final.log` cover the other focused checks.

`runtime/results/2026-10-09T18-29-08-391Z-verify.json` verifies five cases each on
native and Node-after-tsc. The workload's fourteen observations, oracle and denominator
are unchanged; its second readonly installation now uses a direct assignment with a
source contract and mutation guard. There are no timing samples. Exact source inputs
are preserved under content-addressed `runtime/source-evidence/direct-borrowed-callback-stores/`
keys, including bytes before Rust formatting. Records were read back byte for byte.

## Next

Direct consuming inputs and exceptional completion still need explicit witnesses and
implementation as needed; do not infer their acceptance from reusable callbacks.
Then continue nullable stores, mutable data aliases, failure before/after installation,
exact replacement and clearing, branches, loops, early exits and disposal. Nested
generic callback environments and more complex recursive retained-store summaries
remain unfinished. Full B7 acceptance and timing are still pending.

The synthesized forwarding closure has a real checked origin. Keep it in the ordinary
analysis path so source loans and native carriers remain coordinated. Do not delete
the carrier boundary rejection for unrelated abstract callback stores. Keep checked
point types separate from final capture storage layouts.

Fifteen unrelated fixture edits remain uncommitted. Whole-project formatting preserves
their current bytes and semantics. Use explicit Git paths. Installed consumers remain
at B5 until family acceptance. Reuse the single Cargo target with incremental disabled,
Node 24.11.1 first on PATH, and Homebrew Python for SQLite. No full regressions or
timing ran in this slice.
