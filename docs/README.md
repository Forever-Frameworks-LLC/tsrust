# Documentation index

The original vision is organized into 33 focused workstreams. Every original section is preserved once in the workstream documents, including examples and unresolved alternatives.

Start with [the proposed roadmap](./roadmap.md) to scope implementation, [open decisions](./open-decisions.md) to identify design gates, or [the complete section map](./section-map.md) to find an original section.

## Current work

[Owned native bytes](native-bytes.md) documents the initial std:bytes construction and borrowing API. [Gate 187](milestones/187-native-byte-foundation.md) accepts the typed native foundation and untimed matched workload; wider byte operations, views and resource lifecycle remain in progress.

[Milestone 100: call spread](./milestones/100-call-spread.md) is in progress after closed gate 99. It covers dense homogeneous arrays, real argument counts, ownership and current callable/builtin targets.

[Call spread](./call-spread.md) explains array expansion into call arguments, count checks, evaluation order, ownership and built-in calls. Examples are verified with the current compiler and native execution. Gate 100 focused acceptance is recorded in its [milestone](./milestones/100-call-spread.md); measurements are pending checkpoint 101.

[Rest parameters](./rest-parameters.md) explains fresh owning packs, fixed argument defaults, native ownership and array callback tuples. Examples are verified with the current compiler and native execution. Gate 99 focused acceptance and exact artifact archival are complete; full regression and measurements remain queued for checkpoint 101.

[VS Code editor foundation plan](./editor-foundation.md) defines the independent extension/LSP implementation, first-release acceptance, and explicit compiler handoffs for recovery, diagnostic aggregation, cancellation, semantic metadata and modules. E1–E4 now have a locally installed extension and [acceptance evidence](./editor-foundation-acceptance.md); C1–C5 remain deferred.

[Modules and native standard-library roadmap](./standard-library-roadmap.md) tracks concrete dependency and acceptance gates for modules, JSON/schema validation, Tokio, HTTP/TLS, databases, caches and queues. The [researched coverage inventory](./standard-library-coverage.md) maps Node API families, ecosystem references and application coverage to those gates.

[Feature and comparison coverage](./feature-coverage.md) provides the comprehensive implementation backlog, grouped by primitive type, built-ins, control flow, iterators, generators, decorators, and TypeScript constructs. Implemented behavior and benchmark coverage are tracked separately. Use [the development workflow](./development-workflow.md): focused behavioral/editor/documentation checks per gate, with full regressions and accumulated benchmarks every five gates. The current batch is gates 97–101. Preserve passed checks and rerun only affected checks after repairs.

[Milestone 35: uppercase](./milestones/35-string-uppercase.md) is complete with compiler and runtime evidence. [Milestone 36: array reversal and temporary length](./milestones/36-array-reversal.md) is complete with 491 compiler groups, three matched workloads, and disposal evidence. [Milestone 37: string search](./milestones/37-string-search.md) is complete with all 495 compiler groups, eight matched comparisons, and allocation/disposal acceptance. [Milestone 38: control flow](./milestones/38-control-flow-loops.md) is complete with all 502 compiler groups, twelve matched control/primitive comparisons, and allocation/disposal acceptance. Standalone blocks have compile-only scope and cleanup acceptance. [Standard Rust numbers](./milestones/39-native-number.md) pass all 509 compiler groups and required Rust checks, with seven focused groups at optimization levels 0 and 3. Sixteen Number comparison pairs and the normalized number resource probe also pass runtime acceptance, with immutable records and allocation/disposal evidence. [Milestone 40: Number classification](./milestones/40-number-predicates.md) is complete with 512 compiler groups, required Rust checks, four matched comparisons, and allocation/disposal evidence. [Milestone 41: Math transforms](./milestones/41-math-unary.md) is complete with 515 compiler groups, five matched comparisons, and disposal profiles. [Milestone 42: array slicing](./milestones/42-array-slice.md) is complete with 519 compiler groups, four matched comparisons, and rank-aware resource evidence. [Milestone 43: temporary indexing](./milestones/43-temporary-array-indexing.md) is complete with 523 compiler groups, four matched comparisons, and disposal evidence. [Milestone 44: map/filter](./milestones/44-array-map-filter.md) is complete with 527 compiler groups, eight matched comparisons, and disposal evidence. [Milestone 45: fixed scalar storage](./milestones/45-fixed-number-arrays.md) is complete with 531 compiler groups, corrected release emission checks, 24 targeted comparisons, and disposal evidence. [Milestone 46: array predicates](./milestones/46-array-predicates.md) is complete with 535 compiler groups, 24 matched workloads across eight runtimes, and disposal evidence. [Milestone 47: string construction](./milestones/47-string-construction.md) is complete with 539 compiler groups, twelve matched workloads across eight runtimes, and disposal evidence. [Milestone 48: exact Math](./milestones/48-math-rounding-and-integer-functions.md) is complete with 544 compiler groups, seven matched workloads across eight runtimes, sixteen constant checks, and zero-allocation profiles. [Milestone 49: approximate Math](./milestones/49-native-math-functions.md) is complete with one 549-group regression, independent numerical validation, 22 matched comparisons, and zero-allocation profiles.

