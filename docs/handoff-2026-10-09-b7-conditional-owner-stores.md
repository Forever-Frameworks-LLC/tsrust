# B7 conditional owner-store checkpoint

B7/gate 183 remains open. Direct conditional stores now work for nullable and plain callback fields, preserving selected effects and caller-owned capture lifetimes. D2 discriminated unions remain separate. No owner question is pending.

## What changed

A nullable conditional store previously joined two different closure payloads inside the same union, generating incompatible Rust branch types. A plain conditional store named a two-origin family with only two native generic arguments even though owner expansion required four. Sequence-valued branches exposed the nullable join defect after their observable prefix effects.

`record_store_branches_emit.rs` owns the repair outside recursive dispatch. It converts each selected conditional branch to the destination carrier before joining it. It evaluates the predicate once and preserves sequence prefixes only in the selected branch. Tagged callable-store wrappers are converted as individual closure origins. Generic replacement bounds expand tagged family origins rather than requiring a malformed aggregate source-family type.

`record_parameter_borrows.rs` now traverses conditional branches and the final value of sequences when collecting retained capture provenance. The existing source escape check rejects a conditional callback borrowing a local installer owner. Source ownership, native references and all-normal-exits release remain unchanged. No cloning, shared cell, dynamic dispatch or lifetime cast was added.

## Verification and evidence

All 86 integration cases across eight affected suites pass. The four new cases cover nullable and plain stores, exact predicate/selected-branch effects and source escape rejection. Both branch choices execute; two installer calls produce an effect count of 322. All 38 borrowed/provenance unit checks, the conditional-store editor witness and eight depth canaries pass. Editor checks cover nullable readonly hover, released old-owner access and retained new-owner conflict. No whole-crate regression or timing ran.

The matched workload now has 37 observations. Four new independent results observe conditional invocation, old-owner state after release, replacement invocation and exact effects. The oracle, mutation guards, source contracts, registrations and hashes are current. Whole-project formatting changes one workload and its fingerprint, then zero files. Formatted registration passes.

`runtime/results/2026-10-09T20-23-40-138Z-verify.json` verifies five cases on native and Node-after-tsc, with empty timing arrays. SQLite preserves the exact result, compiler/workload inputs and generated text under `runtime/source-evidence/conditional-owner-stores/`; all imports are read back byte for byte. Logs use `tsrust-b7-conditional-owner-*`. The initial short-filter command selected zero tests; the authoritative initial red is red-exact.log. Original native failures and repaired checks remain preserved. Loose evidence and disposable workload artifacts are removed after preservation.

## Remaining scope

This checkpoint accepts direct conditional store expressions, not arbitrary stored-family composition. Next probe a conditional callback first saved to a local and then assigned to a holder. Its precise local family and final owner-expanded record carrier must remain separate. Do not assume the direct branch conversion proves that form. Then continue closure-body caller provenance, installer function values, field/index inputs and computed owner temporaries. Retained loops, early exits, disposal, recursive stores and nested generic environments remain in the B7 plan.

Preserve old-owner reuse while the holder is live and the new-owner rejection checks. Keep source guards until complete provenance exists. B7 family acceptance still requires full regressions, matched timing review and consumer refresh; five installed consumers remain at B5. Preserve fifteen unrelated fixtures and stage explicit paths. Use the existing Cargo target with incremental compilation disabled, Node 24.11.1 first on PATH and Homebrew Python for SQLite.

## Continuation prompt

Continue Tsrust from this checkpoint. Read AGENTS.md, docs/development-workflow.md and docs/caller-borrowed-record-stores-plan.md. B7 direct conditional stores pass with selected effects and independent owner lifetimes. Add a focused red for a conditional family saved in a local before storage; preserve precise local representation separately from the final native carrier. Continue closure-caller provenance and the rest of B7 before its family milestone. Keep D2 separate and the full compiler goal active.
