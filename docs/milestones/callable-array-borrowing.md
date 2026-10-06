# Borrowing callable array elements

Status: implementation in progress. This is a dependency within S06 callable containers, not a separate accepted standard-library gate. The complete roadmap and checkpoint schedule remain authoritative.

The final behavior must support iteration callbacks that temporarily invoke stored reusable environments without copying or transferring them. Bare callback types still own their environment; borrowing must use the established `read`/`mutate` contracts. A once environment cannot satisfy reusable invocation. Borrowing an element must not let it escape through a return, capture, array insertion or owning parameter.

## Native adapter

- Emit `Callback<Arguments, Result>` for reusable stored families, forwarding through their existing `&mut self` dispatch.
- Preserve each parameter's data access, result type and argument lifetimes. Do not introduce trait objects, heap allocation, clones or shared ownership.
- Do not emit this reusable adapter for once families. Do not promise `ReadCallback`: an owning reusable storage signature permits captured-state mutation even if currently registered origins happen to be readonly.
- Verify singleton and tagged families, repeated mutable borrowing, borrowed string arguments, original invocation state and once exclusion at O0/O3.

Implemented focused increments: dispatch adapter and production emitter hookup; stored reusable environments can be lent to ordinary mutate function parameters. Arrow visitors for forEach/map/flatMap and some/every/findIndex/findLastIndex now admit a mutate callable element and optional index, enforce receiver loans and emit mutable traversal through materialized visitor methods. Seeded reduce/reduceRight also lend reusable elements while retaining independent scalar, string, record or array accumulators; focused execution, cleanup and receiver-alias guards pass. Empty families used by concrete array declarations retain native types. Other method families and broader nested/generic acceptance remain pending; evidence is recorded in the container integration ledger.

## Structural source admission

- Match a stored family's structural parameter/result shape to a borrowed callable parameter while retaining declaration names separately. A kind change must be deliberate borrowing, not equality between owning and borrowed contracts.
- Preserve nominal identity, nested callable access, argument order and result identity; source spelling and aliases must not affect compatibility.
- Avoid using ordinary array-element inference on a `Type::Callback` hint. Resolve the receiver family first, then validate the callback's nested borrowed contract.
- Reject owning/once element parameters where the traversal only lends the element. Verify wrong arity, nominal type, access and result diagnostics before native emission.

## Receiver and element loans

- A mutable callable element parameter requires exclusive access to that element during its callback invocation and mutable access to the receiver traversal.
- An active element loan forbids removal, replacement, reordering, insertion and whole-array transfer, including through captures and nested call arguments.
- Do not simultaneously lend a mutable element and an overlapping source-array parameter. Keep index parameters copyable and preserve existing zero-parameter and ordinary-data callback behavior.
- End temporary invocation loans at the callback boundary; retain named closure capture loans according to their existing scope. Test later receiver reuse and early exits.
- Treat sorting separately: two compared elements need distinct mutable loans, and the sort algorithm may compare the same logical owner in ways that require an explicit safe policy. Do not reuse traversal admission blindly.

## Native traversal integration

- Use mutable element iteration only when the checked contract requires it. Preserve once-only evaluation of receiver and callback expressions.
- Adapt the stored environment to the borrowed callback parameter through the finite family adapter; no ownership wrapper may be copied or removed implicitly.
- Preserve independent results for map and explicit-seed reductions. Returning a borrowed element as an owning result must reject.
- Cover forEach, map, flatMap, some/every, index searches, reductions, optional-element searches and sorting according to their individual result/copying requirements.
- Verify mutation of captured state, invocation count/order where specified, short-circuit behavior, nested arrays, imported/private nominal types and unwinding cleanup with normal worker stacks.

## Editor, documentation and acceptance

- Hover, completion and signature help must distinguish mutable borrowing from environment transfer, keep source parameter names, and infer mapped results accurately across modules and unsaved changes.
- Replace the current callable-element parameter limitation in built-in documentation only after source/native evidence proves that method's support. Add executable examples and ownership/error descriptions for each affected method.
- Register matched native and compiled-TypeScript workloads with independent correctness oracles before container gate closure. Keep timing pending until checkpoint 101; retain original records in SQLite.
- Run focused contracts for each increment. Full regression and timing follow the existing five-gate schedule; do not restart preserved passes.

Completion requires all these source, loan, traversal, editor and workload requirements. The native adapter alone does not establish iteration acceptance.
