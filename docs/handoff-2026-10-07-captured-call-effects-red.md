# Handoff: captured invocation effects red checkpoint

Continue from the current Git HEAD. Read AGENTS.md, docs/development-workflow.md, docs/handoff-2026-10-07-expression-order-and-assignment-invocation.md and docs/captured-call-effects-plan.md first.

The accepted compiler remains e2f5289. This checkpoint adds three focused tests without compiler changes. Eight invocation-effect cases wrongly pass; eight precision controls pass. The independent nested generic annotation witness fails because the lifted nested function loses its enclosing type environment. Preserve all reds. Implement lexical generic environment transport first, retaining explicit public arity, then the isolated invocation-effect index and ordered normal/exception summaries. Full Gate173 remains open.

Both capture-effects logs are imported with exact readback into benchmarks/records.sqlite; loose copies are removed. Whole-project formatting changed zero files and zero hashes. The fifteen protected owner fixture edits remain unstaged. Preserve them, use explicit Git paths and do not introduce implicit clones or shared capture cells. No additional editor acceptance or runtime measurement is claimed at this red checkpoint. The existing warm target and five executable consumers still represent e2f5289.

The formatter report from the owner was tested against all four current CLI consumers. Each formats the exact `type Box<T >= { value: T; };` input to `type Box<T> = { value: T; };`. The regression already exists. The current LSP delegates formatting to the compiler, but a running VSCode server's image was not verified because process enumeration was denied. Restarting the language server remains the appropriate first editor action.

## Lexical scope WIP continuation

The next checkpoint adds lexical_generic_scopes.rs and registers its module. Two focused formatted-source unit tests pass, covering declaration identity, nearest shadowing, siblings and inheritance without value captures. A native explicit-generic shadowing control passes. The new native enclosing-type integer/string instance witness is red with unknown T. The original capture-effect and nested annotation reds remain unresolved.

The scope module is not wired into generic specialization yet. Continue with contextual declaration instances as detailed in captured-call-effects-plan.md; retain original named capture provenance, explicit arity and recursive peer identity. Do not call this feature accepted or refresh executable consumers as though runtime behavior changed. Closed lexical logs, including the failing stub and native witness, are imported with exact readback into SQLite. Whole-project formatting changes zero sources or fingerprints. No dispatch or editor behavior changes occur in this foundation checkpoint, so no new whole-editor, canary or matched runtime acceptance is claimed.
