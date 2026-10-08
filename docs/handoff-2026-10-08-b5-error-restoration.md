# B5 error restoration checkpoint

B5 scoped aggregate composition remains incomplete. Gate 181 is the last accepted
numbered gate. This checkpoint extends the work after `56db876`; it is not family
acceptance. The active goal remains the full compiler/editor completion plan.

Read `AGENTS.md`, `docs/development-workflow.md`,
`docs/compiler-completion-plan.md` and `docs/scoped-callable-aggregate-plan.md`.
Only one agent works in this checkout. Preserve the fifteen existing unrelated
fixture modifications without committing them. Their formatting is allowed;
changing their semantics is not. Stage explicit paths.

## Completed work

Checked statement-entry and exact expression availability now select error payloads.
Expression snapshots follow argument/operand analysis and include callee/operator
spans used by native errors. Nested expressions sharing a start byte remain distinct.
Native witnesses cover failure before and after a comma-expression transfer,
nested arithmetic, bounds, and a failing array factory before a later index transfer.
Disposal instrumentation verifies optimization levels 0 and 3. A Map
transferred in a previous statement contributes empty internal storage; source
availability remains consumed. Catch must explicitly restore storage before use.
Empty Map assignment creates fresh native storage, ending the previous lifetime.
Outside completion transport, restoration skips dropping a moved native binding.

Native witnesses cover explicit throw, a later fallible call, a later bounds
failure, record-field storage and an independent sibling field. Disposal tests
at optimization levels 0 and 3 prove exactly one callback drop on both early and
late failure paths. The editor test verifies restoration and rejects reads before
restoration. All 70 composition tests, eight normal-stack depth canaries and 15
exception-contract tests pass. The focused editor test passes.

The matched workload now retains 24 observations. Source guards and rehashed
mutation controls pass. Native and Node-after-tsc verification passes untimed.
The independent oracle includes `value + 29` and `value + 30` for early and late
statement restoration, plus `value + 39` and `value + 40` for expression failures. SQLite preserves exact failures and repaired records; see the plan
for keys, including `runtime/results/2026-10-08T18-05-25-609Z-verify.json`.
Loose copies are removed. Project formatting changes no files on the second pass.
No full family regression or timing run was performed.

## Next implementation

1. Audit rewritten and synthetic expression spans as new compositions land.
   Exact spans now cover the verified call, bounds and arithmetic failure cases.
2. Add exact disposal witnesses for conditional transfers. A joined source binding
   can be unavailable while a native owner survives on one path. Do not replace
   that survivor with an empty Map without proving its proper disposal point.
3. Extend completion storage to loops, return, break, continue, nested try and
   finally without catch, with checked control-flow evidence.
4. Finish precise element and Map-entry capture release and the remaining callable
   compositions described in the plan.
5. Implement record Set keys using the owner's Rust-style value equality/hashing
   decision. Equal eligible fields mean one key. Prevent stored-key mutation.
   Callback fields have no automatic equality; explicit stable-key design and
   floating-point policy remain pending.

Maintain matched workload support during implementation. Run complete compiler,
LSP and timing checks only at the completed family milestone, preserving prior
records and resuming unrun groups. B5 is not accepted through conservative rejection.

## Environment

Use `CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target` and
`CARGO_INCREMENTAL=0`, offline locked Cargo. Node 24.11.1 must be first on PATH;
use `/opt/homebrew/bin/python3` for `benchmarks/storage/store.py`. Scratch debug
and release compilers include this checkpoint. Five user executable consumers
still represent gate 181; refresh atomically after accepted source, not as a
claim that B5 is complete. Reuse the single target and prune obsolete artifacts
before a longer handoff; do not create archives or snapshots.

## Continuation prompt

Continue Tsrust B5 from docs/handoff-2026-10-08-b5-error-restoration.md. Read
AGENTS.md, docs/development-workflow.md and docs/scoped-callable-aggregate-plan.md
first. Error restoration after prior-statement and within-expression transfers is verified;
conditional partial-transfer disposal remains next. Keep Rust-style record
Set equality/hashing, explicit cloning and existing ownership rules. Use focused
red witnesses, native disposal checks, editor coverage and matched benchmarks.
Preserve fifteen unrelated fixture edits, stage explicit paths, format the whole
project before checkpoints, and preserve exact evidence in SQLite. B5 and the
full compiler/editor completion goal remain active.
