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

## Retained-plan and phase-boundary checkpoint

Captured declarations now survive parsing and lexical normalization. Program stores
the capture plans and lifted bodies; declaration metadata retains logical function
owner, enclosing owner and original statement span. Capture identities are logical
source owners, not checked-function array positions. Checked analysis receives this
representation and currently issues the explicit pending environment diagnostic
before generic/type analysis. Capture execution and readiness are still unimplemented.

Plans retain direct per-declaration captures as well as recursive group requirements.
The linker protects direct captured names from global qualification, including a
captured local spelling shared by a module function. It does not query every group's
transitive requirements to link individual bodies. Module linking rebases declaration
identities, logical owners, statement/capture spans and binding declaration starts;
each module's condensed graph retains local group identity with a declaration offset.
Original source locations remain available for diagnostics and later checked binding
resolution. Captured bodies are never accepted as unbound global functions.

A red-to-green phase-boundary witness checks that capture syntax parses, retains its
body/plan/owner/anchor, and reaches the checking guard. A graph witness verifies
rebased direct/transitive requirements and use/declaration locations. A multi-module
witness checks the pending diagnostic after prior capture-free local declarations,
with a captured local shadowing a module function. The next chunk should replace
the checked guard with typed capture contexts, reachable-use readiness and finite
borrowed environments. Do not mistake retained syntax for supported execution.

Retention checkpoint checks pass: focused library16, module-location1, lexical5,
all eight canary suites63 and whole editor272. Capture suite still has one passing
existing reference and five pending feature failures. The existing lexical workload
passes five matched native/node-tsc cases untimed. Canonical runtime-result is
benchmarks/runtime/results/2026-10-06T17-16-47-102Z-verify.json; original red, build and
successful logs are target/editor/nested-retain-* gate-log records. Every record is
read back byte for byte before its loose copy is removed. Five consumers refreshed
atomically and hash verified. No full compiler regression or timing was performed.

## Source readiness precheck checkpoint

local_readiness.rs now checks named function uses against the initialization of
required captured bindings before the pending checked-environment guard. Condensed
group requirements propagate binding identities once in reverse dependency order;
uses of forwarders and mutual groups therefore carry transitive requirements without
repeated graph searches. Planner retains logical owner names and resolved variable-use
identities, rebased through modules. Assigning a shadowed binding cannot initialize
its captured namesake, and indexed writes do not initialize a deferred whole array.

The precheck reuses definite_assignment.rs completion flow through a small named
capture adapter. Initializers, plain assignment, branch joins, early returns, loop
entry bindings, catch parameters and explicit finally assignment use existing rules.
Captures declared inside source closures are checked in their innermost invocation
body; outer captured dependencies still follow existing creation-time borrowing.
This adapter changes no ordinary deferred-variable assignment behavior.

An early call or retained function value now reports unassigned at the function use,
with the required captured binding location and a concrete initialization instruction.
The original early-call witness, transitive forwarder and shadow-assignment witnesses
pass. The existing native arrow-closure reference also passes. Four positive named
capture execution cases remain red at the checked-environment guard.

This is a source precheck, not final typed readiness or capture acceptance. Typed call
failure effects and resolved control targets are unavailable at this point. A new
positive diagnostic witness for fail() throwing into catch before assignment remains
red, and must pass when readiness is integrated with typed contexts/effects. Do not
replace it with accepted unsupported behavior or duplicate the checked effect engine
with syntax guesses. Gate171 acceptance still requires that witness, typed captures,
finite borrowed recursive environments, ownership/loans, native execution, editor,
matched benchmark support and all other planned requirements.

Readiness checkpoint evidence: library18 (including15 source-flow scenarios),
lexical5, module-location1, all eight canary suites63 and whole editor273 pass.
The compiler capture suite is4 passing checks and5 pending failures: three new
initialization diagnostics plus the existing native closure reference pass; four
native named-capture positives and the typed call-failure diagnostic remain red.
The editor check verifies both function-use range and related binding location.
Five existing lexical workload cases pass native/node-tsc untimed. Canonical result:
benchmarks/runtime/results/2026-10-06T17-40-26-459Z-verify.json. Red/build/pass logs are
target/editor/nested-readiness-* SQLite gate-log records, exact-readback verified
before deleting loose copies. Five consumers refreshed and hash verified; generated
examples/native outputs/test executables pruned. No timing/full compiler regression.