[Milestone 50](./milestones/50-number-operators.md), [milestone 51](./milestones/51-array-reduction.md), [milestone 52](./milestones/52-array-for-each.md) and [milestone 53](./milestones/53-array-concat.md) are complete with compiler and matched runtime/resource evidence. Historical gate53 comparisons are grouped in [BENCHMARKS.md](../BENCHMARKS.md).

[Milestone 69: conditional expressions](./milestones/69-conditional-expressions.md) is complete with 633 compiler groups, 648 workspace groups, ten matched comparisons and thirty disposed profiles. The earlier report contained 444 separately dated comparisons. [Milestone 70: String.concat and padding](./milestones/70-string-concat-padding.md) is complete with 637 compiler groups, 652 workspace groups, editor discovery, ten matched comparisons and thirty disposed profiles. The report now has 454 separately dated rows; native leads Node on 5/10 new rows and the fastest scripting runtime on 3/10.

[Milestone 71: String.substring](./milestones/71-string-substring.md) is complete with 641 compiler groups, 656 workspace groups, editor discovery, six matched comparisons and eighteen disposed profiles. The report contains 460 separately dated rows; native leads compiled Node on 6/6 new rows and the fastest scripting runtime on 6/6.

[Native string contract](./string-semantics.md) adopts standard Rust UTF-8 storage: byte length, checked byte slices, scalar character access, and Rust trim/case conversion. [Milestone 33](./milestones/33-native-utf8-strings.md) records migration acceptance. JavaScript surrogate compatibility is superseded.

[Foundation audit](./foundation-audit.md) records current arrays, ownership, recursive cleanup, and native string direction. [Primitive follow-up](./primitive-followup.md) tracks the active migration and matching benchmark epoch. Historical UTF-16 measurements and experiments remain archived outside the crates.

The top-level [BENCHMARKS.md](../BENCHMARKS.md) records accepted equivalent observable operations through gate 72. It records optimized generated native programs, Node after `tsc`, Node type stripping, tsx, ts-node, Bun, and Deno with pinned versions, checked results, raw samples, and separate execution/startup/compilation metrics.

[Milestone 32: primitive array search](./milestones/32-array-search.md) adds read-only `includes`/`indexOf`, optional signed offsets, primitive content equality, and operand loans. The gate is complete with 15 edge regression groups and eight new comparisons, including runtime-created Unicode string arrays. All earlier 29 comparisons are reviewed again; remaining fastest-runner gaps stay explicit.

[Milestone 31: record field moves](./milestones/31-record-field-moves.md) is complete. Direct owned fields move independently, siblings remain usable, and exact replacement restores whole-record availability. Joins, loop edges, contracts, and root loans remain enforced.

[Milestone 30: owned array removal](./milestones/30-array-removal.md) is complete. It adds ordered `remove(index)` with exact owned results. Existing mutation, selection loans, callback/recursive contracts, and source-located bounds failures remain enforced.

