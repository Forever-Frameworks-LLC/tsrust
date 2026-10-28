# Shared owner capture provenance and signature contract

Continue from `5410d7ea`. Read AGENTS.md, docs/development-workflow.md,
docs/milestones/native-byte-views-implementation.md and the variant lifetime handoff first.
Gate 189 and S07 remain open; public borrowed signatures remain guarded.

Returned call summaries now distinguish BorrowedCapture from captured carried
views. A read-only storage borrow through a shared capture retains the external
owner root and field footprint, plus relevant inherited origins. The loan blocks
mutation and transfer while its returned holder remains live. Native signature
planning maps that storage source to the capture's existing reference lifetime.
It does not unnecessarily tie the result to the local closure value.

The new analyzer red used private returned-borrow metadata on a checked closure.
After repair, its returned storage loan keeps the outer owner, permits reads and
rejects mutation/transfer. A separate native control injects the same private
summary, extracts the actual compiler-planned signature and supplies a safe
String-to-byte-slice body. O0/O3 execution checks exact pointer identity after the
local closure is dropped; separate negative compilation rejects owner escape.
This verifies provenance and signature planning. It does not compile a real source
factory or the complete generated closure body. Keep that boundary explicit.

Owned or mutable captured storage still receives a receiver-contract diagnostic.
Exclusive returns through shared captures remain rejected. Those cases need an
actual receiver lifetime and caller loan tied to the closure environment rather
than pretending the moved outer owner is still the physical owner. Escaped/rebound
environments, callback/storage families and the remaining plan matrix stay open.

183 focused checks passed: 51 private view tests, nine registered signatures,
18 Result tests, 89 depth canaries across all ten suites and 16 native editor tests.
No failed or aborted suites. Warm CLI build was warning-free. Whole-project
formatting changed zero files and fingerprints. Original red, repair and check logs
are retained in SQLite under
`runtime/results/native-view-shared-owner-captures-2026-10-10.json`, after exact
readback. No full regression or timing. Installed consumers remain f569190a.

## Continuation prompt

Continue gate 189 from this handoff. Implement true receiver-scoped returns for
owned and mutable capture storage, with caller loans on the closure environment and
named Rust receiver lifetimes. Reject consumption that would destroy returned owner
storage. Add real source/native factory integration witnesses when factories become
available; do not treat private metadata/signature controls as complete source
acceptance. Then finish escaped/rebound environments, callback/storage families and
the remaining implementation-plan matrix. Keep the public guard until the complete
matrix agrees, and finish editor/docs and matched benchmark support. Full
regressions and timing belong to S07 completion.
