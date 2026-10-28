# Gate 162: comma receivers and stable equality guards

Comma expressions can select named array receivers for push/removal, fill/copyWithin
and reverse. Prefix effects run once in order; the selected source retains its
original ownership, readonly access and indexed parent loans. Existing selection
methods such as join, slice and map also retain their checked behavior.

Null and undefined sentinels, typeof operands and literal typeof labels can be
selected through comma expressions. A comparison narrows its observed source only
if evaluating the later operand leaves that source stable. Readonly calls retain
the proof, while a write on either reaching branch invalidates it. This also repairs
an existing primitive-union equality defect: comparing a copied record field with
a mutating function result previously narrowed the changed field using its old value.
The independent red witness is retained.

## Implementation and repairs

Comparison checking inserts a private descendant flow fact at the source place.
Existing mutation invalidation and branch joins track its survival, without a new
write-analysis subsystem or artificial loans on copied scalars. The marker cannot
name a source field; cleanup removes it from current and cached condition outcomes.
Dedicated helpers keep guard construction outside recursive analyzer frames.

Selection loans now follow final comma operands and nonnull wrappers before deciding
whether an indexed/field parent must remain borrowed. A draft bare-owner rejection
was corrected because a bare binding transfers ownership; the actual indexed-borrow
witness rejects. The wrapped-index parent-mutation witness exposed a real missing
loan and passes after repair. Named receiver argument staging remains permitted.

Editor coverage exposed missing inferred type for a narrowed primitive length field.
Checked type projections now publish string/array length results, preserving compiler
proofs rather than deriving independent editor narrowing.

The nullable depth canary exposed oversized indexed-source parsing overflowing before
its diagnostic. A 100-deep valid source passed analysis and emission, but 200-deep
rejection overflowed. Profiling found a 3.4KB comma parser frame retained per nested
index. Splitting later operand construction into comma_tail removes that frame from
ordinary nested indexes. Both cases pass without increasing worker stacks or changing
source limits. The parser canaries remain part of the retained acceptance evidence.

## Focused evidence

- Compiler: 58 distinct checks across coherent retained batches: comma_contexts 10,
  comma_expressions 4, array_fill_copy_within 4, array_removal 16,
  nullable_narrowing 4, union_array_elements 6, higher_order_function_values 6 and
  type_syntax 8. The final loan repair reruns the four affected receiver/comma suites
  (34 checks); the other 24 checks retain the completed parser/guard evidence.
- Editor: two focused checks pass for inferred length/result hover, comma receivers,
  selected typeof labels and idempotent checked formatting. This syntax/context gate
  adds no built-in catalog entry. The guide is docs/guides/language/comma-contexts.md.
- Native receiver and guard fixtures use std:testing and pass pinned Node reference
  assertions. They check prefix order, mutation, copies, mapping, selected guards and
  a copied comparison whose original field becomes null during RHS evaluation.
- Accepted-manifest harness 389/389 passes on the final refreshed consumers.
- Five matched native/node-tsc cases pass in
  runtime/results/2026-10-06T12-52-37-916Z-verify.json, verifiedOnly with zero timed
  execution rows. Workload sources, independent arithmetic/result oracle, source
  hashes/contracts, registration and run/profile/report hooks are complete. The
  TypeScript counterpart uses an erased string assertion after the same runtime
  condition because TypeScript does not narrow through the selected literal label.
- Debug/LSP and release consumers are rebuilt and refreshed atomically. The workload
  passes check before refresh. Exact gate161 originals remain under
  archive/comma-contexts-consumers-start; final hashes are recorded in
  target/editor/comma-contexts-consumers.log.

Red, repair, build, verification and final harness logs are retained under
target/editor/comma-contexts-*. Compiler/runtime/record snapshots preserve relative
paths. No full regression or timing was performed. The workload remains unmeasured;
feature-family measurements are deferred. Earlier failed harness/verify attempts
remain evidence and are not acceptance results.
