# Ownership and memory

[Documentation index](./README.md) · [Proposed roadmap](./roadmap.md) · [Source section map](./section-map.md)

## Workstream scope

**Status:** Unresolved design/research. The grouping and phase below are proposed planning aids, not approved language semantics.

**Proposed phase:** Core proof.

**Scope:** Specify moves, inferred access, copying, sharing, captures, allocation, cycles, and global state.

**Planning deliverable:** Ownership rules with accepted/rejected examples and documented inference limits.

**Dependencies to coordinate:** [principles-and-boundaries](./principles-and-boundaries.md)

**Completed closure experiment:** [Milestone 03: closures](./milestones/03-closures.md) defines provisional capture, invocation, and escape rules. Its source compiler gates are complete, with local capture loans and owned returned environments verified independently of the [handwritten witnesses](./experiments/closures-01.md).

## Current experiment

The completed [ownership proof](./milestones/01-ownership-proof.md) establishes bounded moves and inferred argument access. [Milestone 02](./milestones/02-control-flow.md) adds nested branches and early returns: availability merges over continuing paths, while parameter contracts include every arm. Literal conditions do not relax checks. Local and returned closures are now implemented: capture storage and invocation access are separate contracts, and local loans conservatively end at block exit. [Milestone 04](./milestones/04-synchronous-callbacks.md) adds synchronous reusable callback parameters. Inline captures borrow until call completion; named closure loans retain block scope. Callback parameters cannot escape, and consuming invocation cannot satisfy a reusable annotation. [Milestone 05](./milestones/05-single-use-callbacks.md) adds synchronous single-use callback parameters: passing transfers the environment, invocation or forwarding consumes the parameter, and existing branch joins enforce at most one use per path. [Milestone 06](./milestones/06-closure-arguments.md) adds copied integer/boolean closure arguments, with receiver access held during left-to-right argument evaluation. [Milestone 07](./milestones/07-owned-closure-arguments.md) adds inferred owned argument access in arrow bodies and explicit exact access in callable signatures. A reusable closure can consume fresh arguments on successive calls without consuming its environment. [Milestone 08](./milestones/08-nested-local-closures.md) allows local nested closures and factory results inside arrow bodies. Inner captures propagate access and transfer requirements through enclosing captures and parameters, while inner loans retain block or inline-call lifetime. [Milestone 09](./milestones/09-callable-captures.md) adds concrete callable captures, rejecting borrowed environment escapes. [Milestone 10](./milestones/10-callable-results.md) adds recursive callable result annotations and owned returns from arrow bodies. [Milestone 11](./milestones/11-retained-callbacks.md) adds `owned` and `owned once` callback parameters retained through owned wrappers, with exact contracts and bounded cached specialization. Callable-valued arrow arguments, callback signature results, shared aliases and non-owned callback returns, borrowed/mixed-origin returns, cross-module stability, access variance, shared ownership, and async remain open.

[Milestone 12](./milestones/12-scalar-operators.md) adds scalar operators with left-to-right evaluation and runtime short-circuiting. Copied operand access ends before the next operand, while enclosing loans remain active. A transfer that may occur on a logical right operand makes the binding unavailable afterward and contributes to capture/parameter contracts; literal conditions do not relax this analysis.

[Milestone 13](./milestones/13-while-loops.md) adds loop ownership with explicit back-edge checks. Inherited owned values cannot transfer on a path that repeats; transfers may exit through break or return. Post-loop availability joins condition effects and break states while excluding function-return paths. Iteration-local values are fresh and local capture loans end on each exit. Immutable bindings make these availability checks monotone; mutable ownership reinitialization needs separate analysis.

[Milestone 14](./milestones/14-mutable-bindings.md) adds exact-type replacement of initialized `let` bindings. A right-hand side can consume the old value, then restore availability through assignment. Every continuing branch and loop path must satisfy availability before use or repetition. Existing capture loans forbid replacement; owned environments retain old transferred values independently, and scalar captures remain copied snapshots. [Milestone 15](./milestones/15-capture-replacement.md) subsequently permits replacing inherited `let` captures. Local scalar writers hold exclusive loans; escaped scalar writers mutate private copies. Read-only scalar snapshots remain unchanged. Consuming an old capture while replacing it retains a single-use invocation contract.