[Milestone 29: recursive callable results](./milestones/29-recursive-callable-results.md) is complete. Eager/lazy and mutual factories resolve one environment origin before final checking. Owned captures, invocation, exact signatures, source guards, and definitive metadata remain enforced.

[Milestone 28: recursive owned callbacks](./milestones/28-recursive-owned-callbacks.md) is complete. Owned environments retain transfer/capture/invocation contracts through self and mutual calls. Iterative specialization and canonical source arrow types support finite permutations and fresh replacements; growing types fail during frontend checking.

[Milestone 27: recursive callback parameters](./milestones/27-recursive-callbacks.md) is complete. Reusable/single-use callbacks forward through self and mutual calls with exact contracts and source loans. Concrete type-vector validation bounds recursive native instantiation before emission.

[Milestone 26: recursive functions](./milestones/26-recursive-functions.md) is complete. Self/mutual data recursion infers stable ownership contracts before final checking. Native tree traversal, transfers, closures, callbacks outside recursive signatures, source guards, and optimized failures are verified.

[Milestone 25: nested record fields](./milestones/25-nested-record-fields.md) is complete. Records own named record fields inline. Source layout checks reject direct cycles and excessive layouts. Deep native literals, field updates, cloning, loans, captures, callbacks, iteration, and bounds failure order are verified.

[Milestone 24: arrays in record fields](./milestones/24-array-record-fields.md) is complete. Exact field types include recursive record arrays. Mixed storage paths support mutation, iteration, and callback arguments while loans protect the root owner. Native evaluation order and bounds failure timing are verified.

[Milestone 23: nested arrays](./milestones/23-nested-arrays.md) is complete. Exact nested types support owned rows, indexed borrowing and mutation, nested iteration, explicit cloning, and parent selection loans. Runtime failure order and bounded deep native compilation are verified.

[Milestone 22: owned array elements](./milestones/22-owned-array-elements.md) is complete. String and record elements move into native vectors. Indexing borrows; record fields support updates, and explicit cloning creates independent values. For-of borrows owned elements with enforced lexical lifetimes.

[Milestone 21: borrowed scalar array iteration](./milestones/21-for-of.md) is complete. For-of borrows named arrays, copies scalar elements into fresh const/let bindings, and supports nearest-loop break/continue. Iterable expressions evaluate once; temporary storage lives through the loop. Active loans prevent source mutation and transfer.

[Milestone 20: owned scalar arrays](./milestones/20-scalar-arrays.md) is complete. Native vectors support `int[]` and `boolean[]` literals, indexing, updates, length, push, and explicit cloning. Existing ownership, capture, callback, branch, and loop rules apply. Bounds checks remain active across optimization levels and report source byte ranges.

[Milestone 19: direct owned-callback returns](./milestones/19-callback-returns.md) is complete. Owned parameters and their moved locals can return directly from functions and arrows. Exact signature and ownership checks remain enforced; single-use restrictions survive return, forwarding, specialization, and capture without heap allocation.

[Milestone 18: callback-parameter moves](./milestones/18-callback-moves.md) is complete. Local moves preserve reusable borrows, single-use invocation, owned retention, and exact forwarding contracts. Callback locals can be restored from values derived from the same parameter; branch/loop joins and capture restrictions remain enforced.

[Milestone 17: callable declaration moves](./milestones/17-callable-moves.md) is complete. Concrete environments can move into new bindings without cloning or shared aliases. Return chains preserve the original owned escape, while local borrowed environments retain their lexical loans. Tests cover invocation, captures, callback forwarding, branch/loop availability, and backend-independent diagnostics.

[Milestone 16: mutable callable bindings](./milestones/16-mutable-callables.md) is complete. Replacement preserves concrete environment identity and restores ownership after consuming calls. Tests cover named moves, self-assignment, right-hand-side invocation, captured replacements, loans, branch/loop joins, and mismatched origin diagnostics.

