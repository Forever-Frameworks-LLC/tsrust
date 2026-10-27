# Type system and data shapes

[Documentation index](../../README.md) · [Proposed roadmap](../../roadmap.md) · [Source section map](../../plans/section-map.md)

## Workstream scope

**Status:** Unresolved design/research. The grouping and phase below are proposed planning aids, not approved language semantics.

**Proposed phase:** Core proof.

**Scope:** Specify static types, object shapes, capabilities, and generics.

**Planning deliverable:** A minimal type-system specification and examples of narrowing and rejected programs.

**Dependencies to coordinate:** [principles-and-boundaries](../../plans/principles-and-boundaries.md), [ownership-and-memory](ownership-and-memory.md)

## Implementation history and current scope

The numbered descriptions below record the scope of each early milestone. A limitation stated for one milestone is not a current-language limitation if later work supersedes it. Current accepted nullable data types and strict narrowing are documented in [Milestone 84](../../milestones/84-nullable-type-foundation.md) and [Milestone 85](../../milestones/85-nullable-flow-narrowing.md). Nullable callable transport is [Milestone 97](../../milestones/97-nullable-callable-transport.md), which is still in progress; optional/default parameter syntax is not accepted by that gate. Use [feature coverage](feature-coverage.md) and [the roadmap](../../roadmap.md) for the current implementation backlog.

The compiler supports nominal records with `string`, `int`, `boolean`, array, and named-record fields. [Milestone 02](../../milestones/02-control-flow.md) requires boolean conditions and checks all return paths. [Milestone 04](../../milestones/04-synchronous-callbacks.md) adds `() => Result` callback parameter annotations with exact result matching and reusable, non-escaping invocation. [Milestone 05](../../milestones/05-single-use-callbacks.md) adds `Once<Function<Result>>` for owned, single-use callback parameters. Concrete closures can satisfy the reusable or single-use contract according to their invocation behavior; borrowed reusable parameters and owned single-use parameters are not interchangeable. [Milestone 06](../../milestones/06-closure-arguments.md) extends both callback contracts and callable factory results to ordered integer/boolean parameter lists. Arity and types must match exactly; signature parameter names are not type identity. [Milestone 07](../../milestones/07-owned-closure-arguments.md) adds strings and nominal records with inferred arrow-body access and explicit `read`/`mutate`/`consume` in callable signatures. Signature compatibility requires exact access as well as exact type and arity; no access weakening or implicit adapter is implemented. [Milestone 08](../../milestones/08-nested-local-closures.md) permits nested local creation and invocation with the same argument contracts. [Milestone 09](../../milestones/09-callable-captures.md) adds concrete callable captures, rejecting borrowed environment escapes. [Milestone 10](../../milestones/10-callable-results.md) adds recursive callable result annotations and owned returns from arrow bodies. [Milestone 11](../../milestones/11-retained-callbacks.md) adds `owned` and `owned once` callback parameters retained through owned wrappers, with exact contracts and bounded cached specialization. Callable-valued arrow arguments, callback signature results, shared aliases and non-owned callback returns, borrowed/mixed-origin returns, cross-module stability, access variance, shared ownership, and async remain open. At milestone 11, source generics, union types, optional values, explicit truthiness conversion and type narrowing were not implemented. Later milestones supersede parts of this snapshot; the original proposals below are not evidence of accepted behavior.

[Milestone 12](../../milestones/12-scalar-operators.md) adds integer arithmetic and comparisons, strict equality between matching scalar types, and boolean-only logic. Operators perform no coercion. Strings, records, and callable values do not have equality operators in this gate.

[Milestone 13](../../milestones/13-while-loops.md) adds boolean-only while conditions and scoped break/continue targets. A loop may execute zero times for return-coverage purposes, including literal-true loops. Loop bodies are lexical scopes, and nested closure bodies cannot target a surrounding loop.

[Milestone 14](../../milestones/14-mutable-bindings.md) adds initialized `let` declarations for scalar, string, and nominal record types. Reassignment preserves the original type and nominal identity. `const` and parameter bindings remain non-rebindable; uninitialized declarations remain unsupported. [Milestone 16](../../milestones/16-mutable-callables.md) adds callable `let` bindings whose replacements require the same concrete environment origin, not merely matching signatures.

[Milestone 17](../../milestones/17-callable-moves.md) permits declaration moves of concrete callable environments. Invocation and argument contracts remain unchanged; the source becomes unavailable. Return chains preserve the original escape origin, and borrowed environments retain their lexical capture loans. Capturing wrappers that move an environment are single-use. Shared aliases remain unsupported. [Milestone 18](../../milestones/18-callback-moves.md) adds callback-parameter declaration moves: reusable contracts transfer local borrows, owned contracts retain capture permission, and single-use contracts survive specialization. Mutable callback locals can restore values derived from the same parameter. [Milestone 19](../../milestones/19-callback-returns.md) adds direct owned-callback returns with exact signature checks and parameter-origin substitution through templates. Reusable environments return unchanged; cached concrete wrappers preserve declared single-use invocation when needed without allocating. Non-owned callback returns and captures remain unsupported.

