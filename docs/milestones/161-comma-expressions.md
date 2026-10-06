# Gate 161: comma expressions

Comma expressions evaluate their operands once in source order. Each earlier result
is discarded before the next operand executes; only the final operand determines
result type and access. Parentheses allow comma expressions inside separator lists.
Declaration, argument, parameter, record and array commas remain separators.

Full expression contexts include standalone statements, return and throw operands,
conditions, grouped expressions, computed indexes, template interpolations and
for initializers/updates. The for-in source admits comma expressions; for-of retains
its assignment-expression grammar. Existing assignment-value restrictions remain:
owned replacement and string += can appear as discarded operands, but do not
produce an owning result. Destructuring assignment remains a statement operation. Comma results cannot themselves be assignment targets.

## Implementation and ownership

A flat Sequence node avoids recursive nesting proportional to the number of operands.
Parser, analyzer and emitter helpers stay outside recursive dispatch frames. Checked
walkers visit all operands for linking, callable captures/origins, exception effects
and editor facts. Prefix results use scoped native discard; completion transport
stays in the enclosing function, including throw, catch and finally.

Reading an owned prefix name does not transfer or clone it. Explicit Owned calls
still move their arguments. Discarded closure captures are released before later
operands. The final operand carries borrowed provenance, callable origin and guard
outcomes. Storage roots, readonly selection and partial-move paths follow the final
operand; prefix effects execute before final narrowing. The existing named-array
index lowering safely permits index effects before acquisition when no retained
borrow blocks them. A draft rejection for that valid case was corrected after
independent Node and native execution; it is not a new borrowing restriction.

TDD caught missing final guard narrowing, lost loans on comma-selected indexed
owners, skipped storage prefixes and whole-record moves during field selection.
The harness caught an older string += update becoming a value-expression rejection.
Discarded replacement now forwards enclosing loans and temporary declarations stay
inside each discarded operand, preserving enclosing binding visibility. The next
harness failure exposed destructuring assignment in comma-separated for updates.
For headers now retain dedicated assignment-list lowering, including destructuring
initializers, while ordinary value contexts keep the flat Sequence node. Singleton
internal discard nodes preserve read access for other header operands.
The named-receiver replacement draft was also corrected: native push stages its
argument before reacquiring named storage. The rejection witness instead uses an
indexed receiver, whose parent loan must prevent replacement. Focused fixes preserve those facts. A declaration appearing as a comma
operand retains a syntax diagnostic; the declaration-list rejection test is unchanged.

## Focused evidence

- Compiler 22 distinct checks pass: comma_expressions 4, declaration_lists 2,
  borrowed_optional_fallback 2, higher_order_function_values 6 and type_syntax 8.
  Final parser canaries and the affected storage/discard suites are retained in
  separate coherent batches. No worker stack enlargement or source limit change.
- The std:testing fixture passes natively and with pinned Node. It checks ordered
  effects, final types and values, explicit transfer, borrowed source reuse,
  partial-record field access, discarded captures, contextually typed arrays,
  return/throw/finally behavior, owned replacements and destructuring headers.
  Static rows retain explicit moves, receiver/borrow conflicts and syntax errors.
  The flat 512-operand witness passes on the normal worker stack.
- Editor 1/1 checks final result hover, contextual arrays, formatter idempotence
  and checking formatted output. This syntax feature uses the guide rather than
  adding a built-in catalog method.
- Accepted-manifest harness 388/388 passes after the string += and destructuring-header regressions
  were repaired. Prior failed passes remain evidence; rejection rows were not
  weakened to hide those failures.
- Matched native/node-tsc verification passes five cases in
  runtime/results/2026-10-06T11-59-01-717Z-verify.json. It is verifiedOnly with
  zero timed execution rows. The paired workload, independent direct arithmetic
  oracle, source contracts/hashes, registration and runtime/profile/report hooks
  are complete. Older verification records remain unchanged. Manifest gate metadata
  was brought forward from the stale value 143 to 161; original sample counts and
  protocols remain unchanged. Coverage uses the existing control.commaExpression
  inventory ID; the updater passes all 1,954 operations.
- Debug/release builds pass. The new workload and both affected older workloads
  were checked before atomic five-consumer refresh. Previous gate160 executable
  bytes remain under archive/comma-expressions-consumers-start; refreshed hashes
  are in target/editor/comma-expressions-consumers.log.

Exact compiler, runtime and record snapshots retain relative paths. Red failures,
repairs, final command evidence and Node/native assertions remain under
target/editor/comma-expressions-*. No full regression or timing. The new workload
remains unmeasured pending a language feature-family milestone.