[Milestone 15: capture replacement](./milestones/15-capture-replacement.md) is complete. Closures can replace captured `let` bindings. Local writers hold exclusive capture loans, and escaping writers preserve updates in owned environments. Read-only scalar captures remain snapshots; scalar writers retain inspectable mutable access. Nested closures, callback forms, loop interactions, and consuming replacement are verified.

[Milestone 14: mutable local bindings](./milestones/14-mutable-bindings.md) is complete. Initialized `let` bindings can replace scalars, strings, and records without changing type. Reinitialization restores ownership after a transfer, including on loop back edges when every continuing path restores the binding. Lexical capture loans still prevent conflicting replacement. Milestone 15 subsequently adds captured-binding replacement; milestone 16 subsequently adds mutable callable bindings with exact environment identity.

[Milestone 13: while loops](./milestones/13-while-loops.md) is complete. The compiler checks ownership on every repeating path, joins break exits with condition effects, excludes function-return paths from post-loop availability, and supports nested control targets and fresh local environments. Native-output tests, a 36-cell branch matrix, and source diagnostics cover the gate.

[Milestone 12: scalar operators](./milestones/12-scalar-operators.md) is complete. Checked integer arithmetic, comparisons, strict scalar equality, and short-circuit boolean operators support useful numeric updates and conditions. Ownership accounts for possible right-operand effects, including skipped literal branches. Runtime failures are tested across backend optimization/overflow settings. Milestone 13 subsequently adds while loops and repeated-iteration ownership.

The remaining closure gates are complete: [09: concrete callable captures](./milestones/09-callable-captures.md), [10: callable-valued arrow results](./milestones/10-callable-results.md), and [11: retained owned callbacks](./milestones/11-retained-callbacks.md). Captured environments have independent lifetimes; callable results own their captures; explicit owned callback parameters support returned wrappers through cached concrete specialization. Implementation, native acceptance tests, source diagnostics, and documentation are complete for each gate.

[Milestone 08: nested local closures](./milestones/08-nested-local-closures.md) is complete. Closure bodies create and invoke local arrows and factory results, including through synchronous callbacks. Capture effects propagate through multiple enclosing levels while preserving argument access, environment invocation, and loan lifetimes. All five gates are resolved. Milestones 09 and 10 subsequently add callable captures and callable-valued arrow results.

[Milestone 07: owned closure arguments](./milestones/07-owned-closure-arguments.md) is complete. Arrow bodies infer string/record argument access. Callable annotations expose exact `read`, `mutate`, or `consume` contracts; argument loans and environment ownership remain separate. All five gates include native-output and diagnostic evidence.

[Milestone 06: scalar closure arguments](./milestones/06-closure-arguments.md) is complete. Local closures, callable factory results, and reusable/single-use callback parameters accept zero or more typed `int` and `boolean` arguments. Signature checks and receiver/argument ownership checks pass all five exit gates. Milestone 07 subsequently adds owned argument contracts.

[Milestone 05: synchronous single-use callbacks](./milestones/05-single-use-callbacks.md) is complete. The contextual `Once<Function<Result>>` annotation transfers the callback environment and permits at most one invocation per path. Forwarding, branch joins, unused-parameter transfer, and non-escape checks are implemented and verified. All five exit gates are resolved.

[Milestone 04: synchronous reusable callbacks](./milestones/04-synchronous-callbacks.md) is complete. Zero-argument callback parameters accept named closures, inline arrows, and factory results through static dispatch. The compiler enforces reusable invocation, result types, synchronous forwarding, non-escape, and temporary capture loans. All five exit gates are resolved.

[Milestone 03: closures](./milestones/03-closures.md) is complete for local capture loans, invocation contracts, and returned owned environments. All 15 source cases now pass compiler acceptance expectations, alongside the independent [16-witness Rust experiment](./experiments/closures-01.md). The supported subset has no outstanding gate dependency; milestone 11 subsequently adds owned retained callbacks; async behavior remains deferred.

