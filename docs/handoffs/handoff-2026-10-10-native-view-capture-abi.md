# Native view capture ABI checkpoint

Continue from `eea2ad7f` and the capture-provenance handoff. Read AGENTS.md,
docs/development-workflow.md and docs/milestones/native-byte-views-implementation.md first.
Gate 189 and S07 remain open; public borrowed signatures remain guarded.

Capture environments now count native view leaves as inner lifetime slots,
independently of outer capture references. The shared native signature plan accepts
capture lifetime names. Returned carried origins refer to those environment names
instead of unconstrained invocation lifetime parameters. Record capture fields keep
independent slots. The ordinary-program capture-storage shortcut remains in place.

The first native red was a missing named lifetime on a captured `Record0<'_>`.
The extended formatted-source native witness now verifies consuming closures that
return a captured read-only view, one field of a captured record and a captured
writable view. O0/O3 checks prove exact pointer identity, a selected field surviving
an unrelated shorter sibling lifetime and writes reaching the original owner.
A separate negative Rust compilation proves the captured result cannot outlive its
original owner. No byte copies, erased pointers or shared owners were introduced.

Lifetime-count controls cover native array leaves and preserve the prior count for
ordinary recursive array containers. Native arrays' full capture execution matrix
is not accepted by that counting control alone.

Pending: borrowing receiver contracts and captured owner storage; Result capture
layout and variant lifetime identity; escaped or rebound environments; specialized
callbacks and stored families; remaining collection/control-flow matrix and source
factories. In particular, do not mistake consuming closure support for borrowing
callback acceptance. The implementation plan retains the complete final scope.

Focused verification passed 49 private view/native ABI tests, three environment
lifetime controls, nine registered callable signatures, 89 depth canaries across all
ten suites and 16 native editor project checks: 166 total. Whole-project formatting
changed zero files and fingerprints. Exact original logs, including native red and
repairs, are retained under
`runtime/results/native-view-capture-abi-2026-10-10.json` in records.sqlite.
No full regression or timing run. Installed consumers remain f569190a; the warm CLI
was rebuilt for whole-project formatting.

## Continuation prompt

Continue gate 189 from this handoff. Verify and finish borrowing receiver lifetime
contracts, captured owner storage and Result capture layouts, then escaped/rebound
environments and specialized callback/storage combinations. Preserve actual slices
and explicit ownership. Use source and generated native red witnesses, pointer and
escape controls, focused compiler/editor checks and SQLite evidence. Keep factories
guarded until the required matrix agrees. Finish docs and matched benchmark support;
full regressions and timing belong to S07 completion.
