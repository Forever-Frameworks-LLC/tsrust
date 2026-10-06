# Gate 171: nonescaping nested function captures

Status: accepted. Nested named functions read and change the variables of their
enclosing functions, natively and under the existing ownership rules. The sections
below are the chronological record; where an early section says a part is pending,
the acceptance section at the end is current. The [guide](../nested-functions.md)
describes the language behavior.

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

## Typed lexical projection preparation

capture_context.rs now owns the typed enclosing-context projection used by ordinary
closure analysis. Defining contexts record source declaration starts separately
from the active spelling map. Projection remaps those identities to capture slots,
so a hidden outer binding remains distinguishable from a shadow with the same name.
Source starts already share the linker's rebased source-location namespace; they are
not checked-function array positions. Selected slots retain their checked types and
callback metadata while enclosing argument defaults, omission ABI, transfer/use
state and outer flow/loan state are excluded as before. Parameters and locals added
after projection remain invocation-local through the existing capture boundary.

View roots now use one outer-to-inner lookup table rather than searching the visible
binding vector for every root. This changes compiler metadata handling, not runtime
ownership: no owner data is implicitly cloned, no shared cells are introduced and
no environment is materialized at a declaration. The two focused tests exercise
shadow identities, sparse/reordered slots, scalar copy access, root remapping and
separation from enclosing invocation state. They failed before projection was
implemented and pass with the shared path.

The next step is to resolve planned named captures against these typed defining
contexts, check their bodies and access contracts, and build finite group layouts.
The pending checked-environment guard remains. Typed call-failure readiness and all
four native named-capture execution witnesses are still unfinished; this preparation
does not accept gate 171 or add a benchmark workload.

Projection preparation evidence: 18 planner/readiness library tests and two typed
projection tests pass; all eight depth canaries pass 63 checks; the whole editor
crate passes 273 checks. The lexical suite retains five accepted passes and two
future-stage failures for captured escape and omitted-result inference. The capture
suite retains four passes and five pending failures, unchanged from the previous
checkpoint. No pending feature witness was weakened.

Five matched lexical workload cases pass native/node-tsc untimed. Canonical result:
benchmarks/runtime/results/2026-10-06T17-57-50-610Z-verify.json. Its startup,
compilation and execution sample arrays are empty. Fifteen original log/result
records, including the corrected red test-construction failure, are imported and
read back exactly in SQLite under target/editor/capture-context-* and that result
path. Five consumers were atomically refreshed and hash verified. Thirty-one
example directories and 51 scratch executables were pruned; libraries and current
consumers remain. No full compiler regression or timing was performed.

## Typed named-body checkpoint

named_capture_context.rs connects planned direct and transitive captures to typed
defining slots on direct calls. Group requirements are propagated once through the
shared planner helper; preparing an actual used body selects its external source
identities on demand. Dependencies owned by that named invocation are excluded from its projected slots.
Captured slots follow reserved source parameters, preserving the ordinary contiguous
argument ABI. The checker recognizes the separate named-capture range for existing
scalar, array and callable access rules. Parameter defaults can see direct captured
names; transitive slots retain identity without injecting their names into the body's
lexical scope. This prevents a peer's capture from replacing an explicit parameter
with the same spelling.

Ready direct bodies now reach normal field/type checking. Five source scenarios
cover a captured private record, caller shadowing, transitive forwarding, a
forwarder's same-named scalar parameter and a captured parameter default. A module check verifies rebased locations
and private record types. An editor check verifies the actual field error and exact
source range. Checked-context witnesses verify read access, array mutation and
scalar mutation separately from the source parameter ABI.

The environment guard now follows typed body checking, with guards for unresolved
defining contexts and captured callable-value transport. Captured checked trees do
not reach emission. No declaration creates a runtime borrow or transfer during this
stage. Four native capture positives and typed call-failure readiness remain pending;
recursive capture access still needs its group lattice and finite layout. Gate171 is
not accepted, and owned escape/result inference remain later stages.

A shadowing witness exposed two ordinary checker defects. Inner blocks rejected
same-named outer variables, and arrow parameters rejected capture spellings. Block
entry now keeps a narrow shadow journal and restores outer names at exit; the source
slots remain distinct. Arrow parameter reservation uses its own name map before
default visibility is restored. Function-body parameter redeclarations remain errors.
Native assertions verify outer scalar restoration and arrow parameter independence.
The matched lexical workload now retains a seventh observation for block shadowing,
arrow parameter shadowing and restoration. Both source fingerprints, source guard,
registration checks, independent oracle and coverage exports are updated. Existing
measurement protocols and iteration counts are preserved; this workload is unmeasured.

