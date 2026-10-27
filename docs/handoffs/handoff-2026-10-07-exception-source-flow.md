# Handoff: exception and cleanup source flow

Read AGENTS.md, docs/development-workflow.md, docs/plans/nested-function-declarations-plan.md and docs/plans/unused-generic-call-validation-plan.md first. The red checkpoint is 32bf618; the following accepted source checkpoint implements structured exception flow. Git HEAD is authoritative. Full Gate173 remains open.

## Implemented scope

The iterative graph routes try/catch/finally through separate normal, return, throw, break and continue destinations. Cleanup can replace a suspended completion. Checked-expression failure propagates the incoming environment before assignment commits. Catch failures and cleanup failures reach outer handlers. The isolated generic_call_exception_edges module caches cleanup entries by their continuation context, and source-span call facts merge across cleanup visits.

Eleven rejection cases, six precision controls, a 24-level nesting witness and formatted native cleanup controls pass. The strict TypeScript counterpart confirms integer and owned-string results, snapshot behavior, cleanup order and return overriding throw. The editor reports the invalid call and its reachable failing declaration. Unsupported statement regions and expression/capture writes retain the conservative path; they are not claimed as full source-flow parity.

Broader checks exposed nested generic closer token preservation in the formatter. Both input and output now normalize only lexer-packed closers within parsed type spans. Comparisons and shifts retain exact operator tokens. Existing adapter failures are preserved as evidence, then repaired; no tests were weakened or dropped.

## Evidence

Focused acceptance comprises 201 unique compiler tests, including all eight depth canaries, formatter tests and the fixture corpus. Whole editor passes 324 tests; final source-contract diagnostics also pass after the loop-edge refactor. Five existing generic adapter scenarios pass native and Node-after-tsc untimed. The exact record benchmarks/runtime/results/2026-10-07T04-28-17-908Z-verify.json and all /private/tmp/tsrust-source-exception-*.log outputs are retained in benchmarks/records.sqlite. Closed loose copies were read back, verified and removed. No performance timing or full compiler regression is claimed.

Project formatting is mandatory before every source checkpoint. The last pass is idempotent with no fingerprint updates. Coverage status and its generated inventory reflect structured exception support while keeping Gate173 and measurement pending.

## Next work

Start expression evaluation-order witnesses: callable writes inside sequence, receiver, arguments, conditions and short-circuit branches; assignment right sides must snapshot before replacement. The current guard intentionally defers these regions. Then audit captured write effects, stored callable fields, concrete environment replacement and lifetime/context coverage. Do not accept Gate173 until that full scope is verified.

Preserve the fifteen protected owner fixture edits from the earlier project-formatting handoff. They remain uncommitted. Use explicit Git paths. Warm target is /private/tmp/tsrust-gate100-editor-target, CARGO_INCREMENTAL=0, offline locked Cargo. Node is /Users/seancotter/.nvm/versions/node/v24.11.1/bin/node. Keep native assertion controls and matched source/oracle support ready, preserve SQLite evidence, and avoid routine heavy regressions or timing. Refresh five consumers atomically from verified builds, plus the installed CLI when needed; restart the language server to load new diagnostics.
