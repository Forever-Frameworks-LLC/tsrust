# Handoff: retained capture plans at checked-context boundary

/caveman Continue tsrust at /Users/seancotter/Desktop/dev/learning/tsrust. Read AGENTS.md, docs/development-workflow.md, docs/nested-function-declarations-plan.md and docs/milestones/171-nested-function-captures.md first; inspect current Git status/checkpoint.

Gate171 is not accepted. Prior planner checkpoint1269730; accepted lexical gate170 bcd7e88 and editor repair ba34914 remain. This chunk makes capture syntax survive the parser and lexical normalization. Program.local_capture_plans stores cloned/debuggable graph plans; lifted bodies remain in Program.functions. Declaration now retains function_owner, enclosing_owner and statement_span. IDs are logical source owners, not guaranteed checked-function array positions; use declaration emitted identity to resolve the actual function. The original statement anchor and enclosing owner locate the defining context and environment lifetime.

Analyzer::new currently calls Plan::pending_diagnostic before generic/type analysis. This explicitly preserves pending execution while checked environments are being implemented. Captures remain genuine positive red tests. Do not remove the guard until their binding types, readiness, ownership and native environment representations work; never allow a lifted capture body to resolve as an unrelated global.

Plan.direct preserves each declaration's direct capture identities alongside recursive group summaries. Project linker uses direct captured spellings as lexical locals without repeatedly following transitive graphs. It rebases declaration IDs, logical owners, binding declaration starts and all statement/capture spans using module offsets. Plan.node_start offsets declaration queries while group indices remain module local; multiple module plans stay independent. A source phase-boundary test is red-to-green, rebase unit checks direct/transitive locations, and a project test verifies the diagnostic after earlier module-local declarations and local/global spelling collision.

Next replace the checked guard with a dedicated local-function checking module. Resolve logical capture BindingId and source scope against typed defining contexts, distinguish group-external storage from member invocation-local parameter/slot dependencies, and infer existing access modes from bodies. Separate that checking from materialization: hoisted declarations require readiness at reachable calls/value uses, not textual declaration initialization. Reuse existing closure contracts, loans and layout without copying owners or weakening lifetimes. Form finite borrowed recursive environments/static dispatch and fresh loop/block instances. Preserve source/editor identities. Existing policy remains borrow locally, transfer supported owned escape later, scalar copy semantics; no implicit clone/shared cells. Owned escape gate172 and result inference gate173 remain pending.

Use final counts/SQLite evidence in milestone171. Preserve four unstaged fixture edits: comma-contexts/guards.tsr, map-for-each/values.tsr, map-group-by/documented.tsr, interfaces/shapes.tsr. One agent; explicit staging paths only; same /private/tmp/tsrust-gate100-editor-target with CARGO_INCREMENTAL=0 and pinned Node /Users/seancotter/.nvm/versions/node/v24.11.1/bin/node. Eight canary suites after dispatch changes and whole editor before acceptance; no routine full compiler regression/timing. Consumers refreshed atomically. No archives; durable logs/results in benchmarks/records.sqlite with exact readback. Goal active, hourly checks paused.

Final checks: library16, project1, lexical5, eight canaries63 and editor272 pass.
Capture reference1 passes; feature5 remain genuinely red. Five matched lexical
native/node-tsc cases pass untimed. SQLite runtime-result path is
benchmarks/runtime/results/2026-10-06T17-16-47-102Z-verify.json; detailed red/build/pass
logs are target/editor/nested-retain-* gate-log records. Exact readback verified
before removing loose copies. Generated examples/native outputs/test executables
pruned; keep libraries and five consumers. Gate171 still not accepted.
