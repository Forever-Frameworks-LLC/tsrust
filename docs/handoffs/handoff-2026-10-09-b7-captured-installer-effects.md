# B7 captured installer effects checkpoint

B7/gate 183 remains open. D2 discriminated unions remain a dedicated future feature. No owner question is pending. Read AGENTS.md, docs/development-workflow.md and docs/plans/caller-borrowed-record-stores-plan.md before continuing.

## Implemented behavior

A directly invoked ordinary installer closure can capture mutable effect state and copied scalar state while retaining explicit input parameters in a callback field. Native replacement releases the old named caller input and retains the replacement. The native witness observes arrays containing 3 and 10, a released first array changed to 4, and a captured effect counter of 6 after two invocations.

Source checking now retains a captured source input independently of the installer value. A callback stored into a caller-owned holder keeps that input borrowed after the installer leaves its block. Conflicting mutation reports `capture-conflict`. Clearing through a named helper releases the loan. Direct nullable local callback replacement remains an existing restriction; the new test initially exposed that restriction and uses the already supported clear helper.

`record_callable_identity.rs` separates explicit parameter bindings from all input bindings, including captures. `record_parameter_borrows.rs` builds binding summaries, then exports ordinary argument borrows and distinct captured-input borrows. `captured_record_borrows.rs` attaches captured loans to destination storage. Direct local borrowed captures identify their outer binding; consuming captures depend on installer storage; indirect borrowed receivers conservatively inherit their receiver loans. Checked program construction computes both summary forms together.

Native owner classification permits effect-only captures while retaining guards for captured record storage, captured retained sources, escaping, polymorphic, aliased and stored installer uses. Captured-source origins participate in incomplete-owner suppression. No closure-local owner is encoded as a named function ID. No implicit clone, lifetime cast, shared capture cell or invocation-contract change was added.

## Verification

The two original red witnesses now pass. Eight affected compiler integration suites total 100 passing tests. The new editor lifetime/conflict/clear witness and all eight depth canaries pass. The independent registration test passes before and after formatting. No complete regression or timing run was performed.

The matched workload now observes 49 values per iteration. Four new columns cover captured installer state, released input, replacement callback and exact effect totals. Five cases pass on native and Node-after-tsc. The first workload run caught an incorrect new oracle constant; its failure is preserved. The corrected oracle includes both post-install increments. Formatting changes one workload and fingerprint, then zero files. The formatted workload is independently verified again.

SQLite preserves corrected result `runtime/results/2026-10-09T21-55-18-197Z-verify.json` and formatted result `runtime/results/2026-10-09T21-55-59-500Z-verify.json`. Logs, compiler/runtime inputs and generated text are preserved under content-addressed `runtime/source-evidence/captured-installer-effects/` paths; early red and affected-run logs use `compiler-checks/captured-installer-effects/`. Exact readback precedes loose deletion. The pre-format workload is reconstructed in memory and verified against its recorded SHA256 before storage. Current raw CLI emission and trimmed harness emission both have exact records; the harness emission matches its recorded hash and instrumented prefix. Disposable workload outputs are removed.

## Remaining interfaces

Captured retained source inputs have source checking, not native acceptance. Complete their hidden owner-marker and lifetime interface, including closure-local owner identities. Do not remove conservative guards to make a test compile. Hidden captured sources propagated through additional closure-call edges still need their explicit mapping; captured record targets likewise remain unfinished. Preserve distinctions between actual binding IDs, argument slots and capture indices.

Continue concrete callback capture vectors, abstract calls, aliases, consuming wrappers, method bodies, defaults, field/index inputs, computed owners, retained control flow, recursive stores and the remaining B7 plan rows. Keep the complete compiler objective active. Family acceptance still requires complete regressions, accumulated matched measurements and consumer refresh. Five installed consumers remain B5; 15 unrelated owner fixture edits remain untouched and uncommitted.

## Continuation prompt

Continue Tsrust from this checkpoint. Captured effect-state installers now execute native replacement with explicit input owners; captured retained sources keep destination loans alive after installer disposal in source/editor checks. Finish captured retained-source native transport and the explicit closure-local owner namespace, then continue the full B7 plan. Use focused TDD, formatted native examples, matching editor checks and complete untimed benchmark support. Preserve guards until interfaces exist, the 15 unrelated fixtures, B5 consumers and exact SQLite evidence. Reuse the warm Cargo target with incremental compilation disabled; put Node 24.11.1 first on PATH and use Homebrew Python for SQLite. Keep D2 separate.
