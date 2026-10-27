# Unused generic alias validation red checkpoint

Accepted implementation remains c18b211. No compiler source changed during this
audit. Read AGENTS.md, workflow, both generic value plans and
plans/unused-generic-call-validation-plan.md before implementation.

The new compiler test proves all three invalid unused-alias calls are accepted:
wrong generic count, explicit application on a monomorphic alias and excess value
arguments. Strict TypeScript independently rejects all three. The borrowed-result
controls pass with parameter-access for capture-free and captured adapters. Initial
incorrect diagnostic expectations are retained as evidence, not compiler defects.

All audit logs and the exact temporary TypeScript source are imported and read back
byte-for-byte in benchmarks/records.sqlite before loose removal. No editor, full
compiler, depth or benchmark run was performed: this is a red checkpoint, not feature
acceptance. Five executable consumers remain at c18b211. No process remains live.
Preserve the same four owner fixture edits. Use explicit Git paths and one warm
Cargo target with CARGO_INCREMENTAL=0.

Continuation prompt: Continue Tsrust from this red checkpoint. Implement the physical
unused-generic-call-validation plan with lexical identity, source callable contracts
and conservative flow. Preserve three red witnesses, repair aliases without running
unused bodies, then add shadow/default/rest/editor controls and bounded acceptance.
Keep full Gate173 open until every remaining context requirement has evidence.
