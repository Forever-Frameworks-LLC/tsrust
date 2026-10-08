# Handoff: resumable summary gate started

Last production source acceptance is 74eeaa0, published by 27db9fc. This checkpoint adds test-only tracing, an opt-in ignored diagnostic, a red general scheduling witness and a physical implementation plan. No production behavior or consumer binary changes. Fifteen owner fixtures remain uncommitted and untouched.

The current full release adapter check still reaches a 55-second bound. A parsed-source-only trace completes earlier roots with one evaluation, then enters workload with 181 bindings. A 25-second progress trace reaches 336 requests, 644 evaluations and 22,272 invocation visits before explicit termination. These incomplete probes are diagnostic evidence, not throughput measurements or runtime timings. Records use tsrust-worklist-adapter-* and tsrust-worklist-full-adapter-current.json in SQLite.

The new nested_independent_calls_resume_without_replaying_prefixes witness is intentionally red. Eight calls have 17 requests, 33 evaluations and 60 invocation visits; target budgets are 18 evaluations and 32 visits. The initial witness incorrectly expected duplicated inner source facts; corrected expected facts are n outer sites plus one shared inner site. Both original and corrected red logs are retained. Existing 94 generic-flow controls pass when this new red witness is excluded; the opt-in full diagnostic is ignored in ordinary runs.

Next follow docs/resumable-callable-summaries-plan.md. Extract persistent body state and prove one-shot equivalence, then record node/partition dependencies so completed caller prefixes are retained. Defaults, captured environment versions, exceptions/finally and recursive opacity propagation require explicit gates. Do not add more eager-return special cases as a substitute for the general repair.

Read AGENTS.md and docs/development-workflow.md first. Use explicit staging, pinned Node/Python, one Cargo target and CARGO_INCREMENTAL=0. Preserve the intentional red witness during parallel worker reconciliation; it is not a library regression. Main publication remains exclusive. Give a physical handoff at the next checkpoint.
