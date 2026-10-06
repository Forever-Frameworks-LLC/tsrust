# Handoff: named capture source readiness precheck

/caveman Continue tsrust at /Users/seancotter/Desktop/dev/learning/tsrust. Read AGENTS.md, docs/development-workflow.md, docs/nested-function-declarations-plan.md and docs/milestones/171-nested-function-captures.md first; inspect current Git status/checkpoint.

Gate171 remains incomplete. Retention checkpoint542069e preserved capture bodies/plans/logical owners/statement anchors. This chunk adds local_readiness.rs and integrates it at Analyzer::new before the pending checked-environment guard. Required binding identities propagate once over condensed groups. Plan.owners maps logical owners to function names; Plan.uses maps variable expression starts to BindingId. Project linker qualifies owner names and rebases keys/identities/spans. The definite-assignment adapter recognizes callable aliases for each captured binding and preserves ordinary variable behavior. Only a whole-name assignment to the resolved binding initializes it; shadow/indexed writes do not. Branch/return/loop/catch/finally behavior reuses existing flow.

The original early-call diagnostic now passes, marking the named-function use with a related captured-binding location. Transitive forwarder and shadow-assignment error witnesses pass too. Existing arrow borrowing native reference passes. Four positive native named-capture cases still fail at the environment guard. A fifth pending witness, checked_call_failure_keeps_capture_unassigned_on_the_catch_path, requires typed call-failure effects (fail() throws before value assignment then catch calls read). It remains genuinely red. Source precheck cannot replace final typed readiness/effect/control-target checks. Never turn the positive red witnesses into accepted unsupported contracts.

Source closures currently share their enclosing logical owner in lexical resolution. The precheck locates the innermost source closure containing the binding and checks its invocation body with parameter readiness, rather than treating local initialization as outer creation. This uses temporary compiler AST copies only. Typed context integration should avoid repeated copies and use actual checked lexical binding identities. No runtime owner clone or shared capture cell is introduced. Existing owner policy remains borrow locally, transfer supported owned escape later, scalar copy semantics.

Next implement typed local-function contexts and finite borrowed group environments. Resolve logical IDs/source anchors against typed defining contexts, partition group-external captures from invocation-local slots, infer existing access/storage contracts and run final readiness after typed call/error effects and control destinations are available. Then materialize only at valid use points with current loan checks; recursive peers use one finite environment and static/view dispatch, not fields containing each other. Preserve defaults/generics/source/editor identity and fresh loop/block environments. Owned escape gate172; result inference gate173. Keep the checked guard until this works, rather than allowing lifted bodies to bind unrelated globals or eagerly lowering hoisting to const-arrow declarations.

Use final milestone counts and SQLite records. Preserve four unstaged fixture edits: comma-contexts/guards.tsr, map-for-each/values.tsr, map-group-by/documented.tsr, interfaces/shapes.tsr. One agent, explicit staging paths only. Same warm Cargo target /private/tmp/tsrust-gate100-editor-target and CARGO_INCREMENTAL=0; pinned Node /Users/seancotter/.nvm/versions/node/v24.11.1/bin/node. No routine full compiler regression/timing/archives. Five consumers refreshed atomically; existing running editor server needs restart to load the new binary. Goal remains active and hourly checks remain paused.

Final evidence: library18, lexical5, project1, canaries63, editor273 pass. Compiler
capture suite4 passes/5 pending failures as described above. Five matched lexical
cases pass untimed. SQLite runtime-result path:
benchmarks/runtime/results/2026-10-06T17-40-26-459Z-verify.json; exact red/build/pass logs
under target/editor/nested-readiness-* verified before loose-copy removal. Outputs
pruned; retain libraries and five consumers. No gate171 acceptance yet.

When investigating native materialization, keep environment formation separate from
body/type checking. Unused declarations must not borrow or transfer captures. A
borrowed environment that stays live must preserve current scope loans rather than
silently release them after each direct call. Prove conditional placement and Rust
drop order with native witnesses; a slot declared before a captured owner can have
an invalid drop lifetime even if initialized later. Do not eagerly create references
before captures are ready. The retained statement/enclosing-owner anchors are needed
for placement, and typed closure contexts should replace source-body copying.


## Editor follow-up

The interface type-reference highlighting repair is complete: annotations, aliases,
array elements and nested generic arguments have type scopes. Real TextMate tests
and extension compilation pass; the local VSIX is installed with exact source
verification. Reload the VS Code window to activate it. See the gate 166 milestone
for evidence. Resume the typed capture-context work described above; this editor
repair does not accept gate 171. Preserve all four unrelated fixture edits.
