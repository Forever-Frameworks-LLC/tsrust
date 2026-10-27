# Fused JSON schema decoding checkpoint — 2026-10-12

S01–S10 remain accepted. S11 and the full S21 goal remain active. This handoff
lands with the fused implementation; use Git HEAD for the source checkpoint.
Previous checkpoint: `8557ff20` (ValidationError naming and seeded decoder).

## Implemented

`schema.safeParseJson(text)` borrows text and schema, returning owned typed output.
Failures are `{kind:'json',error:JsonError}` or
`{kind:'validation',error:ValidationError}`. Callback exceptions propagate.
Whole-document syntax, Unicode, numeric-token and resource checks precede callbacks.
Native scalar/composite/union plans and refine/transform/coerce pipelines are wired.
Coercion accepts JSON-representable owned input shapes; non-data and recursive raw
input types receive explicit checker diagnostics. Recursive inputs remain required
with the recursive/lazy schema tranche, not permanently excluded from S11.

`stdlib/json/source.rs` owns the source cursor implementation. Array children stream;
objects index decoded keys and borrowed slices, preserving last-duplicate behavior.
`emit/schema_decode*.rs` builds typed values directly and preserves callback
commitment. Dynamic values are constructed only when explicitly requested as
retained extras or JsonValue callback input. Ordinary branches can revisit source
without cloning input. Allocation costs still require family measurement.

Focused evidence: six public compiler/native cases, four cursor runtime cases,
one editor case covering hover/completion/result/signature help, 26 workload
contracts, and native/Node-after-tsc parity at n=0,1,17,2000. Whole-project formatting
passed. Exact failed and successful logs are preserved in SQLite, listed in the
schema plan. No timing, full regression or S11 family acceptance is claimed.
Installed consumers remain accepted S10.

## Next work

1. Fix direct constructor chaining: `s.union([s.string(),s.string()]).min(3)`
   currently fails receiver analysis. The intermediate-variable form works and
   its bound is checked. Preserve that witness; add the direct form as a regression.
2. Continue required recursive/lazy composition, collection schemas, richer issue
   APIs and JSON Schema export. Reconcile source and plan; backend prototypes do
   not establish public acceptance.
3. Complete S11 family regressions, differential requirements, matched timing,
   optimization review and consumer refresh. Then S12–S21 in dependency order.

Use one agent and batch checks after coherent sections. Shared Cargo target:
`/private/tmp/tsrust-gate100-editor-target`; `CARGO_INCREMENTAL=0`, offline/locked.
Whole-project formatter uses that target's `debug/tsrust`. SQLite remains canonical.
Preserve the six unrelated owner documents listed in the pipeline handoff.

## Copyable continuation prompt

```text
Use $caveman. Continue the active Tsrust goal through S21 in
/Users/seancotter/Desktop/dev/learning/tsrust. Inspect Git status and read AGENTS.md,
docs/development-workflow.md, docs/plans/standard-library-roadmap.md,
docs/guides/language/standard-library-coverage.md, docs/plans/schema-plan.md and
docs/handoffs/2026-10-12-schema-fused-json.md.

S01–S10 are accepted. Fused safeParseJson has focused acceptance; S11 family timing
and remaining public requirements are open. First fix direct union-constructor
constraint chaining, then finish recursive/lazy composition, collection schemas,
richer issue APIs, JSON Schema export and S11 acceptance. Continue S12–S21 in order.
Preserve approved semantics and unrelated owner files. One agent; implementation
sections before batched checks, complete editor/examples/workload support, whole
project formatting before checkpoints, shared Cargo target without incremental
compilation, and SQLite evidence. Keep the goal active while required work remains.
```
