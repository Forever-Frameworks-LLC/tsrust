# Callback adapter integration checkpoint

Read AGENTS.md, docs/development-workflow.md, generic-value-adapters-plan.md and
the preceding benchmark handoff. Gate173 and full adapter acceptance remain open.
Four unrelated owner fixtures remain untouched and uncommitted.

Reduction now ignores deferred empty prototypes when collecting result/parameter
hints. It supplies accumulator, element, position and receiver hints to the existing
adapter helper. Sorting supplies two element types. Search already routes through
array_transform. All wrappers retain the existing environment and ordinary loans.

Nine adapter tests, 63 depth canaries, 13 existing callback tests and 298 editor tests
pass. New reduction and sort-search fixtures have strict TypeScript counterparts.
Reduction prints 1,2,3,9,4,after; sort-search prints 1,false, matching native output.
No timing ran. Matched benchmark coverage for the new callback paths is still owed.
The prior seventeen-observation workload remains intact, with historical SQLite
verification. Full editor/native callback context and ownership audit remain owed.

An exploratory factory returning an inferred record of two callable fields fails
because a record literal requires a named record context. Retain this boundary
in the wider aggregate/alias audit; do not silently treat it as supported.

Next extend the matched workload/oracle/guards for reduction/search/sort, verify
untimed native/Node results, and continue defaults/rest/pattern/nullable and arbitrary
expression contexts. Preserve all original failures and successful evidence in
SQLite. Continue full Gate173 scope rather than accepting only current tests.

Continuation prompt: Continue from this checkpoint. Complete matched callback
adapter benchmark expansion, then audit remaining adapter contexts and ownership
failures. Preserve owner fixtures and exact evidence; use explicit Git paths.

Matched callback benchmark expansion is now complete for the implemented paths.
The pair has twenty-six observations, independent oracle and semantic mutation
guards. Five untimed native/Node cases pass. Canonical SQLite record:
benchmarks/runtime/results/2026-10-07T00-02-45-912Z-verify.json. Registration passes;
oracle hashes match current files; exact readback verified before loose cleanup.
All processes are terminal. No compiler source changed in this support increment.
Next move directly to defaults/rest/pattern/result-only/nullable and arbitrary
callable expression audits; preserve the exploratory named-record boundary.
