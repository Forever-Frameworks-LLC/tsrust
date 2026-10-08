# B5 error restoration checkpoint

B5 scoped aggregate composition remains incomplete. Gate 181 is the last accepted
numbered gate. This checkpoint extends the work after `58b4dc6`; it is not family
acceptance. The active goal remains the full compiler/editor completion plan.

Read `AGENTS.md`, `docs/development-workflow.md`,
`docs/compiler-completion-plan.md` and `docs/scoped-callable-aggregate-plan.md`.
Only one agent works in this checkout. Preserve the fifteen existing unrelated
fixture modifications without committing them. Their formatting is allowed;
changing their semantics is not. Stage explicit paths.

## Completed work

Checked statement-entry storage availability now selects error payloads. A Map
transferred in a previous statement contributes empty internal storage; source
availability remains consumed. Catch must explicitly restore storage before use.
Empty Map assignment creates fresh native storage, ending the previous lifetime.
Outside completion transport, restoration skips dropping a moved native binding.

Native witnesses cover explicit throw, a later fallible call, a later bounds
failure, record-field storage and an independent sibling field. Disposal tests
at optimization levels 0 and 3 prove exactly one callback drop on both early and
late failure paths. The editor test verifies restoration and rejects reads before
restoration. All 68 composition tests, eight normal-stack depth canaries and 15
exception-contract tests pass. The focused editor test passes.

The matched workload now retains 22 observations. Source guards and rehashed
mutation controls pass. Native and Node-after-tsc verification passes untimed.
The independent oracle adds `value + 29` and `value + 30` for early and late error
restoration. SQLite preserves exact failures and repaired records; see the plan
for keys, including `runtime/results/2026-10-08T17-57-52-860Z-verify.json`.
Loose copies are removed. Project formatting changes no files on the second pass.
No full family regression or timing run was performed.

## Next implementation

1. Add a red witness that transfers a Map within an expression before a later
   fallible operation in that same expression. Track availability at expression
   evaluation points; statement-entry snapshots cannot prove this case.
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
first. Error restoration after a prior-statement transfer is verified; moves
within expressions and conditional disposal remain next. Keep Rust-style record
Set equality/hashing, explicit cloning and existing ownership rules. Use focused
red witnesses, native disposal checks, editor coverage and matched benchmarks.
Preserve fifteen unrelated fixture edits, stage explicit paths, format the whole
project before checkpoints, and preserve exact evidence in SQLite. B5 and the
full compiler/editor completion goal remain active.
