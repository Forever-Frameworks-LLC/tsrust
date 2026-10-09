# Precise indexed capture release

B5 remains incomplete. The first replacement witness is
`replacing_known_array_element_releases_only_its_capture` in
`scoped_record_composition.rs`. Replacing index 1 with a callback borrowing the
first array must release the previous callback borrowing the second array.
Expected native output is `3 3 9`. The independent Node assertion passes;
current source checking rejects mutation of the second array with
`capture-conflict`. No compiler implementation has changed for this slice yet.

The companion rejection cases protect surviving captures, a borrowed element
handle, and an unknown replacement index. A borrowed handle currently rejects
replacement with `view-conflict`; that existing storage protection must remain.

## Source ownership

Introduce an isolated indexed-owner module. Dense literal slots need distinct
internal paths, including nested arrays and record fields. Preserve these paths
through moves and reaching-state joins. Resolve only checked constant indices;
unknown indices retain every possible capture. Stage replacement values before
discarding the previous slot, including self-assignment and fallible evaluation.
Remove the redundant conservative append in indexed assignment only once the
precise replacement path is complete.

Index identity changes under shift, unshift, splice, reverse and sort. Model the
operation exactly or collapse its slots into conservative container ownership
before changing their positions. Never keep stale slot identities. Pop and shift
also transfer the removed payload into a result: discarded results, bound
results, nullable results and temporary arguments need separate lifetime checks.
Do not release a removed capture before its last result owner is gone.

## Native storage

The whole-function array carrier currently mentions all inserted environments.
Source loan removal alone therefore cannot establish acceptance. Add independent
point-sensitive array epochs after carrier planning. Track actual payload types
under carrier wrappers, join the surviving concrete environments and move
surviving elements into their checked destination representation. Retarget later
indexing, field invocation and moves to that epoch without changing public
nominal contracts. Prove every excluded carrier alternative unreachable; do not
cast or clone to hide a lifetime.

Start with straight-line constant-index replacement, then field storage and
moves. Follow with removal and reordering, branches, loops, exceptions and
completion transport. Keep uncertain paths conservative. Measure any survivor
conversion scans at the B5 family milestone rather than claiming ordinary
constant-time mutation is unchanged.

## Acceptance

Require focused source and native witnesses, disposal at optimization levels 0
and 3, editor diagnostics and guide examples. Extend the existing matched
composition workload, independent oracle, source guards and fingerprints during
implementation. Run untimed native and Node-after-tsc verification. Run the eight
depth canaries after analyzer dispatch changes. Format the complete project
before committing; preserve unrelated fixture semantics and use explicit paths.
Full family regressions, timing review and installed consumer refresh remain
pending until the complete B5 scope is ready.