Typed named-body checkpoint evidence: 21 focused library tests, five module/capture checks,
63 checks across all eight depth canaries, 274 editor tests, one registration test and five matched lexical
cases pass. The whole editor result combines retained passed suites with the repaired
stdio suite and previously unrun suites. The editor assertion's expected column and
the unit scalar slot's source let flag were corrected; original failed logs remain.
Captures retain six passes and five pending feature failures. The lexical declaration
suite retains five passes and two pending future-stage failures. No pending witness
was weakened or converted into an accepted unsupported contract.

Canonical final untimed result is
benchmarks/runtime/results/2026-10-06T18-23-58-402Z-verify.json. All 107 compiler source
fingerprints match current files, and no startup/compilation/execution samples exist.
The earlier untimed result 2026-10-06T18-16-08-756Z-verify.json is also preserved.
Thirty-two original logs/results are imported into SQLite and read back byte for
byte before loose-copy removal; logs use target/editor/named-context-* paths.
Five consumers are atomically refreshed and hash verified. Thirty-one example
directories, 52 scratch executables and the small runtime-comparisons build tree are
pruned; libraries and current consumers remain. No full compiler regression or timing.


## Typed exceptional readiness checkpoint

The failing-call capture witness now passes without syntactic effect guesses.
The internal checked tree retains source capture plans until final typed readiness.
Both public analysis entry points keep an environment barrier after assignment
checking; incomplete captured programs cannot reach emission. Actual checked closure
and method bodies are used, with defining declarations distinguished from hidden
projected capture slots. Failure before assignment and in assignment RHS expressions
keeps the catch-path capture unassigned. Catch/finally assignments establish readiness.
The editor reports the exact named-call range with the captured declaration related.

Focused readiness library tests3, all eight canaries63 and the whole editor275 pass.
The capture suite has seven passes and four genuinely pending native environment
failures. No positive witness was weakened, no runtime capture acceptance is claimed,
and no full compiler regression or timing was run. Five consumers are refreshed
atomically. Continue finite capture access inference and borrowed group lowering.

Final checkpoint evidence: matched lexical cases5 pass untimed; canonical result
2026-10-06T18-53-10-079Z-verify.json. All107 compiler source hashes are verified.
Ten exact logs/results are imported into SQLite and read back before removal.
Readiness library3, depth canaries63, editor275; captures7 pass/4 pending.


## Recursive capture-access checkpoint

Named capture summaries now retain defining binding identities, hidden checked slots
and access modes separately from source parameter contracts. Static named call edges
propagate capture access into enclosing capture/parameter coordinates. A forwarder
previously left its captured scalar at Copy even when its leaf mutated it; a focused
failing witness now checks Mutate without adding supplied arguments or creating loans.

The recursive inference driver has a separate monotone capture lattice. New
coordinates extend the finite iteration budget; read access can rise to mutation or
consumption. Scalar reads remain copies, while scalar mutation propagates. A mutual
recursion test proves convergence, stable defining identities and unchanged source
argument contracts. These summaries are retained in the private checked tree for
future layout work. This phase does not form environments or establish runtime loans;
the public environment barrier remains mandatory.

Next use the settled summaries to build finite borrowed group layouts and implement
valid-use placement, reentry checks, lexical loan lifetime and native dispatch.
The four native positive capture witnesses remain pending until that stage works.

Capture-access checkpoint evidence: library25, canaries63, editor275 pass;
captures7 pass/4 pending. Matched lexical cases5 pass untimed. Canonical result
2026-10-06T18-58-52-478Z-verify.json has 108 verified compiler fingerprints
and no timing samples. 9 exact original records, including failed compile
evidence, are imported/read back in SQLite before loose removal. Five consumers
are refreshed atomically; 50 scratch executables and runtime comparison output
are pruned. Library caches remain; tests will relink. Gate171 is not accepted.


## Finite typed group layout checkpoint

