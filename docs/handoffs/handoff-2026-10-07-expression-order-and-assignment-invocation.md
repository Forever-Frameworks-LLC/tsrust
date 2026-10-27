# Handoff: expression order and named assignment invocation

Read AGENTS.md, docs/development-workflow.md, docs/plans/nested-function-declarations-plan.md and docs/plans/unused-generic-call-validation-plan.md first. The prior in-progress checkpoint is fe8d618. This following checkpoint accepts expression source order and named callable assignment invocation within the documented scope. Git HEAD is authoritative; full Gate173 remains open.

## Accepted behavior

The iterative expression-flow evaluator snapshots callees before arguments, returns assignment snapshots, joins selected branches and records failure environments at individual evaluation steps. Thirteen rejection cases and nine precision controls pass. The parser recognizes explicit generic calls on grouped callable expression forms while retaining numeric comparisons. Optional expression regions and closure/captured effects still use conservative validation and remain required work.

assignment_callee.rs stages a named callable replacement as an ordinary discarded assignment, then invokes the same destination. Existing assignment, loan, consuming-call and disposal rules apply. Native integer and owned-string results pass; captured state survives self-assignment and failure. Fresh instances replace the previous owner, and O0/O3 checks observe exactly two original records, once each. Once invocation returns an original owned record; subsequent consumption and argument-side receiver replacement violations are rejected. Record/array assignment callees and owned assignment results escaping still need their storage/lifetime work. Do not introduce implicit clones or claim full assignment parity.

Checked call facts supply concrete expression-callee parameter help. The missing-help red is retained and repaired. Aggregate whole-editor coverage is 326 tests: preserve the unaffected completed groups and the repaired 184-test intelligence group. Compiler coverage is 207 unique bounded tests, including eight unfiltered depth canaries, formatting and fixture corpus. The earlier filter-only command is retained but not counted as depth evidence.

## Matched support and evidence

The existing unmeasured generic_value_adapters family appends four observations for 120 per scenario, covering self-assignment and fresh captured factory replacement. Both sources, independent oracle, fingerprints, source guards and mutation rejection are current. Existing oracle/run/profile/report hooks already route the family. Operation and sample counts remain unchanged. Five native/Node-after-tsc cases pass untimed in benchmarks/runtime/results/2026-10-07T04-54-09-224Z-verify.json, stored exactly in SQLite. Original 116-observation records remain immutable. No timing or complete compiler regression is claimed.

Closed /private/tmp/tsrust-assignment-callee-*.log records, including failed harness/registration/stale-consumer checks and repaired runs, are imported and verified in benchmarks/records.sqlite. Loose copies are removed. The final whole-project formatter reports zero files and zero hash changes. Coverage status, inventory, milestone, guide and roadmap reflect this checkpoint without accepting the full gate.

## Next work

Start capture-effect source witnesses. Distinguish declaring a nested function from invoking it; uncalled bodies must not mutate the caller's abstract state. Known calls must propagate captured writes in evaluation order, while unknown or escaped invocations remain conservative. Cover lexical shadowing, recursion, callbacks, snapshots and exception exits with explicit tests. Then finish optional expression regions, stored callable fields, concrete environment replacement and remaining lifetime/context coverage before Gate173 acceptance.

Preserve fifteen protected owner fixture edits from the earlier formatting handoff; they remain uncommitted. Use explicit Git paths and format the entire project before every checkpoint. Warm target is /private/tmp/tsrust-gate100-editor-target, CARGO_INCREMENTAL=0, offline locked Cargo. Node is /Users/seancotter/.nvm/versions/node/v24.11.1/bin/node. Refresh five consumers atomically from verified builds, plus the installed CLI; restart the language server to load current diagnostics and help. Keep SQLite canonical, retain failed evidence, prune generated outputs and avoid routine heavy regressions or timing.
