# Handoff: caught generic assertions repaired

Source checkpoint 8af3e12 registers the internally generated runtime-error record in the analyzer type lookup. Generic calls using caught values now resolve their inferred annotations. This fixes the reported internal unsupported-type error without changing ownership or exposing a public runtime-error type.

Focused formatted project/native checks cover integer and string payloads, generic borrowed observers, identity and runtime bounds errors at optimization levels 0 and 3. The editor project witness passes hover and signature help. All 25 affected exception tests and eight depth canaries pass. Whole compiler/editor milestone evidence from named-declaration-family.md remains retained; this bounded repair does not repeat those full runs or timing.

All five executable consumers are atomically refreshed from 8af3e12 and pass combined caught-assertion, path/OS and independent callable-state smoke checks. Canonical SQLite records use tsrust-caught-generic-assertion-* identities. Original failures, test-authoring corrections, final results, build logs, consumer hashes and smoke source are imported and read back byte for byte. Loose copies are removed and closed test executables pruned. Fifteen owner fixture changes remain untouched and uncommitted.

## Next work

Investigate large generic adapter compiler efficiency using the preserved specialization/worklist/state-join sample from the preceding family handoff. Do not speculate from a stack sample or weaken ownership to reduce work. Create a bounded failing performance or operation-budget witness before changing algorithms; preserve semantic controls and original measurements. Scoped aggregate composition and named function expressions remain separate work.

The std:path.relative worker must integrate this newer source checkpoint before publication. Obtain exclusive main ownership, verify combined effects and editor contracts, import worker evidence without replacing main SQLite, refresh consumers, then return ownership. See plans/parallel-path-relative-plan.md.

## Continuation prompt

Continue Tsrust from this handoff and docs/handoff-2026-10-08-named-declaration-family.md. Read AGENTS.md and docs/development-workflow.md first. Preserve fifteen owner fixtures and use explicit Git paths. Caught generic assertions are accepted at 8af3e12; do not reopen their solved witness. Next investigate sampled generic specialization joins with a bounded test, or integrate std:path.relative after exclusive ownership handoff. Node 24.11.1 first on PATH; /opt/homebrew/bin/python3 for SQLite; reuse /private/tmp/tsrust-gate100-editor-target with CARGO_INCREMENTAL=0. Give a physical handoff at the next checkpoint.