[Milestone 02: ownership across branches](./milestones/02-control-flow.md) extends the completed [ownership proof](./milestones/01-ownership-proof.md) with `boolean`, nested `if`/`else`, branch-local scope, and early returns. The project has a working [Rust compiler and shared analysis library](../crates/tsrust-compiler/README.md). It checks the bounded `.tsr` subset, infers parameter access, reports source diagnostics, and emits Rust. All 11 original source cases pass their acceptance checks. The [Rust language server](./editor-foundation.md) now shares this library for bounded single-file diagnostics. The [integer amendment](./milestones/01-integer-subset.md) defines provisional signed 64-bit copy semantics; the original numeric `User` example now compiles and prints `pat`. Arithmetic, broader numeric design, and broader research remain open.

The compiler, integration tests, fixtures, and analysis benchmark live under `crates/tsrust-compiler/`. Rust 1.96.1 is [pinned for both compilation stages](./rust-toolchain-policy.md). Run the complete compiler suite, formatting, and Clippy after native string migration. Historical test counts and timing proofs remain archived at their recorded snapshots. `build/run --release` selects optimized generated programs; no Python runner is required.

## Document authority

- Workstream documents are the editable home for their respective topics. Their original text is a proposal, not an approved specification.
- Scope summaries, dependencies, milestones, and open-decision notes are new planning suggestions. They do not silently settle any original research question.
- [The original vision](./archive/vision-original.md) is an unchanged historical snapshot. Do not maintain it alongside the workstream documents.
- “Core,” “web,” and “desktop” section IDs distinguish the three original document families; their numbering does not set priority.
- External project, namespace, license, and performance claims are carried over without fresh verification. Research work must verify those claims before relying on them.

## Workstreams

