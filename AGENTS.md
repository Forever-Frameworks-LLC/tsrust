# Development requirements

## Active product direction — 2026-10-26

The owner approved the backend application platform pivot. Follow
`docs/agent-goal.md` and `docs/plans/backend-platform-pivot.md` for active scope.
Keep Tsrust as the native backend compiler; use React/Vite with shadcn for web
and Electron, Expo with React Native Reusables for mobile, and named generated
clients/hooks over TanStack Query. Electron Forge's Vite plugin is owner-approved.
Frontend projects remain user-owned: users can manage dependencies and scripts
directly. Tsrust supplies setup/import, optional orchestration, generated clients
and scoped integration migrations. Support existing/external projects and multiple
web mounts without forcing one layout or package manager.
Preserve the old UI source and acceptance history in place. Universal rendering,
Table/component expansion and embedded UI are frozen, not prototype prerequisites.
This direction supersedes old UI continuation priorities and the local skill's
prohibition on introducing React. Backend acceptance, ownership, editor agreement
and analytics/privacy requirements remain in force.

## Local agent continuation

For autonomous continuation, read `.agents/skills/tsrust-development/SKILL.md`
and `docs/agent-goal.md`. Use project-local Ponytail for implementation decisions
and Caveman for concise chat updates; retain full technical detail in durable
evidence. These preferences never weaken the acceptance requirements below.
Routine checkpoints are local Git commits. Do not push on a timer, create release
tags for testing, or use published releases as a development loop. Follow the
remote-validation decision in the project skill before triggering GitHub Actions.


Use [the development workflow](docs/development-workflow.md) for feature acceptance.

One agent works in this tree at a time. Use Git commits as source checkpoints before risky edits and after accepted gates. Inspect status first and preserve unrelated changes. Do not create archive directories or file snapshots. The owner deleted historical archives. Keep SQLite as the durable analytics store. Public `benchmarks/measurements.sqlite` contains only indexed benchmark measurements and allowlisted provenance; export it with `benchmarks/storage/public.py`. The legacy ignored `benchmarks/records.sqlite` contains mixed historical evidence and must not be published. Do not import routine command logs, source snapshots or session receipts into analytics. Preserve existing historical records; retain only concise disposable current-gate logs. Rebuild executable consumers from commits when needed. Reuse one Cargo target, disable incremental compilation with CARGO_INCREMENTAL=0, and prune obsolete build/test/native outputs before handoff rather than accumulating them across gates.

- Format the entire project before every source checkpoint with `node tools/format-project.mjs`, using the current verified compiler via `TSRUST` when needed. Keep formatting opinionated and automatic. Verify token preservation, idempotence and formatted-source behavior; do not restrict formatting to newly edited files. The command refreshes benchmark source fingerprints only after proving code tokens unchanged. Preserve protected owner edits and never treat formatting as permission to change their semantics.
- Prioritize implementation and batch validation by complete sections. Add focused regression witnesses while implementing, but defer running checks and tests until the coherent section is complete across compiler, native behavior, editor, examples and benchmark support. Run an earlier check only when a specific uncertainty blocks implementation. After section validation, repair failures and rerun only affected checks. Do not run the whole compiler, LSP or benchmark matrix after routine edits or individual gates.
- Run complete regressions and performance benchmarks at meaningful feature-family milestones, not every five gates. For example, finish the scoped Node built-in array methods across the compiler, LSP, IntelliSense, diagnostics, examples and benchmark support before the heavy array milestone run. Gate numbers and old checkpoint plans do not automatically trigger heavy work. An explicit user request can trigger a broader run.
- Keep tests and benchmarks ready as features land. Complete matched TypeScript counterparts, independent result oracles, current source hashes, ABI/driver support, guards and registrations during implementation. Validate representative affected cases with quick untimed checks. A later benchmark run must not require implementing or repairing support from dozens of earlier gates. Keep original measurement protocols and sample counts when timing is actually performed.
- At a meaningful milestone, run the complete compiler and LSP regressions, measure the accumulated new or affected workloads, review comparable performance regressions, then optimize. Preserve completed checks and original records. Repair/rerun affected checks and resume unrun groups instead of restarting full passes. If the user changes priorities, stop the heavy continuation and retain its completed evidence.
- Every new built-in method or property must have complete IntelliSense documentation in `crates/tsrust-lsp/src/builtins.json` before its implementation gate is complete. This applies to gates in progress too.
- Explain observable behavior first: purpose; argument meanings, types, ranges, defaults and errors; return value and receiver mutation; a concise Tsrust example and expected result; useful edge cases; native differences from JavaScript. Put a short ownership note after the behavioral explanation. Avoid generic claims such as “uses the native Rust operation.”
- Keep signatures, parameter help, completion and hover consistent. Verify documented examples with the current compiler and generated native execution when output matters. Reuse focused fixtures where practical, and include documentation/example verification in gate evidence.
- Keep SQLite as the canonical analytics store. Markdown reports are optional exports. At completed feature-family milestones, review correctness and matched native versus Node-after-tsc performance signals before optimization or further dependencies. Preserve original records and exact restoration.
- Preserve existing editor fixes and unrelated changes. Track implementation acceptance and pending checkpoint measurements separately; do not describe unmeasured workloads as measured.

IntelliSense documentation must focus on purpose, observable behavior, arguments, defaults, errors and useful examples. Keep runtime reference versions, test baselines, benchmark/checkpoint details and implementation provenance in developer evidence, not hover or parameter help. Describe relevant limitations directly without a runtime comparison narrative.

Use descriptive feature names for active source and fixture directories, such as `call-spread` and `array-literal-spread`. Gate numbers belong in milestone/evidence metadata, not active feature directory names. Preserve retained historical evidence paths.