named_capture_layout.rs derives deterministic fields and member-slot mappings from
settled capture access. Each recursive group deduplicates external BindingId values,
checks their typed slots agree and joins access modes. Read scalars use Copy storage,
mutable scalars use Mutate storage and owned values retain their read/mutate/consume
requirements. Recursive peers are static members, not fields containing each other.
Fields are ordered by defining identity, so shadowed spellings cannot merge.

Captures whose owner is a group member are explicitly Invocation sources rather than
shared fields. A recursive outer/child parameter witness verifies that a fresh
invocation-local value is not incorrectly hoisted into group storage. Another
witness checks two recursive peers share one mutable scalar field and one readonly
array field, with source parameter counts unchanged. Layout construction is called
only after final typed readiness and retained privately before the environment barrier.

This checkpoint does not instantiate native environments or remove the emission
barrier. Next implement ready-use placement and native group dispatch, with existing
lexical loans, mutable reentry and drop-order checks. Cross-group forwarding must
reborrow correctly without creating duplicate retained mutable references. Invocation
sources need explicit forwarding from their defining activation. The four positive
native witnesses remain pending. Owned escape remains gate172.

## Final checkpoint evidence

Focused capture library27, all eight canaries63 and whole editor275 pass. The
capture suite has seven passes and four genuinely pending native environment
failures. Five matched lexical cases pass native/Node-after-tsc verification with
seven observations each. No timing was run. Canonical result is
benchmarks/runtime/results/2026-10-06T19-04-52-509Z-verify.json; all
109 compiler source fingerprints match current files. 9 original
logs/results are imported into benchmarks/records.sqlite and read back byte for
byte before loose-copy removal.

Five consumers are atomically refreshed and hash verified. 50 scratch test/native
executables and runtime-comparison output are pruned; library caches remain. No
process is left running from these checks. Only the concise target/editor/gate171.log
remains for this checkpoint. Gate171 stays incomplete, and hourly checks remain paused.


## Native lowering checkpoint: hidden borrowed arguments

Capturing nested functions now execute. The four native witnesses pass, together
with a broader assertion fixture (`borrowed-calls.tsr`) that also passes the pinned
Node reference.

### Model

A nested function that uses bindings of an enclosing function is emitted as an
ordinary native function. Its captures are hidden parameters after its source
parameters, in the order of the checked hidden slots that
`named_capture_context.rs` already creates. A direct call passes each captured
binding from the calling body: a copy for a scalar that is only read, `&` for an
owner that is read, `&mut` for an owner or scalar that the function changes. A
caller that is itself a nested function forwards its own hidden parameter by
reborrow. `named_capture_lowering.rs` documents the model and validates uses;
`Emitter::named_capture_arguments` builds the arguments.

Consequences, each covered by a witness:

- No environment value exists. A recursive group is a set of static functions that
  forward the same hidden parameters, so its native types are finite.
- A borrow lasts for one call. A capture declared after the function is borrowed
  only at calls, which the readiness checks already require to follow
  initialization. Every block or loop entry lends its own bindings.
- A mutated scalar capture is a `&mut` hidden parameter; the hidden binding is
  marked `capture_reference`, the form closures already use.
- Calls inside arrow functions work: the arrow captures the bindings that the
  called function needs, under the existing closure loan rules, and forwards them.

The finite group layouts of `named_capture_layout.rs` are still derived and retained.
They are not needed for direct calls; they remain the basis for an owned environment
value when escaping callables are implemented (gate 172).

### Checks at a call

`Analyzer::named_capture_arguments` treats each hidden capture as an argument that
is evaluated after the written arguments. Written arguments that keep a borrow into
the call stay active while the captures are checked. Lending a binding as a mutable
argument to a function that also changes it through capture is `conflicting-access`;
an argument that only copies a value out of the binding (`tick(counts.length)`) is
accepted. Existing view, iteration and closure loan conflicts apply unchanged.
Taking ownership of a captured value is rejected until gate 172.

The call expression stores the caller slots in `Expr::capture_slots`. A per-function
table was tried first and replaced, because arrow bodies have their own slot
numbering.

### Order and unused functions

A linked program can place a lifted function before the function that declares it.
`Analyzer::capturing_function_is_unused` checks the declaring function first, so its
calls prepare the typed capture context. A capturing function that no call reaches
has no typed context: it is not emitted and its body is not type-checked. This is an
open item, not accepted final behavior: check such bodies against the declaring
function's final context.

### Fixed array storage