| Document | Proposed phase | Scope |
| --- | --- | --- |
| [Product vision](./product-vision.md) | Groundwork | Define the intended developer experience and differentiated product thesis. |
| [Principles and boundaries](./principles-and-boundaries.md) | Groundwork | Define compatibility boundaries, early non-goals, and decision priorities. |
| [Research program](./research-program.md) | Groundwork | Track the required design studies and unresolved cross-cutting questions. |
| [Ownership and memory](./ownership-and-memory.md) | Core proof | Specify moves, inferred access, copying, sharing, captures, allocation, cycles, and global state. |
| [Type system and data shapes](./type-system.md) | Core proof | Specify static types, object shapes, capabilities, and generics. |
| [Primitives and collections](./primitives-and-collections.md) | Core proof | Specify numeric, string, array, and iterator behavior. |
| [Async and runtime](./async-and-runtime.md) | Backend proof | Define execution, task lifetimes, cancellation, concurrency, and runtime initialization. |
| [Error handling](./error-handling.md) | Core proof | Choose recoverable-error propagation and fatal-failure behavior. |
| [Metadata and serialization](./metadata-and-serialization.md) | Backend proof | Scope reflection, annotations, code generation, and checked serialization. |
| [Compiler, modules, and diagnostics](./compiler-and-modules.md) | Core proof | Define the frontend, analyses, intermediate forms, Rust generation, modules, and source diagnostics. |
| [Interoperability and secondary targets](./interop-and-targets.md) | Backend proof; later targets | Scope safe Rust wrappers, foreign interfaces, and eventual WebAssembly support. |
| [Standard library](./standard-library.md) | Core proof; later expansion | Define library boundaries and compatibility policy; implement only the subset needed by each prototype. |
| [Toolchain and development experience](./toolchain-and-development.md) | Core proof; ongoing | Scope CLI commands, editor/debugger integration, toolchain pinning, and iteration speed. |
| [Security and supply chain](./security-and-supply-chain.md) | Groundwork; ongoing | Define unsafe boundaries, build/install permissions, and dependency visibility. |
| [Packages and registry](./packages-and-registry.md) | Ecosystem proof | Scope content-addressed storage, version identity, registry requirements, and the package-manager prototype. |
| [npm analysis and porting](./npm-analysis-and-porting.md) | Ecosystem proof; later conversion | Separate compatibility inspection from automatic conversion and upstream maintenance. |
| [Backend ecosystem](./backend-ecosystem.md) | Backend proof; later packages | Define native framework ergonomics and the first ecosystem packages. |
| [Database integration](./databases.md) | Backend proof | Scope native drivers, typed queries, schema validation, and database ergonomics. |
| [HTTP and Hono compatibility](./http-and-hono.md) | Backend proof | Define HTTP primitives, routing, middleware, compatibility levels, and independent implementation constraints. |
| [Validation](./validation.md) | Backend proof | Compare schema APIs and type-derived validation, including HTTP integration. |
| [Server HTML and JSX](./server-html.md) | Web proof | Scope typed HTML, JSX components, escaping, streaming, and the native web showcase. |
| [Source migration and compatibility tests](./source-migration.md) | Backend proof | Measure application migration and independently test documented public HTTP behavior. |
| [Identity and naming](./identity.md) | Groundwork | Research the language name, extension, CLI, and ecosystem namespaces. |
| [Competitive landscape](./competitive-landscape.md) | Groundwork | Compare neighboring projects through public evidence without incorporating their implementation code. |
| [Benchmark methodology](./benchmarks.md) | Groundwork; ongoing | Define independent equivalent workloads, runtime metrics, and fair comparisons. |
| [Core and backend prototypes](./core-and-backend-prototypes.md) | Core proof; Backend proof | Preserve the first compiler, ownership, and native HTTP demonstration requirements. |
| [Native deployment](./deployment.md) | Backend proof | Define executable deployment and runtime resource requirements. |
| [Desktop architecture and rendering](./desktop-architecture.md) | Desktop research; later implementation | Compare rendering strategies and define distinct UI/HTML targets without committing to a renderer. |
| [Desktop reactivity and ownership](./desktop-reactivity.md) | Desktop research; later implementation | Scope fine-grained updates, component lifetimes, signals, conditional trees, identity, and state. |
| [Desktop controls, layout, and accessibility](./desktop-controls.md) | Desktop research; later implementation | Scope windows, controls, styling, layout, events, bindings, menus, accessibility, and the UI module. |
| [Desktop async and platform integration](./desktop-async-and-platform.md) | Desktop research; later implementation | Scope UI-thread execution, component-owned tasks, system APIs, target checks, packaging, and assets. |
| [Desktop tooling and benchmarks](./desktop-tooling-and-benchmarks.md) | Desktop research; later implementation | Scope compile iteration, reactive inspection, ownership inspection, and UI/resource benchmarks. |
| [Desktop prototypes and shared code](./desktop-prototypes-and-shared-code.md) | Desktop research; later implementation | Preserve open desktop questions and the counter, data application, and shared server/client experiments. |

## Scoping a work item

Before implementation, write down the chosen subset, prerequisites, deliverable, acceptance examples, and explicit deferrals. Record decisions in the owning workstream and link related workstreams. Keep research conclusions separate from untested assumptions.

[Milestone 72: primitive identity](./milestones/72-primitive-identity.md) is complete with 645 compiler groups, 660 workspace groups, eight matched comparisons and twenty-four disposed profiles. Six primitive methods preserve their natural values with standard Cow costs. The report has 468 separately dated rows; native leads Node on 8/8 new rows and the fastest scripting counterpart on 4/8.

[Milestone 73: integer formatting](./milestones/73-integer-formatting.md) is complete: signed-i64 formatting in every radix 2–36, 649 compiler groups, 665 workspace groups, four comparisons and twelve disposed profiles.

[Milestone 74: Number decimal formatting](./milestones/74-number-decimal-formatting.md) is complete: standard Rust formatting, explicit native rounding differences, 653 compiler groups, 670 workspace groups, six comparisons and eighteen disposed profiles.

[Milestone 75: Number decimal prefix parsing](./milestones/75-number-prefix-parsing.md) is complete: borrowed UTF-8 input, standard Rust parsing, 657 compiler groups, 675 workspace groups, six comparisons and eighteen disposed profiles.