[Milestone 20](../../milestones/20-scalar-arrays.md) adds exact homogeneous `int[]` and `boolean[]` types. Arrays are owned values; elements copy. Moves, explicit clone independence, parameter/capture access, lexical loans, branch joins, and loop restoration apply to whole arrays. Public callable argument signatures require explicit array access contracts. Indexing is bounds checked; writes and push mutate named arrays, including const-bound arrays. Owned elements, nested arrays, and array record fields remain outside this gate.

[Milestone 21](../../milestones/21-for-of.md) adds borrowed for-of iteration with fresh const/let scalar bindings. Iterable expressions evaluate once and temporary arrays live through the loop. Named sources retain ownership and cannot be mutated, replaced, or transferred while iteration holds a shared loan. Break/continue, lexical captures, and ownership joins follow the existing loop rules; consuming iteration contracts remain open.

[Milestone 22](../../milestones/22-owned-array-elements.md) adds owned string and named-record array elements. Construction, push, and replacement move owned values; indexing borrows, and explicit cloning produces independent ownership. Indexed record fields support reads and writes. Element argument loans conservatively protect the whole array. For-of borrows owned elements for reading; moves, writes, rebinding, and escaping retention receive source diagnostics. Temporary arrays, captures, callback contracts, native bounds checks, and deterministic cleanup are verified.

[Milestone 23](../../milestones/23-nested-arrays.md) adds exact nested arrays such as `int[][]` and `User[][]`. Rows move into owned vectors; indexing borrows, scalar leaves copy, and explicit cloning creates independent nested storage. Indexed rows support length, push, replacement, record-field access, and mutable arguments. Nested iteration keeps row borrows lexical. Parent selections stay stable during later operand evaluation through conservative source loans. Bounds failure timing, callbacks, captures, ownership restoration, 100-level native access, and the 128-dimension type boundary are verified.

[Milestone 24](../../milestones/24-array-record-fields.md) adds owned array fields with exact leaf/rank types, including forward and recursive record arrays through vector indirection. Mixed field/index paths support borrowing, mutation, arguments, and iteration. Loans protect the root owner during selected-parent operand evaluation and shared loops. Construction and replacement move arrays; explicit cloning creates independent field storage. Milestones 25 and 31 subsequently add direct record fields and direct named field extraction.

[Milestone 25](../../milestones/25-nested-record-fields.md) adds direct named-record fields with inline native representation and exact nominal identity. Nested literals, updates, cloning, borrowed arguments, captures, callbacks, and iteration preserve root-owner contracts. Iterative cached layout validation rejects direct cycles and bounds inline depth/storage before Rust emission; array indirection permits recursive owned trees. Native deep access and source failure timing are verified. Milestone 31 subsequently adds direct named field extraction and restoration.

[Milestone 26](../../milestones/26-recursive-functions.md) adds self and mutual named-function recursion for data parameters/results. Monotone access summaries stabilize before a fresh definitive ownership analysis. Transfers, loans, captures, callbacks outside recursive signatures, iteration, runtime failure ranges, and source guards remain enforced. Milestone 27 subsequently adds ordinary reusable/single-use parameters. Milestone 28 subsequently extends owned parameters too. Milestone 29 subsequently resolves single-origin callable results; termination and tail-call behavior remain outside the guarantee.

[Milestone 27](../../milestones/27-recursive-callbacks.md) extends self and mutual recursion to reusable and ordinary single-use callback parameters with non-callable results. Exact argument/result/access contracts, declaration aliases, single-use transfers, capture loans, shared iteration, and stable data ownership summaries remain enforced. Fresh closed arrows/factory results and callback permutations are supported. Definitive typed-call analysis bounds each recursive callback function to 128 distinct concrete environment type vectors; repeated values and erased lifetimes count once. Non-owned callback parameters still cannot be captured. Milestone 28 subsequently extends owned callback parameters; milestone 29 subsequently adds single-origin callable-result cycles.

[Milestone 28](../../milestones/28-recursive-owned-callbacks.md) extends named recursion to owned reusable/single-use callback parameters with non-callable results. Abstract templates preserve exact transfer, retention, invocation, and argument contracts. Concrete data-result bodies drain iteratively through the existing global 128-specialization cache; mixed ordinary parameters retain the per-function vector guard. Source arrows with matching capture layouts and callable contracts reuse native types while each construction applies fresh values and loans. Growing environment types receive source diagnostics. Milestone 29 subsequently adds single-origin callable-result cycles.

