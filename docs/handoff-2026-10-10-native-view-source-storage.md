# Native view callbacks through normal source storage

Read AGENTS.md, docs/development-workflow.md and
native-view-callable-storage-plan.md. This checkpoint follows 9ee2e3a1.
Gate 189/S07 remain open. Public view factories remain guarded.

The original stored-callback positive now passes normal source analysis and complete
native emission. It constructs a Holder containing Once<Function<Window>>, returns
that record and invokes its stored field. The selected second input supplies the
returned view. O0/O3 execution preserves its original pointer after the unrelated
first owner ends; independent Rust compilation rejects selected-owner escape.

Callable compatibility distinguishes owning a captured value from proving that
value has no borrowed lifetime. The scoped environment contract accepts carried
native views without admitting physical shared/mutable reference captures. Checked
construction interns a scoped owning family, and record construction specializes
the field to that family. Ordinary storage admission retains its lifetime-free guard.

A shared ABI eligibility check runs after final error-effect annotation and before
emission. Unsupported data arguments, missing returned capture lifetimes and throwing
callbacks receive source diagnostics rather than reaching the emitter assertion.
The emitter uses the same eligibility check and retains its detailed lifetime-slot
validation. These remain intermediate implementation limits.

Evidence is stored in SQLite under
runtime/results/native-view-source-storage-2026-10-10.json, including the original
red witness and subsequent failed/successful checks. Early rejection-test attempts
had invalid Function argument syntax and unreachable code; those original logs are
retained honestly alongside the repaired controls.

## Continuation prompt

Continue normal source storage coverage from this checkpoint. Do not repeat the
private ABI proofs or report the original positive as red. Next implement finite
alternative families, moves, replacement, arrays, nullable fields, nested capture
and mutable views through normal source analysis and complete emission. Keep exact
owner provenance and consuming/reusable invocation contracts; never erase lifetimes
or substitute implicit copies. Extend the explicit ABI diagnostics into supported
forms as required. Complete public factory/observer IntelliSense and matched subarray
workload support before publishing view/viewMut. Gate 189/S07 and the broader goal
remain open. Use bounded TDD, dispatch canaries, project formatting and exact SQLite
evidence. No full regression or timing is due until S07 completion.

Installed consumers remain at f569190a; only the warm CLI is rebuilt for formatting.
No consumer refresh, full regression or timing is claimed. Target growth remains
approximately 1.3 GB in the workspace and 2.4 GB in the shared target. Native tests
clean their temporary compilations automatically.

Focused acceptance: 96 native-view tests, 114 callable tests, 107 depth canaries
and 16 native editor checks pass, with successful process exits and no aborted
targets. The warm CLI builds without warnings. Project formatting is checked
before the source checkpoint; full family regression and timing remain pending.