The fixed-shape array proof did not know the hidden arguments and kept a captured
array as `[i64; 1]` while the callee expected a growable array. The proof is now
conservative: a body that calls a capturing function keeps ordinary storage for its
arrays. A precise rule per captured slot is a later optimization.

## Acceptance

### Completed after the lowering checkpoint

- **Function values.** `values.map(add)`, `const run = tick` and passing a nested
  function to a callback parameter work. The lexical resolver marks a local function
  name outside the callee position; once every capture is known,
  `local_function_values` replaces a marked use of a capturing function with the
  arrow `(parameters) => name(parameters)`. The escape pass and closure loan rules
  then see ordinary closure syntax, so the value borrows its variables while it is
  alive, and transfers them when it is handed to an owning callback parameter or
  returned. Capture-free functions keep their direct function value.
- **Variables without a visible name.** An arrow body can call a nested function
  that needs a variable the arrow cannot name: a shadowed one, or one that only the
  called function uses. Closures of programs with capture plans now project every
  declared variable of the enclosing body, and names resolve as before.
- **Uncalled functions.** A capturing function that no call reaches is checked
  against the finished body of the function that declares it. A function declared
  inside a nested function always receives the variables it needs from that
  function, called or not (a declaration is a dependency edge in the plan).
- **Signatures.** Rest parameters, callback parameters and type parameters receive
  captures. A concrete copy of a callback-taking function, and every instance of a
  generic function, inherits the capture request of its declaration.
- **Recursive groups.** A variable owned by one member of a recursive group is passed
  only to the functions lexically inside that member; each call of the member has its
  own. The three-level shape that kept the pending diagnostic now runs.
- **Diagnostics.** A function that takes ownership of a captured variable reports
  that directly, also when nothing calls it. A generic function used as a value
  reports the arrow form to write, with the source name of a local function. No
  accepted path reports the internal pending message.

### Limits recorded for later gates

- A capturing function with optional or default parameters cannot be used as a value;
  a generic function cannot be used as a value. Both report the arrow form.
- A nested function cannot take ownership of a captured variable.
- An omitted result annotation on a named declaration still means void (gate 173);
  `nested_declarations_infer_nonrecursive_result_when_omitted` stays red.
- A body that calls a capturing function keeps ordinary array storage for its arrays.
  Precise fixed-shape storage per captured slot is an optimization.
- The gate 172 witness
  `nested_declarations_borrow_captures_and_transfer_escaping_environments` passes
  through the arrow form. Gate 172 still owes its own acceptance: factory instances,
  aggregate storage, disposal, callback failures and recursive group lifetime.

### Defects repaired in passing

- Benchmark manifest rows of gates 167 to 170 had been inserted at the front of the
  workload list, which broke the append-only prefix guards of 20 harness tests. The
  rows moved to the end in gate order; content is unchanged.
- Two harness tests evaluated counterparts in sloppy mode, where a function declared
  in a block replaces the outer one. Benchmark runs compile with `strict`, so the
  tests now do too, and `nested_function_declarations` agrees with its oracle there.
- A generic function used as a value reported `Unknown value` with the internal name.

### Evidence

- `nested_function_captures`: 17 of 17. Native fixtures: readonly later
  initialization, mutable block release, recursive forwarding, fresh loop entries,
  direct calls, function values, signatures (rest, callback, generic, three-level
  recursion) and a two-module project. Every assertion fixture passes the pinned Node
  reference.
- Affected compiler batch with all eight depth canaries, formatting, exceptions and
  function expressions: 435 pass, 1 known gate 173 witness fails. Compiler unit
  tests 308 of 308.
- Whole editor crate: 276 of 276, including hover, definition, references and
  completion of captured variables inside a nested function, and hover and definition
  of a nested function used as a value.
- Matched `nested_function_captures` workload: source contract, independent oracle,
  registration test, run, profile, oracle and report hooks, manifest row. Native and
  Node-after-tsc agree with the oracle on all five untimed cases. No timing.
- Benchmark harness: 398 of 398.
- The three guide examples run natively; the two assertion examples pass the Node
  reference.
- Logs `target/editor/gate171-acceptance-{compiler,captures,editor,consumers,verify,harness}.log`
  and results `benchmarks/runtime/results/2026-10-06T19-45-33-398Z-verify.json` and
  `2026-10-06T19-45-34-468Z-verify.json` are in SQLite and were read back byte for
  byte. Five consumers were refreshed from the accepted sources.
