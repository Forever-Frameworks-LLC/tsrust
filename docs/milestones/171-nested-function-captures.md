# Gate 171: nonescaping nested function captures

Status: red checkpoint, not implemented or accepted. Gate 170 remains the accepted
lexical foundation. Existing capture ownership is the settled language contract.

The focused nested_function_captures suite separates this stage from owned escape.
Four executable fixtures cover a readonly owner initialized after its declaration,
reusable mutable array capture with block-exit release, captured mutual recursion
with a transitive forwarder, and fresh mutable environments on each loop entry.
Independent strict TypeScript compilation and Node execution pass all four assertion
fixtures. The native witnesses fail at the explicit pending capture diagnostic.
A fifth witness requires an unassigned diagnostic at an early call, with the captured
binding location. It currently fails because capture support is rejected earlier.

The existing arrow-closure reference witness passes native checking and execution
for readonly record reuse and mutable array loan release. It establishes that these
contracts already exist and should be reused rather than replaced by shared cells.
An initial reference witness omitted the required record annotation; that test
construction error was corrected to a named User type. Failed evidence is retained.
Final focused result: one reference check passes, five feature witnesses remain red.
No compiler source changed, and there is no claim of feature or benchmark acceptance.

## Next implementation chunk

Separate lexical binding and declaration identity from emitted names in
local_functions.rs. Introduce local_capture_plan.rs for direct capture dependencies,
transitive callable dependencies and recursive strongly connected components.
Identify captures by defining binding and owner, not spelling alone; record source
use and declaration spans for diagnostics. The capture planner must preserve
nested shadowing and parameter/default scopes and exclude sibling function names
from owner payloads. Use iterative graph traversal to avoid new recursion depth
pressure. Keep the capture-free fast path.

Retain captured declarations until checked context types and access modes exist.
The current resolver statically lifts declarations before Analyzer::closure has
access to the defining bindings. Simply adding hidden untyped parameters or replacing
each declaration with a const-arrow would lose inference, hoisting or ownership.
Choose an explicit checked declaration/environment representation instead. Reuse
Analyzer::closure capture contract discovery, CheckedClosure capture layout and
callable transport where those invariants hold; keep ordinary analyzer dispatch thin.

Readiness must be checked at reachable call and callable-use sites, rather than
requiring captured data at the textual function declaration. Existing closure
analysis borrows at creation, while definite_assignment.rs traces captures through
closure creation. Extend that flow contract explicitly for hoisted declarations.
A declaration alone does not initialize or borrow an unavailable native slot. Direct,
transitive and recursive-group uses must prove required bindings assigned on every
path before environment materialization. Produce call/use and binding locations.
After types and readiness are proven, form one finite borrowed environment for each
recursive group; callable views or static dispatch must not recursively embed sibling
functions or copy owners. Materialize fresh environments per block/loop entry and
release existing loans at their lexical boundaries. Owned escape stays gate 172.

Add access-conflict, transferred-owner, nested forwarding, conditional readiness and
recursive mutable reentry witnesses as those seams land. Run all eight depth canaries
after parser/analyzer dispatch changes. Accept only after the native capture witnesses,
editor facts/diagnostics, guide, whole editor suite, complete matched retained-result
benchmark support and untimed oracle verification pass, with SQLite evidence and
consumer refresh. No timing or full regression until the named-declaration family
milestone described in nested-function-declarations-plan.md.

Original logs are SQLite gate-log records under target/editor/nested-capture-*,
verified byte for byte before loose copies were removed. No archives. Keep all four
unrelated worktree fixture edits unstaged, including interfaces/shapes.tsr.
