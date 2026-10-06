# Handoff: generic alias result-context witness

Previous implementation checkpoint: 522986e. Read its captured-deferred handoff,
AGENTS.md, docs/development-workflow.md and generic-function-values-plan.md.
This checkpoint adds a genuine red native witness and strict TypeScript reference.
No compiler source changed; gate173 and the full objective remain open.

result-context.tsr stores capture-free empty and captured fresh declarations as
unannotated values. int[] and string[] result annotations must independently infer
T. The captured factory shares its counter and prints exactly 11, 12 and 3 x.
Native checking fails on the first plain() because expected result context is lost.
Strict TypeScript compilation and Node execution pass with that exact output.
Both original logs are imported and read back in benchmarks/records.sqlite under
tsrust-result-context-red.log and tsrust-result-context-node.log. Loose logs,
reference scratch and obsolete test executables are removed. All handles terminal.

Carry expected from named-call dispatch through bound_named_call,
closure_method_call and deferred_generic_call into prepare_generic_call. Audit the
method cache too: argument types alone cannot distinguish zero-argument int[] and
string[] result specializations. Retain original type constraints and bounded cache
counts. Add invalid constraints, editor evidence, matched support and ordinary gate
records after implementation. Preserve the four owner fixture edits.

Continuation prompt: Continue Tsrust from this handoff. Implement expected result
context through generic alias calls, including captured factories and distinct
zero-argument result methods. Continue the full generic editor/storage/context
scope; do not mark gate173 complete.