[Milestone 76: native String well-formedness](./milestones/76-native-string-well-formedness.md) is complete: once-only borrowed receivers, valid UTF-8 guarantees, standard Cow ownership, 661 compiler groups, 680 workspace groups, four comparisons and twelve disposed profiles.

[Milestone 77: ordinary quoted String literals](./milestones/77-quoted-string-literals.md) is implementation complete under focused compiler/editor/native acceptance. Quote decoding occurs at compile time. Benchmark comparisons are queued for checkpoint 81; the existing full regression and current editor checks pass.

[Milestone 78: ordinary template literals](./milestones/78-template-literals.md) is implementation complete. Primitive interpolation uses native Rust formatting and sequential borrowing; nested/multiline templates and editor discovery pass focused checks. Two comparisons are queued for checkpoint 81.

[Milestone 79: split storage and documentation](./milestones/79-split-storage-and-documentation.md) preserves existing limits and improves static Cow pieces. Complete split documentation and O0/O3 examples pass. Two new comparisons and three affected historical split rows await checkpoint 81.

[Milestone 80: String.substr](./milestones/80-string-substr.md) adds relative byte starts and optional byte counts with complete IntelliSense and executable examples. Four comparisons await checkpoint 81.

[Milestone 81: String HTML wrappers](./milestones/81-string-html-wrappers.md) implements thirteen legacy methods with exact tags, quote-only attributes and complete executable documentation. The combined gates 77–81 regression/measurement checkpoint follows focused acceptance.

[Checkpoint 86](../benchmarks/runtime/diagnostics/2026-10-03T14-02-43-000Z-checkpoint86-completed-proof.json) completes [Array.join](./milestones/82-array-join.md), [Array.toString](./milestones/83-array-to-string.md), [nullable foundation](./milestones/84-nullable-type-foundation.md), [strict nullable narrowing](./milestones/85-nullable-flow-narrowing.md), and [pop/shift](./milestones/86-array-pop-shift.md). It accepts 806 tested-revision checks and 41 selected comparisons with 123 fully disposed profiles; the report has 554 dated measured rows. Broader nullable-language features remain in progress.

[Milestone 87: Array.at](./milestones/87-array-at.md) accepts strict signed indices, optional copied or scoped read-only results, and verified editor/catalog examples. [Accepted implementation evidence](../benchmarks/runtime/diagnostics/2026-10-03T15-09-13-000Z-array-at-implementation.json) binds actual focused checks and native examples. Eight source comparisons are registered; checkpoint 91 measurements are accepted.

[Milestone 88: Array.find/findLast](./milestones/88-array-find.md) accepts forward/reverse strict-Boolean searches and scoped optional results. [Accepted implementation evidence](../benchmarks/runtime/diagnostics/2026-10-03T15-50-48-000Z-array-find-implementation.json) binds six verified native catalog examples with ten output lines and sixteen registered source comparisons; full regression and measurements are accepted at checkpoint 91.

[Milestone 89: Array.unshift](./milestones/89-array-unshift.md) accepts variadic same-type insertion and signed native lengths, with five verified catalog examples. [Implementation evidence](../benchmarks/runtime/diagnostics/2026-10-03T16-15-35-000Z-array-unshift-implementation.json) records focused acceptance and eight registered source pairs; checkpoint 91 measurements are accepted.

[Milestone 90: Array.splice](./milestones/90-array-splice.md) accepts signed ranges, strict variadic insertion and independently owned removed results. [Implementation evidence](../benchmarks/runtime/diagnostics/2026-10-03T16-42-24-000Z-array-splice-implementation.json) binds six verified catalog examples and eight registered source comparisons; full regression and measurements are accepted at checkpoint 91.

[Milestone 91: Array.sort/toSorted](./milestones/91-array-sorting.md) accepts stable native ordering and strict reusable comparators with six verified catalog examples. [Implementation evidence](../benchmarks/runtime/diagnostics/2026-10-03T17-35-53-000Z-array-sorting-implementation.json) records focused acceptance and 28 registered source comparisons; the combined checkpoint regression and measurements are accepted.

