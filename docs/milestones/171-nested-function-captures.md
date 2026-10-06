# Gate 171: nonescaping nested function captures

Status: capture-planner checkpoint; capture execution is not implemented or accepted. Gate 170 remains the accepted
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

## Capture planner implementation checkpoint

local_capture_plan.rs now records direct captures by defining owner and declaration
start, with original spelling, declaration and use spans. Callable dependencies
carry declaration identities and remain separate from owner payloads. The lexical
resolver collects these facts for each local function instead of aborting at the
first captured occurrence. Captured programs still receive the existing explicit
pending diagnostic after planning; no new callable or ownership behavior is accepted.

An iterative two-pass strongly connected component traversal forms finite recursive
groups. Bindings are deduplicated within a group while retaining their earliest source
use. A separate iterative requirements query follows the condensation graph on demand
and deduplicates transitive bindings. It does not materialize an all-pairs dependency
matrix or guess checked access modes. Binding owner identity stays available so the
next checked stage can distinguish an enclosing capture from a group member's
invocation-local parameter or local slot. Requirements are lexical dependencies,
not proof of initialization or a storage layout. Files without local declarations
retain the existing immediate return; capture-free declarations skip graph solving.

Direct unit witnesses cover mutual recursion with a forwarder, repeated binding
captures, distinct shadowed spellings and a 4,096-node chain without recursive graph
frames. A resolver integration witness verifies that the pending diagnostic names the
captured owner rather than a sibling function. This is the planned compiler module
seam, not a replacement for the five positive capture/readiness witnesses.

The next chunk must retain these plans and captured declarations into checked
contexts rather than discard them at the staging diagnostic. Resolve binding types
and modes there, partition group-external storage from invocation-local dependencies,
and integrate reachable-use readiness before constructing borrowed environments.
Then support recursive dispatch through that finite environment while preserving
existing loan conflicts and lexical release. Owned escape remains separate.

Planner checkpoint checks pass: 14 focused library tests (including all three new
graph tests and resolver integration), five accepted lexical compiler tests, all
63 tests across the eight depth-canary suites and all 272 editor tests. The capture
suite still has one passing existing closure reference and five pending feature
failures. These results prove the planner seam and preservation of accepted behavior,
not borrowed named-function execution. Original red and final logs are retained as
SQLite gate-log records under target/editor/nested-plan-*.

The existing nested_function_declarations retained-result workload passes all five
native/node-tsc cases after the resolver change, untimed. Exact runtime-result
benchmarks/runtime/results/2026-10-06T17-05-38-605Z-verify.json is retained in SQLite.
No new runtime feature landed, so no new workload or measured claim is registered.
