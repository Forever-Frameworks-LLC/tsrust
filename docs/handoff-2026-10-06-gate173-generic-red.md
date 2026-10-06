# Handoff: generic function value red witnesses

Continue from 2f4ad5d and this source checkpoint. Read AGENTS.md,
docs/development-workflow.md, docs/nested-function-declarations-plan.md and
docs/generic-function-values-plan.md. Gate172 is accepted; gate173 is open.

Four owner fixtures remain untouched and uncommitted: comma-contexts/guards.tsr,
interfaces/shapes.tsr, map-for-each/values.tsr and map-group-by/documented.tsr.
Explicit Git paths only. One agent in the tree. Hourly checks remain paused.

No compiler implementation changed in this checkpoint. Four new native witnesses
are red because generic declarations cannot be used as values. Four strict
TypeScript counterparts pass Node-after-tsc. The tests retain capture-free
polymorphic calls, fixed contextual specialization, shared local captured state
and independent returned factory environments. The factory expects exact stdout
11, 21, 12; the other tests assert without output. A separate local captured
witness prevents the capture-free rejection from hiding that boundary.

The implementation plan separates contextual construction, deferred data
specialization and retained environment/storage integration. Contextual support
alone must not become the final feature or gate acceptance. Existing closure
polymorphism substitutes callable parameters; it does not make unknown data
parameters generic. Do not merely remove Header/expression_name guards or rewrite
T as an inferred arrow parameter. Preserve one environment across specialized
methods and all ownership rules.

Next implement contextual construction in a dedicated generic_function_values
module, using the original template solver and checked signature metadata. Keep
all four original red witnesses until the complete feature passes. Then complete
deferred polymorphism and factories, direct ownership/disposal/failure/context
checks, editor/docs and matched benchmark support. The wider gate173 audit and
meaningful family regressions/performance review remain outstanding.

Use CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target,
CARGO_INCREMENTAL=0 and offline locked Cargo. Node:
/Users/seancotter/.nvm/versions/node/v24.11.1/bin/node. Refresh five consumers
atomically after compiler changes. No source changes here required consumer
refresh, canary reruns or whole-editor reruns. Existing rest checkpoint evidence
remains in SQLite. Use --verify; unknown flags now fail before timing.

Durable evidence: 3 original logs and four actual TypeScript reference inputs
were imported into benchmarks/records.sqlite and verified byte-for-byte. Logs use
target/editor/generic-values-*.log record paths. Reference inputs use
tsrust-generic-values-reference/{polymorphic,contextual,factory,captured}.ts.
Loose inputs/logs and generated Node scratch were removed after verification.
Pruned two obsolete test executables; library caches and five consumers remain.