[Milestone 16](./milestones/16-mutable-callables.md) permits mutable concrete callable bindings. Replacement restores availability after a consuming invocation; named assignment transfers the source environment. Captured loans, branch joins, and loop entry invariants still apply. Different environment origins are rejected before Rust emission.

[Milestone 17](./milestones/17-callable-moves.md) permits declaration moves of concrete callable environments. Invocation and argument contracts remain unchanged; the source becomes unavailable. Return chains preserve the original escape origin, and borrowed environments retain their lexical capture loans. Capturing wrappers that move an environment are single-use. Shared aliases remain unsupported. [Milestone 18](./milestones/18-callback-moves.md) adds callback-parameter declaration moves: reusable contracts transfer local borrows, owned contracts retain capture permission, and single-use contracts survive specialization. Mutable callback locals can restore values derived from the same parameter. [Milestone 19](./milestones/19-callback-returns.md) adds direct owned-callback returns with exact signature checks and parameter-origin substitution through templates. Reusable environments return unchanged; cached concrete wrappers preserve declared single-use invocation when needed without allocating. Non-owned callback returns and captures remain unsupported.

[Milestone 20](./milestones/20-scalar-arrays.md) adds exact homogeneous `int[]` and `boolean[]` types. Arrays are owned values; elements copy. Moves, explicit clone independence, parameter/capture access, lexical loans, branch joins, and loop restoration apply to whole arrays. Public callable argument signatures require explicit array access contracts. Indexing is bounds checked; writes and push mutate named arrays, including const-bound arrays. Owned elements, nested arrays, and array record fields remain outside this gate.

[Milestone 21](./milestones/21-for-of.md) adds borrowed for-of iteration with fresh const/let scalar bindings. Iterable expressions evaluate once and temporary arrays live through the loop. Named sources retain ownership and cannot be mutated, replaced, or transferred while iteration holds a shared loan. Break/continue, lexical captures, and ownership joins follow the existing loop rules; consuming iteration contracts remain open.

[Milestone 22](./milestones/22-owned-array-elements.md) adds owned string and named-record array elements. Construction, push, and replacement move owned values; indexing borrows, and explicit cloning produces independent ownership. Indexed record fields support reads and writes. Element argument loans conservatively protect the whole array. For-of borrows owned elements for reading; moves, writes, rebinding, and escaping retention receive source diagnostics. Temporary arrays, captures, callback contracts, native bounds checks, and deterministic cleanup are verified.

[Milestone 23](./milestones/23-nested-arrays.md) adds exact nested arrays such as `int[][]` and `User[][]`. Rows move into owned vectors; indexing borrows, scalar leaves copy, and explicit cloning creates independent nested storage. Indexed rows support length, push, replacement, record-field access, and mutable arguments. Nested iteration keeps row borrows lexical. Parent selections stay stable during later operand evaluation through conservative source loans. Bounds failure timing, callbacks, captures, ownership restoration, 100-level native access, and the 128-dimension type boundary are verified.

[Milestone 24](./milestones/24-array-record-fields.md) adds owned array fields with exact leaf/rank types, including forward and recursive record arrays through vector indirection. Mixed field/index paths support borrowing, mutation, arguments, and iteration. Loans protect the root owner during selected-parent operand evaluation and shared loops. Construction and replacement move arrays; explicit cloning creates independent field storage. Milestones 25 and 31 subsequently add direct record fields and direct named field extraction.

[Milestone 25](./milestones/25-nested-record-fields.md) adds direct named-record fields with inline native representation and exact nominal identity. Nested literals, updates, cloning, borrowed arguments, captures, callbacks, and iteration preserve root-owner contracts. Iterative cached layout validation rejects direct cycles and bounds inline depth/storage before Rust emission; array indirection permits recursive owned trees. Native deep access and source failure timing are verified. Milestone 31 subsequently adds direct named field extraction and restoration.

[Milestone 26](./milestones/26-recursive-functions.md) adds self and mutual named-function recursion for data parameters/results. Monotone access summaries stabilize before a fresh definitive ownership analysis. Transfers, loans, captures, callbacks outside recursive signatures, iteration, runtime failure ranges, and source guards remain enforced. Milestone 27 subsequently adds ordinary reusable/single-use parameters. Milestone 28 subsequently extends owned parameters too. Milestone 29 subsequently resolves single-origin callable results; termination and tail-call behavior remain outside the guarantee.

