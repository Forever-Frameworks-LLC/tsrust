# Captured Result view payloads

Continue from `607a7204`. Read AGENTS.md, docs/development-workflow.md,
docs/native-byte-views-implementation.md and the capture ABI handoff first.
Gate 189 and S07 remain open. Public borrowed signatures remain guarded.

Native union capture layout now renders the complete non-copy variant shape with
named environment lifetime arguments. It no longer falls back to an anonymous
data-union type after rendering a single alternative. The environment renderer
handles both owned unions and read-only union views.

Private formatted-source native witnesses cover consuming `Result<ByteView, int>`
captures and read-only callbacks matching either variant. O0/O3 execution verifies
the success pointer and unchanged integer error. Existing pointer, writable capture,
independent sibling lifetime and owner-escape controls remain in the combined native
test. No copying or hidden shared ownership was added.

A separate defect was exposed: capture_context projected Mutable parameter facts
but omitted ReadOnly facts. A captured Result match then tried to take ownership
of a ReadOnly parameter. Projection now remaps both facts. Unit controls check the
remapping, and the ordinary Result fixture executes captured matches over a record
success and string error. The original attempted borrowed harness first passed the
wrong Rust argument form; its failure is preserved separately from the compiler's
subsequent ReadOnly diagnostic and the successful repair.

190 focused checks passed: 49 private native view tests, five capture-context tests,
four callable-layout controls, nine registered signature controls, 18 Result tests,
89 depth canaries across all ten suites and 16 native editor tests. Warm CLI build
was warning-free. Whole-project formatting changed zero files and fingerprints.
Exact original logs and records are retained in SQLite under
`runtime/results/native-view-result-captures-2026-10-10.json` after byte-exact
readback. No full regression or timing. Installed consumers remain f569190a.

Pending: variant-distinct lifetime identity when both Result payloads contain views;
borrowing receiver returns and captured owner storage; escaped/rebound environments;
specialized callback/storage matrix; remaining collections/completion paths and
public factories. Do not claim the entire Result capture matrix is accepted.

## Continuation prompt

Continue gate 189 from this handoff. Start with Result payloads containing independent
native view lifetimes in both variants, then finish borrowing receiver return
contracts and captured owner storage. Verify escaped/rebound environments and
specialized callbacks/storage, followed by the remaining implementation plan matrix.
Use source/native red witnesses and independent pointer/escape controls. Keep actual
slices and explicit ownership. Preserve the public guard until the complete matrix
agrees. Finish editor/docs and matched benchmarks before gate acceptance; full
regressions and timing belong to S07 completion.
