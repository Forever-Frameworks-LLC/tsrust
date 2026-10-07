# Recursive formatting checkpoint

The formatter now preserves expression continuation after object literals followed by `as` or `satisfies`, adds the required space before callable type annotations, and keeps generic types, generic calls, nested array types and indexing tight. Type contexts include unions, tuples and callback results. Token preservation and second-pass stability remain required.

`tsrust format` without paths recursively writes `.tsr` files below the current directory. Directory and multiple-path inputs also write by default. A single explicit file retains stdout behavior unless `--write` or `--check` is supplied. `--check` never writes. Discovery skips `.git`, `node_modules`, `target` and symlinks, sorts and deduplicates files, and continues formatting readable files after another file fails. Output is a concise changed-file count.

The workflow now requires formatting successful fixtures in memory and running the formatted source with the original assertions. Native helpers apply this automatically. The corpus test checks every lexable fixture for token preservation and idempotence and rechecks originally valid standalone contracts. Owner fixtures were not rewritten as a side effect.

A valid interface with collection fields and a callable field exposed missing Map/Set native header accounting in callable storage validation. Collection elements are heap allocated; their headers contribute to containing record storage. The formatter context witness now checks successfully. Map/Set factory capture controls also execute successfully.

Focused formatting checks pass: 24 tests. Corpus and all eight depth suites pass. All 317 editor tests pass. Five workspace executable consumers and the installed CLI are refreshed. Exact closed command logs are stored in benchmarks/records.sqlite. No timing was performed; formatting adds no matched runtime workload. Previous generic adapter workload registrations remain unchanged.

## Continue

Read AGENTS.md, docs/development-workflow.md and docs/handoff-2026-10-06-branch-call-contracts.md. Gate 173 is still open. Continue distinct source contract sets, loop and exception flow, expression ordering and capture writes using bounded TDD checks. Preserve all unrelated owner fixture edits and stage explicit paths only. Keep the single warm Cargo target and disable incremental compilation. Formatting verification is now part of every successful acceptance fixture. Do not run repository-wide formatting as verification.
