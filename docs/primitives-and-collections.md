# Primitives and collections

[Documentation index](./README.md) · [Proposed roadmap](./roadmap.md) · [Source section map](./section-map.md)

## Workstream scope

**Status:** Unresolved design/research. The grouping and phase below are proposed planning aids, not approved language semantics.

**Proposed phase:** Core proof.

**Scope:** Specify numeric, string, array, and iterator behavior.

**Planning deliverable:** A semantics table covering representation, mutation, allocation, evaluation order, and bounds behavior.

**Dependencies to coordinate:** [type-system](./type-system.md), [ownership-and-memory](./ownership-and-memory.md)

## Current experiment

The [integer amendment](./milestones/01-integer-subset.md) implements a provisional signed 64-bit `int` with checked decimal literals and copy semantics. The original numeric `User` example runs. [Milestone 02](./milestones/02-control-flow.md) adds copied `boolean` values and strict boolean conditions. [Milestone 12](./milestones/12-scalar-operators.md) adds checked integer arithmetic, comparisons, strict scalar equality, and short-circuit logic. Arithmetic failures remain fatal under every build optimization level; integer division truncates toward zero. Milestone 39 subsequently adds standard Rust f64 numbers and explicit primitive conversion. Recoverable arithmetic errors and broader collection contracts remain open; the original proposals below are preserved.

[Milestone 20](./milestones/20-scalar-arrays.md) adds owned `int[]` and `boolean[]` values backed by native vectors. Literals, checked indexing and updates, read-only length, push returning void, and explicit cloning are implemented. Elements copy; whole arrays move. Array parameters, results, and captures follow the existing ownership rules. Negative and out-of-range indices fail with source byte ranges in every optimization profile. Owned elements, nested arrays, slices, removal, consuming iteration, and iterator pipelines remain separately scoped.

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

[Milestone 32](./milestones/32-array-search.md) adds read-only `includes` and `indexOf` for integer, boolean, and string arrays, including optional signed offsets and first-match behavior. Exact primitive content equality, temporary ownership, source-root argument loans, source failure order, and existing cloning/capture contracts remain enforced. Eight matched comparisons cover all six type/method combinations and two runtime-created Unicode string cases. Their evidence and remaining performance gaps are recorded in the gate and foundation audit.

[Milestone 33](./milestones/33-native-utf8-strings.md) uses standard Rust UTF-8 strings and methods. Length counts bytes; slice uses checked byte coordinates; charAt uses scalar iteration. Rust whitespace and lowercase tables apply. Source transfers, captures, and mutable collection clone independence remain enforced. Bracket indexing and character-code result types remain separately scoped.

[Milestone 34](./milestones/34-string-comparisons.md) adds exact string equality and standard Rust UTF-8 lexical ordering. Operands borrow without implicit clones and evaluate once in source order. A left operand's source loan remains active during right operand evaluation. Complete compiler regression and matched benchmark acceptance pass.

[Milestone 35](./milestones/35-string-uppercase.md) adds `toUpperCase()` using standard Rust Unicode conversion. The receiver is read and the result owns a normal UTF-8 string. Compiler, matched benchmark, and disposal acceptance pass.

[Milestone 36](./milestones/36-array-reversal.md) adds in-place `reverse()` using standard Rust slices and vectors, with a void result, and read-only length on temporary arrays. Compiler and three matched reversal comparisons pass.

[Milestone 37](./milestones/37-string-search.md) adds `startsWith`, `endsWith`, `includes`, and `indexOf` using standard Rust substring methods and byte positions. All 495 compiler groups, eight matched comparisons, and disposal profiles pass.

[Milestones 38–46](./roadmap.md) add classic for/do loops and block scope, native f64 numbers and primitive conversion, Number predicates, standard Math transforms, owning array slice, temporary indexed transfers, typed eager map/filter, and short-circuit some/every/findIndex/findLastIndex. Compiler and matched runtime acceptance are complete for these bounded gates. Map/filter support zero through three typed callback arguments; readonly source loans and native independent clone semantics remain explicit.