[Milestone 29](../../milestones/29-recursive-callable-results.md) resolves self/mutual callable-result cycles to one concrete environment origin before definitive checking. Source branch/declaration order does not select native evaluation order. Declared callable shapes support provisional local invocation, including lazy nested factories, but cannot supply concrete returns or captures. Final bodies enforce transfers, invocation, exact signatures, all callback contracts, and source loans; discovery-only environments are removed before public metadata or emission. Mixed/missing origins and recursive inline environment storage remain unsupported.

[Milestone 30](../../milestones/30-array-removal.md) adds ordered `array.remove(index)` with an exact element result. Scalar, string, named-record, and nested-array values transfer out without cloning; the remaining array retains ownership and order. Removal infers root-owner mutation and preserves argument, capture, iteration, callback, recursive, and selection loans. Named receivers permit index effects before final mutation; selected field/index parents stay stable. Negative/out-of-range indices fail with the full removal call's source byte range under every optimization profile. Milestone 31 subsequently adds direct field moves. Optional/range removal and mutable/consuming iteration remain separately scoped.

[Milestone 31](../../milestones/31-record-field-moves.md) adds moves of direct named string/record/array fields with per-path availability. Siblings remain usable; missing fields block overlapping or whole-record access with related transfer locations. Exact field/parent replacement and whole let rebinding restore availability. Branch joins and loop edges retain missing-field state. Field consumption infers owning root parameter/capture contracts; detached argument values release source loans, while borrowed field arguments and existing capture/iteration/selection loans remain conservative. Native places/drop flags preserve initialization, evaluation, and bounds behavior without runtime field-state storage. Indexed and iteration elements remain borrowed; disjoint field loans and partial environment captures remain separately scoped.

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="core-04"></a>

## 4. Type System

The type system should be stricter than TypeScript.
Do not preserve TypeScript behaviors solely for compatibility if they weaken static guarantees.
Likely remove or prohibit:
any
Research treatment of:
unknown
never
object
{}
undefined
null
Potential goals:
* No implicit any.
* No unchecked type assertions.
* No arbitrary casts.
* No prototype mutation.
* No runtime property injection on statically shaped objects.
* No implicit coercive equality.
* No widespread truthiness-based type confusion.
* No unchecked indexed access.
* Exhaustive union handling.
* Strong nullability guarantees.
* Sound generic constraints.
Preserve TypeScript-style control-flow narrowing where possible.
Example:
function getUser(): User | null {
    ...
}

const user = getUser();

if (user) {
    console.log(user.name);
}
The compiler should narrow user to User inside the branch.

<a id="core-06"></a>

## 6. Objects, Structs, and Data Shapes

Object syntax should feel like TypeScript:
type User = {
    id: number;
    name: string;
};

const user: User = {
    id: 1,
    name: "Pat",
};
But object shapes should normally be statically fixed.
This should likely fail:
user.randomField = 123;
Use maps/dictionaries for dynamic key-value structures.
Research distinction between:
* record/struct types
* classes
* maps
* anonymous object types
* tuples
* tagged unions
* enums
Determine whether JavaScript-style classes are desirable or whether classes should be syntactic sugar over data + traits + methods.

<a id="core-07"></a>

## 7. Traits / Interfaces / Capabilities

Create a capability system inspired by Rust traits but expressed through familiar TypeScript syntax.
Potential syntax:
trait Display {
    toString(): string;
}

class User implements Display {
    name: string;

    toString(): string {
        return this.name;
    }
}
Generic constraints might resemble:
function print<T extends Display>(value: T) {
    console.log(value.toString());
}
Static dispatch should be preferred where possible.
Research a separate dynamic-dispatch type such as:
Trait<Display>
Potential semantics:
function print(value: Trait<Display>) {
    console.log(value.toString());
}
which may internally use a trait object.
Research whether this syntax is intuitive enough.
Potential compiler-known traits/capabilities:
Copy
Clone
Equal
Hash
Display
Debug
Serialize
Deserialize
Send
Sync
Iterator
IntoIterator
Error
Default
Determine which should be language primitives versus ordinary standard-library traits.

<a id="core-41"></a>

## 41. Generics Strategy

Research:
* monomorphization
* code bloat
* dynamic dispatch
* generic constraints
* variance
* higher-kinded types
* associated types
* generic specialization
Do not automatically add advanced type-system features unless they solve concrete application problems.

## Generic interface inheritance

An interface can extend a generic record or interface with explicit arguments:

```typescript
interface Box<T> { value: T; }
interface NamedBox<U> extends Box<U> { name: string; }
interface StringBox extends NamedBox<string> { length: int; }
```

Inherited fields use the supplied types, including nested arrays, optional fields
and callable annotations. Bases may be declared later or imported under an alias;
private field types retain their defining module identity. Multiple bases are
expanded in written order before local fields. Duplicate fields and inheritance
cycles reject, including cycles through generic bases. Every base requires exactly
its declared number of type arguments.

Inheritance composes fields; it does not introduce implicit base conversions,
virtual methods, shared ownership or clones. Derived values remain nominal.
Interface method declarations and generic parameter defaults remain unsupported.
