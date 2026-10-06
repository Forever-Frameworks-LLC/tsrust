# Open decisions and cross-cutting gaps

[Documentation index](./README.md) · [Proposed roadmap](./roadmap.md)

These are review observations to guide scoping. They supplement the original questions in the [research program](./research-program.md) and [desktop prototype workstream](./desktop-prototypes-and-shared-code.md); they do not decide the language semantics.

## Priority order

The [opening priority list](./product-vision.md#core-00) puts TypeScript ergonomics first. The [decision rule](./principles-and-boundaries.md#core-62) puts memory safety first. [Benchmark guidance](./benchmarks.md#web-32) supplies another order.

**Decision needed:** Distinguish hard requirements from preferences and establish one authoritative rule for tradeoffs.

**Blocks:** Final language-design decisions. Small experiments can retain explicitly provisional assumptions.

## Function access contracts

[Argument inference](./ownership-and-memory.md#core-02) depends on how a function uses its parameters. The document does not yet define the public contract when implementations, modules, callbacks, generics, or dynamic dispatch hide that usage.

**Decision needed:** Define how borrowed, mutable, and consuming behavior is represented in exported signatures and callable types. Decide which implementation changes count as API changes. Include aliasing, returned borrows, and recursion in the examples.

**Blocks:** Reliable cross-module ownership inference and general callback support.

## Closure captures and public callback contracts

[Milestone 03](./milestones/03-closures.md) separates capture storage from invocation access. Its provisional subset uses block-scoped local loans and owned returned environments. Compiler acceptance tests now verify capture inference and owned escape behavior alongside independent Rust witnesses. [Milestone 04](./milestones/04-synchronous-callbacks.md) defines a bounded callback annotation: zero arguments, exact result type, synchronous non-escaping reuse, and exclusive access when invoked. Inline capture loans end at call completion; named captures retain block lifetime. [Milestone 05](./milestones/05-single-use-callbacks.md) adds an explicit `once` parameter contract: ownership transfers on passing, and branch analysis permits at most one invocation or forwarding per path. Both callback contracts remain synchronous and non-escaping. [Milestone 06](./milestones/06-closure-arguments.md) adds copied integer/boolean arguments with exact ordered signature matching. [Milestone 07](./milestones/07-owned-closure-arguments.md) supplies provisional explicit read/mutate/consume contracts for owned arguments in callable signatures, matched exactly against inferred arrow-body access. [Milestone 08](./milestones/08-nested-local-closures.md) extends these rules to nested local creation and invocation, with transitive capture effects and lexical loans. [Milestone 09](./milestones/09-callable-captures.md) adds concrete callable captures, rejecting borrowed environment escapes. [Milestone 10](./milestones/10-callable-results.md) adds recursive callable result annotations and owned returns from arrow bodies. [Milestone 11](./milestones/11-retained-callbacks.md) adds `owned` and `owned once` callback parameters retained through owned wrappers, with exact contracts and bounded cached specialization. Callable-valued arrow arguments, callback signature results, shared aliases and non-owned callback returns, borrowed/mixed-origin returns, cross-module stability, access variance, shared ownership, and async remain open.

[Milestone 13](./milestones/13-while-loops.md) completes bounded while-loop ownership for immutable bindings, with explicit continuing-edge checks and distinct break/return joins. [Milestone 14](./milestones/14-mutable-bindings.md) adds initialized local `let` bindings and exact-type ownership reinitialization. Every repeating path must preserve entry availability; capture loans still block replacement. [Milestone 15](./milestones/15-capture-replacement.md) adds writable inherited `let` captures with explicit local loans and private escaped state. [Milestone 16](./milestones/16-mutable-callables.md) adds same-origin mutable callable bindings and explicit move replacement. Mixed callable environment types, callable fields and non-owned callback returns, shared capture cells, reusable consuming replacement, general initialization inference, collection iteration, and termination analysis require separate contracts.

[Milestone 17](./milestones/17-callable-moves.md) permits declaration moves of concrete callable environments. Invocation and argument contracts remain unchanged; the source becomes unavailable. Return chains preserve the original escape origin, and borrowed environments retain their lexical capture loans. Capturing wrappers that move an environment are single-use. Shared aliases remain unsupported. [Milestone 18](./milestones/18-callback-moves.md) adds callback-parameter declaration moves: reusable contracts transfer local borrows, owned contracts retain capture permission, and single-use contracts survive specialization. Mutable callback locals can restore values derived from the same parameter. [Milestone 19](./milestones/19-callback-returns.md) adds direct owned-callback returns with exact signature checks and parameter-origin substitution through templates. Reusable environments return unchanged; cached concrete wrappers preserve declared single-use invocation when needed without allocating. Non-owned callback returns and captures remain unsupported.

**Decision needed:** Evolve block-scoped loans toward last-use analysis, define callable contracts across module/API boundaries, extend owned retention to library/UI/async APIs, and choose whether borrowed closure results or mixed environment types are supported. Explicitly account for UI and async callback escape routes.

**Blocks:** General callback APIs and borrowed/shared escaping closures. The local, owned-return, and synchronous reusable/single-use callback and scalar/owned argument and nested local closure, callable capture/result, and owned retained callback gates are complete under the documented provisional rules.

[Milestone 20](./milestones/20-scalar-arrays.md) establishes a realized native-vector collection for copied integer/boolean elements. Ownership and access are explicit through existing inference/contracts; no implicit array cloning occurs. Bounds failures are fatal with source ranges. Slices, recoverable bounds errors, removal, consuming iteration, and lazy/eager pipeline contracts remain open. [Milestone 21](./milestones/21-for-of.md) adds borrowed scalar for-of iteration with explicit active loans and one-time iterable evaluation.

[Milestone 22](./milestones/22-owned-array-elements.md) adds owned string and named-record array elements. Construction, push, and replacement move owned values; indexing borrows, and explicit cloning produces independent ownership. Indexed record fields support reads and writes. Element argument loans conservatively protect the whole array. For-of borrows owned elements for reading; moves, writes, rebinding, and escaping retention receive source diagnostics. Temporary arrays, captures, callback contracts, native bounds checks, and deterministic cleanup are verified.

[Milestone 23](./milestones/23-nested-arrays.md) adds exact nested arrays such as `int[][]` and `User[][]`. Rows move into owned vectors; indexing borrows, scalar leaves copy, and explicit cloning creates independent nested storage. Indexed rows support length, push, replacement, record-field access, and mutable arguments. Nested iteration keeps row borrows lexical. Parent selections stay stable during later operand evaluation through conservative source loans. Bounds failure timing, callbacks, captures, ownership restoration, 100-level native access, and the 128-dimension type boundary are verified.

[Milestone 24](./milestones/24-array-record-fields.md) adds owned array fields with exact leaf/rank types, including forward and recursive record arrays through vector indirection. Mixed field/index paths support borrowing, mutation, arguments, and iteration. Loans protect the root owner during selected-parent operand evaluation and shared loops. Construction and replacement move arrays; explicit cloning creates independent field storage. Milestones 25 and 31 subsequently add direct record fields and direct named field extraction.

[Milestone 25](./milestones/25-nested-record-fields.md) adds direct named-record fields with inline native representation and exact nominal identity. Nested literals, updates, cloning, borrowed arguments, captures, callbacks, and iteration preserve root-owner contracts. Iterative cached layout validation rejects direct cycles and bounds inline depth/storage before Rust emission; array indirection permits recursive owned trees. Native deep access and source failure timing are verified. Milestone 31 subsequently adds direct named field extraction and restoration.

[Milestone 26](./milestones/26-recursive-functions.md) adds self and mutual named-function recursion for data parameters/results. Monotone access summaries stabilize before a fresh definitive ownership analysis. Transfers, loans, captures, callbacks outside recursive signatures, iteration, runtime failure ranges, and source guards remain enforced. Milestone 27 subsequently adds ordinary reusable/single-use parameters. Milestone 28 subsequently extends owned parameters too. Milestone 29 subsequently resolves single-origin callable results; termination and tail-call behavior remain outside the guarantee.

[Milestone 27](./milestones/27-recursive-callbacks.md) preserves ordinary callback contracts through recursion and bounds native callback type vectors. [Milestone 28](./milestones/28-recursive-owned-callbacks.md) extends owned contracts with iterative concrete data-result checking and canonical source arrow types. Matching layouts reuse types without sharing values or suppressing creation effects. Growing owned environments and excess specialization receive frontend diagnostics; milestone 29 subsequently adds single-origin callable-result cycles.

[Milestone 29](./milestones/29-recursive-callable-results.md) resolves self/mutual callable-result cycles to one concrete environment origin before definitive checking. Source branch/declaration order does not select native evaluation order. Declared callable shapes support provisional local invocation, including lazy nested factories, but cannot supply concrete returns or captures. Final bodies enforce transfers, invocation, exact signatures, all callback contracts, and source loans; discovery-only environments are removed before public metadata or emission. Mixed/missing origins and recursive inline environment storage remain unsupported.

[Milestone 30](./milestones/30-array-removal.md) adds ordered `array.remove(index)` with an exact element result. Scalar, string, named-record, and nested-array values transfer out without cloning; the remaining array retains ownership and order. Removal infers root-owner mutation and preserves argument, capture, iteration, callback, recursive, and selection loans. Named receivers permit index effects before final mutation; selected field/index parents stay stable. Negative/out-of-range indices fail with the full removal call's source byte range under every optimization profile. Milestone 31 subsequently adds direct field moves. Optional/range removal and mutable/consuming iteration remain separately scoped.

[Milestone 31](./milestones/31-record-field-moves.md) adds moves of direct named string/record/array fields with per-path availability. Siblings remain usable; missing fields block overlapping or whole-record access with related transfer locations. Exact field/parent replacement and whole let rebinding restore availability. Branch joins and loop edges retain missing-field state. Field consumption infers owning root parameter/capture contracts; detached argument values release source loans, while borrowed field arguments and existing capture/iteration/selection loans remain conservative. Native places/drop flags preserve initialization, evaluation, and bounds behavior without runtime field-state storage. Indexed and iteration elements remain borrowed; disjoint field loans and partial environment captures remain separately scoped.

## String representation and observable indices

The user superseded the earlier JavaScript string decision on October 1, 2026 and selected the [native Rust UTF-8 contract](./string-semantics.md). Length counts bytes, slice checks byte boundaries, charAt iterates scalars, and Rust trim/lowercase apply. JavaScript surrogate compatibility is not required.

**Blocks:** New string indexing/slicing contracts and claims of complete JavaScript string compatibility. Existing array search and current prototype transform behavior remain defined.

## Assignment, sharing, and cloning

The original [ownership rules](./ownership-and-memory.md#core-02) make complex assignments transfer ownership, but leave a possible temporary-borrow exception. The [diagnostic example](./compiler-and-modules.md#core-22) suggests `Shared(user)` as a repair without showing how both later accesses work.

**Decision needed:** Define whether assignment behavior is fixed or inferred, whether `Shared(user)` consumes `user`, and how another shared handle is obtained. Specify whether cloning shared handles duplicates state or shares it; distinguish that from the stated independent-copy goal of `clone()`.

**Blocks:** Consistent move diagnostics, shared-state APIs, and source-level cost explanations.

## Signal assignment and mutation

The [counter](./desktop-architecture.md#desktop-01) declares `const count` and then increments it. The [async UI example](./desktop-async-and-platform.md#desktop-18) assigns to a `const` signal binding. Those examples require special signal semantics or different surface syntax.

**Decision needed:** Distinguish rebinding, ordinary field mutation, and signal updates. Decide whether signals are compiler primitives or ordinary values. State how captures and reactive dependencies behave when values escape.

**Blocks:** Reactive lowering, two-way binding, and predictable callback semantics.

## Async lifetime and cleanup guarantees

[Async design](./async-and-runtime.md#core-09) and [component-scoped tasks](./desktop-async-and-platform.md#desktop-19) leave cancellation and asynchronous destruction open.

**Decision needed:** Specify task ownership, cancellation completion, resource cleanup, and access after component/request teardown. Separate cancellation requests from guarantees that work has stopped.

**Blocks:** Safe async captures, deterministic teardown claims, and component-owned background work.

## Numeric behavior

The [integer amendment](./milestones/01-integer-subset.md) provisionally chooses signed 64-bit `int`, decimal literal bounds, and scalar copying for the first prototype. [Milestone 12](./milestones/12-scalar-operators.md) adds checked integer arithmetic with fatal overflow/zero-divisor failures, truncation toward zero, integer comparisons, strict scalar equality, and short-circuit boolean operators. These provisional rules are independent of backend optimization and overflow settings.

**Decision needed:** Evaluate recoverable arithmetic errors, broader value equality, conversion rules, explicit widths, floating-point support, and the meaning of `number`. Keep these independent of build optimization level.

**Blocks:** Numeric operations and broader collection/serialization APIs, not the completed original `User` example.

## Compatibility and observable behavior

[Collection semantics](./primitives-and-collections.md#core-08), [Hono compatibility levels](./http-and-hono.md#web-15), and [source compatibility](./principles-and-boundaries.md#core-49) intentionally leave room for divergence.

**Decision needed:** Record preserved and changed behavior for each supported API, including callback order, eager/lazy execution, ownership effects, and error timing. Define what any reported compatibility metric measures.

**Blocks:** Credible migration claims and optimization correctness criteria.

## Desktop scope and update guarantees

Desktop is a [first-class research target](./desktop-architecture.md#desktop-00), while the initial language proposal is backend-first. The [fine-grained benchmark](./desktop-tooling-and-benchmarks.md#desktop-36) focuses on one affected text binding.

**Decision needed:** Separate early desktop research from implementation commitments. Choose a first platform experiment. Distinguish the number of reactive bindings evaluated from layout and painting caused by a changed native control.

**Blocks:** A bounded desktop implementation task and measurable performance acceptance criteria.

## Evidence and naming

[Prior naming findings](./identity.md#web-07) and [competitor descriptions](./competitive-landscape.md#web-08) are preserved from the proposal without fresh verification.

**Decision needed:** Confirm current primary sources, record dates and uncertainty, and complete the [required research deliverables](./research-program.md#web-34) before final naming.

**Blocks:** Final identity selection and public competitive claims. It does not block a provisionally named compiler experiment.