[Milestone 45](./milestones/45-fixed-number-arrays.md) extends standard `[T; N]` storage to proved number literals and directly indexed int/number/boolean literals. The shared 4 KiB proof budget and conservative escape/resize/capture fallbacks remain. Growing number arrays and escaping constructor returns retain ordinary vectors. All 531 compiler groups pass; 24 targeted comparisons and disposal profiles complete acceptance.

[Milestone 47](./milestones/47-string-construction.md) adds repeat, string-pattern replacement with template substitutions, and owned splitting through standard Rust methods. All 539 compiler groups and required checks pass. Twelve matched comparisons preserve complete outputs and operand effects; all 36 native allocation profiles fully dispose storage. Empty-pattern and empty-separator comparisons use shared BMP inputs, while native Unicode scalar semantics remain explicit.

[Milestone 48](./milestones/48-math-rounding-and-integer-functions.md) adds seven Math methods and sixteen constants through standard Rust floating-point and integer operations with small semantic guards. All 544 compiler groups and required checks pass. Seven matched comparisons and 21 zero-allocation profiles complete acceptance; recorded min/max/imul performance gaps remain visible.

## Primitive performance follow-up

The active [foundation audit](./foundation-audit.md) records array layout proofs and standard Rust string storage. Integer and boolean values remain inline. Stable scalar array literals use ordinary `[T; N]` after a complete-use proof; growable arrays use bounded inline storage followed by vectors. Recursive records retain bounded layout validation and iterative clone/cleanup. Standard `Cow<'static, str>` borrows static literals and owns dynamic `String` buffers; owned string cloning copies those buffers. Retired literal normalization caches and interning domains are archived outside the crates.

Read-only literal string arrays can use private static `str` slices after complete-use and budget proofs. Indexed reads and iteration expose standard borrowed string values, and search compares exact contents through standard Rust methods. Explicit clones restore independently mutable array storage. Moves, mutation, captures, dynamic elements, and unproved uses keep ordinary owned storage. This is an internal optimization, not a source view or shared mutable array contract.

Indexing, replacement, length, push, ordered removal, cloning, and borrowed iteration retain source checks and evaluation order. Clones preserve independent mutable storage. Larger inline values trade stack/environment size for fewer heap allocations; allocator probes do not measure that footprint. New array methods and mutable/consuming iteration remain separate implementation gates.

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="core-05"></a>

## 5. Primitive Types

Do not automatically inherit JavaScript’s primitive model.
Research and decide:
Numbers
Should:
number
mean:
* f64, JavaScript-style
* inferred integer/float
* compiler-selected numeric representation
* an alias for a default integer or floating type
Potential explicit types:
i8
i16
i32
i64
u8
u16
u32
u64
f32
f64
int
uint
number
Determine whether this is too much complexity for the target audience.
Investigate:
* overflow semantics
* integer division
* checked arithmetic
* decimal/money support
* bigint support
* literal inference
Strings
Research:
* UTF-8 behavior
* indexing semantics
* substring slicing
* grapheme versus byte versus Unicode scalar indexing
* immutable vs mutable string types
* whether string methods should mimic JavaScript even when JavaScript behavior is inefficient

<a id="core-08"></a>

## 8. Collection Model

Preserve familiar JavaScript methods and chaining where practical:
users
    .filter(user => user.active)
    .map(user => user.email)
    .take(100);
Do not necessarily preserve JavaScript execution mechanics.
The compiler should be free to implement this as:
* lazy iterators
* fused loops
* stack allocation
* optimized native loops
* SIMD where possible
Research whether:
array.map()
returns a realized collection, as in JavaScript, or whether chains are lazy by default.
Potential compromise:
array.map(...)
returns an array.
array.iter()
    .filter(...)
    .map(...)
is lazy.
However, additional syntax may reduce JavaScript familiarity.
Explore whether the compiler can automatically fuse eager-looking pipelines without observable semantic differences.
Core APIs to evaluate:
map
filter
reduce
find
findIndex
some
every
includes
slice
splice
sort
reverse
flat
flatMap
forEach
join
push
pop
shift
unshift
For every familiar JavaScript API, explicitly determine whether preserving exact semantics would harm performance, safety, or predictability.