[Milestone 27](./milestones/27-recursive-callbacks.md) extends self and mutual recursion to reusable and ordinary single-use callback parameters with non-callable results. Exact argument/result/access contracts, declaration aliases, single-use transfers, capture loans, shared iteration, and stable data ownership summaries remain enforced. Fresh closed arrows/factory results and callback permutations are supported. Definitive typed-call analysis bounds each recursive callback function to 128 distinct concrete environment type vectors; repeated values and erased lifetimes count once. Non-owned callback parameters still cannot be captured. Milestone 28 subsequently extends owned callback parameters; milestone 29 subsequently adds single-origin callable-result cycles.

[Milestone 28](./milestones/28-recursive-owned-callbacks.md) extends named recursion to owned reusable/single-use callback parameters with non-callable results. Abstract templates preserve exact transfer, retention, invocation, and argument contracts. Concrete data-result bodies drain iteratively through the existing global 128-specialization cache; mixed ordinary parameters retain the per-function vector guard. Source arrows with matching capture layouts and callable contracts reuse native types while each construction applies fresh values and loans. Growing environment types receive source diagnostics. Milestone 29 subsequently adds single-origin callable-result cycles.

[Milestone 29](./milestones/29-recursive-callable-results.md) resolves self/mutual callable-result cycles to one concrete environment origin before definitive checking. Source branch/declaration order does not select native evaluation order. Declared callable shapes support provisional local invocation, including lazy nested factories, but cannot supply concrete returns or captures. Final bodies enforce transfers, invocation, exact signatures, all callback contracts, and source loans; discovery-only environments are removed before public metadata or emission. Mixed/missing origins and recursive inline environment storage remain unsupported.

[Milestone 30](./milestones/30-array-removal.md) adds ordered `array.remove(index)` with an exact element result. Scalar, string, named-record, and nested-array values transfer out without cloning; the remaining array retains ownership and order. Removal infers root-owner mutation and preserves argument, capture, iteration, callback, recursive, and selection loans. Named receivers permit index effects before final mutation; selected field/index parents stay stable. Negative/out-of-range indices fail with the full removal call's source byte range under every optimization profile. Milestone 31 subsequently adds direct field moves. Optional/range removal and mutable/consuming iteration remain separately scoped.

[Milestone 31](./milestones/31-record-field-moves.md) adds moves of direct named string/record/array fields with per-path availability. Siblings remain usable; missing fields block overlapping or whole-record access with related transfer locations. Exact field/parent replacement and whole let rebinding restore availability. Branch joins and loop edges retain missing-field state. Field consumption infers owning root parameter/capture contracts; detached argument values release source loans, while borrowed field arguments and existing capture/iteration/selection loans remain conservative. Native places/drop flags preserve initialization, evaluation, and bounds behavior without runtime field-state storage. Indexed and iteration elements remain borrowed; disjoint field loans and partial environment captures remain separately scoped.

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="core-02"></a>

## 2. Core Ownership Model

Research and formally define an ownership model that preserves memory safety without requiring normal developers to annotate function parameters with borrowing syntax.
Initial proposed semantics:
Owned values
Ordinary complex values have a single logical owner.
const user = getUser();
user owns the value.
Assignment
For nontrivially-copyable owned values:
const second = user;
ownership moves to second.
Using user afterward should produce a compile-time error unless the compiler can prove the operation was only a temporary borrow.
Copying
Provide explicit built-in operations.
Potential model:
copy(value)
clone(value)
Suggested distinction:
copy(value)
* Only allowed for trivially copyable values.
* Must not perform hidden allocation.
* Intended for primitive/small value types.
clone(value)
* Creates an independent owned duplicate.
* May allocate.
* Requires a Clone capability/trait.
Do not silently clone expensive values.
Research whether these names are ideal.
Function arguments
Function calls should not inherently move values.
The compiler should inspect usage and infer whether a parameter can be:
* borrowed immutably
* borrowed mutably
* moved
* copied
Example:
function logUser(user: User) {
    console.log(user.name);
}

const user = getUser();