[Checkpoint 91 completed evidence](../benchmarks/runtime/diagnostics/2026-10-03T18-25-41-000Z-checkpoint91-completed-proof.json) accepts gates 87–91 with 890 checks, 68 native/compiled-Node comparisons and 204 disposed profiles. Native leads on 34/68; exact records live in SQLite. The next combined checkpoint is 101.

[Milestone 92: Array.flat/flatMap](./milestones/92-array-flattening.md) accepts signed-depth flattening, checked callback result types and independent owned output, with eight verified native catalog examples. [Accepted implementation evidence](../benchmarks/runtime/diagnostics/2026-10-03T19-01-32-000Z-array-flattening-implementation.json) binds sixteen registered source comparisons; the combined regression and measurements are accepted at checkpoint 96.

[Milestone 93: Array.of](./milestones/93-array-of.md) accepts typed homogeneous construction and concrete generic editor results, with six verified native catalog examples. [Accepted implementation evidence](../benchmarks/runtime/diagnostics/2026-10-03T19-34-31-000Z-array-of-implementation.json) binds eight registered source comparisons; combined regression and measurements are accepted at checkpoint 96.

[Milestone 94: nullish coalescing](./milestones/94-nullish-coalescing.md) accepts lazy typed selection, fallback-scoped absent facts and checked editor results. [Language examples](./nullish-coalescing.md) and [accepted implementation evidence](../benchmarks/runtime/diagnostics/2026-10-03T20-48-38-000Z-nullish-coalescing-implementation.json) bind six native snippets and eight registered source comparisons; combined regression and measurements are accepted at checkpoint 96.

[Milestone 95: nullish assignment](./milestones/95-nullish-assignment.md) accepts typed lazy assignment and readonly result aliases. [Language examples](./nullish-assignment.md) and [accepted evidence](../benchmarks/runtime/diagnostics/2026-10-03T21-49-17-000Z-nullish-assignment-implementation.json) bind six native snippets and eight registered comparison pairs; combined regression and measurements are accepted at checkpoint 96.

[Milestone 96: optional chaining](./milestones/96-optional-chaining.md) and its [language guide](./optional-chaining.md) cover lazy field/index/call chains, grouping and readonly ownership. [Accepted implementation evidence](../benchmarks/runtime/diagnostics/2026-10-03T23-30-44-000Z-optional-chaining-implementation.json) binds six native examples; combined regression and comparisons are accepted at checkpoint 96.

[Checkpoint96 completed evidence](../benchmarks/runtime/diagnostics/2026-10-04T01-06-08-000Z-checkpoint96-completed-proof.json) binds 1,033 retained baseline checks plus six focused refinement groups, 48 new measured pairs and 18 older affected refreshes. Of 674 registered pairs, 670 have historical measurements; four peer rows remain unmeasured. SQLite is canonical analytics, with exact original records preserved. The next combined checkpoint is 101.

[Milestone 97: nullable callable transport](./milestones/97-nullable-callable-transport.md) has focused implementation acceptance. It preserves existing callable ownership and invocation contracts inside nullable tags before optional/default parameter syntax is implemented. The [nullable callable guide](./nullable-callables.md) explains guarded calls, optional invocation, nullish assignment and absent-value conversion with verified native examples.

[Milestone 98: optional and default parameters](./milestones/98-optional-default-parameters.md) has focused implementation acceptance across current data and callable kinds. The [parameter guide](./optional-default-parameters.md) contains verified examples. Thirteen comparison pairs are registered; full regression and measurements remain queued for checkpoint 101.

[Milestone 100: dense homogeneous call spread](./milestones/100-call-spread.md) has focused implementation acceptance across current callable and built-in targets. Array-literal spread, object spread and heterogeneous tuples remain separate; full regression and measurements are queued for checkpoint 101.

[Milestone 99: typed rest parameters](./milestones/99-rest-parameters.md) has focused implementation acceptance. Its scope covers current homogeneous array element identities and callable dispatch contexts; call spread and array-literal spread remain separate work.