logUser(user);
saveUser(user);
The compiler should be capable of lowering logUser(user) to an immutable borrow internally.
The source-language function syntax should remain simple.
Mutation
The compiler should infer mutable access when a function mutates an argument.
function normalize(user: User) {
    user.name = user.name.trim();
}

normalize(user);
save(user);
Potential internal lowering:
normalize(&mut user);
save(user);
No explicit &mut should normally appear in source code.

<a id="core-03"></a>

## 3. Shared Ownership

Treat shared ownership as part of the type system rather than merely as a keyword.
Initial candidate types:
Shared<T>
LocalShared<T>
Weak<T>
Possible semantics:
LocalShared<T> -> single-thread reference-counted ownership
Shared<T>      -> thread-safe reference-counted ownership
Weak<T>        -> non-owning reference into shared ownership
Potential Rust implementation:
LocalShared<T> -> Rc<T>
Shared<T>      -> Arc<T>
Weak<T>        -> Weak<T>
This mapping is implementation detail and should not appear in ordinary language documentation unless discussing compiler internals.
Research:
* Whether Shared<T> should imply thread safety.
* Whether separate LocalShared<T> and Shared<T> types are worthwhile.
* Mutation rules for shared values.
* Whether shared mutation should use explicit wrappers such as:
SharedCell<T>
SharedMap<K, V>
Atomic<T>
* Whether interior mutability should be available at all in general-purpose types.
* How to prevent reference cycles.
* Whether cycle detection is necessary.
* Whether Weak<T> is sufficient.
* Whether some shared structures should use arenas rather than reference counting.

<a id="core-11"></a>

## 11. Memory Allocation Strategy

A major goal is avoiding garbage collection without making application developers perform manual memory management.
Research:
* stack allocation
* escape analysis
* region/arena allocation
* heap allocation
* deterministic destruction
* reference counting
* temporary allocation
* bump allocators for request scopes
* allocator customization
* memory pools
* zero-copy APIs
Investigate whether HTTP request processing can benefit from request-scoped arenas.
Example:
incoming request
    ↓
request arena
    ↓
temporary parsing / validation / routing values
    ↓
response
    ↓
entire arena released
This could provide excellent backend performance while reducing ownership complexity.
Do not assume Rust’s default allocation patterns are automatically ideal for the language.

<a id="core-42"></a>

## 42. Closures

Closures are one of the hardest ownership areas.
Research:
* captured values
* mutable captures
* escaping closures
* shared captures
* async closures
* recursive closures
The compiler should infer capture strategy wherever possible.
Only require explicit developer intervention when semantics are genuinely ambiguous.

<a id="core-43"></a>

## 43. Cyclic Data Structures

Explicitly investigate graphs, trees with parent references, ORM entity graphs, caches, and observer patterns.
Examples:
parent.children.push(child);
child.parent = parent;
Determine supported patterns without tracing GC.
Possible techniques:
* weak references
* arenas
* handles/IDs
* specialized graph containers
* explicit Shared/Weak
* region lifetimes
Do not hand-wave this problem.

<a id="core-44"></a>

## 44. ORM and Object Graphs

Many backend frameworks rely heavily on mutable cyclic object graphs.
Research whether the ecosystem should encourage:
* immutable records
* IDs rather than object references
* data-oriented design
* explicit graph containers
Determine what ergonomic cost this creates for developers coming from Node.

<a id="core-45"></a>

## 45. Global State

Define safe global initialization.
Potential syntax:
const CONFIG = lazy(() => loadConfig());
Research:
* thread-safe initialization
* mutable globals
* test isolation
* dependency injection
* application state

### Moving arrays into tagged element types

An owned array can move into an array whose union element type includes every
source alternative. The conversion preserves order and moves each element once;
it does not clone resources or convert integers to floating point. Nullable
arrays retain their `null`/`undefined` alternatives when the destination includes
the same alternatives.

```tsr
const labels = ['Save', 'Cancel'];
const children: (string | boolean)[] = labels;
// labels has moved; children owns both strings.
```

This conversion requires ownership. Passing `string[]` as a borrowed or mutable
`(string | boolean)[]` is rejected. Move it into the wider owned array first, then
borrow that array. Narrowing element alternatives, structural resource casts and
`int[]` to `(number | string)[]` are not implicit conversions.
