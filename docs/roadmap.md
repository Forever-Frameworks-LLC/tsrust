**Gate 173 accepted:** [Named declaration signatures and inferred results](./milestones/173-nested-function-signatures.md). Omitted result types are inferred for nonrecursive functions; generic functions and functions with optional, default and rest parameters are values, also when they use variables of the enclosing function. Eight matched workloads verified untimed. Two callable-replacement witnesses stay red for the work that follows. Gate 172 was reconciled against the plan with no open requirement.

**Scoped callable records accepted:** [Borrowing callback fields](./milestones/173-scoped-callable-records.md). Record fields declared `ReadOnly<Function<...>>` or `Mutable<Function<...>>` borrow local variables with independent field storage, record parameters, replacement, moves and escape rejection. Whole compiler crate passes except two kept callable-replacement witnesses; editor 339; one matched workload verified untimed. Seven regressions in accepted features and about 30 stale expectations were repaired. Arrays, Maps, Sets and unions of such records stay rejected.

**Gate 173 source-validation progress:** Distinct branch signatures and saved alias sets now retain arity checks and declaration links. Loop, exception and capture effects remain open. See [handoff](./handoff-2026-10-06-distinct-branch-contracts.md).

**Formatter repair accepted:** Recursive `tsrust format` defaults to the working directory, with read-only `--check`, directory exclusions and stable type/assertion spacing. See [formatter handoff](./handoff-2026-10-06-recursive-format.md). Gate 173 remains in progress.

**Gate 173 in progress:** [Named declaration signatures and inferred results](./milestones/173-nested-function-signatures.md). Nonrecursive omitted results now infer scalar, generic, captured and array types; generic argument hints defer unknown results. Captured optional/default function values now preserve omission, single default evaluation, ownership and signature help. Generic/rest function values and wider context coverage remain open. Matched result/default workloads are registered and verified untimed.

**Gate 172 accepted:** [Owned nested function escape](./milestones/172-nested-function-owned-escape.md). Independent factories, retained records/arrays, inferred callable storage, consuming captures, once invocation, recursive retained owners, callback failures and exact native disposal are verified. Unused inner bodies are checked without runtime captures. Gate173 signature/context coverage and omitted-result inference remain open. No performance measurement is claimed.

**Gate 171 accepted:** [Nested function captures](./milestones/171-nested-function-captures.md). Nested named functions read and change variables of enclosing functions through per-call borrows; nested forwarding, recursive groups, later initialization, per-iteration captures, rest/callback/type parameters and use as a callable value work under the existing ownership rules. Captures 17, affected compiler batch 435 with eight depth canaries, unit 308, editor 276, harness 398 and five matched cases pass untimed. One omitted-result witness stays red for gate 173. Gate 172 (owned escape acceptance) is next; see the [full plan](./nested-function-declarations-plan.md).

**Gate 170 accepted:** [Nested function foundation](./milestones/170-nested-function-foundation.md). Hoisted block-scoped capture-free declarations, recursion, callable use, checked editor facts and nested outlines. Compiler 5, formatting 16, editor 271, harness/report 17 and five matched cases pass untimed; prior eight depth canaries retained. Future-binding and unbraced-loop editor scopes repaired. Captures and omitted-result inference remain genuine pending witnesses. See the [full plan](./nested-function-declarations-plan.md).

**Gate 169 accepted:** [Whole numeric string conversion](./milestones/169-numeric-string-conversion.md): shared decimal/radix grammar, exact numeric whitespace and radix rounding across scalars, unions and enums. Exponential member-hint traversal repaired. Compiler 80, editor 267, harness/report 17 and five matched cases pass untimed. SQLite records verified; consumers refreshed; outputs pruned. No timing.

**Gate 168 accepted:** [Scalar unary plus](./milestones/168-scalar-unary-plus.md): scalar conversion and Number scalar unions, enum payloads, borrowing and editor result types; catalog constructor documentation and direct type diagnostics. Compiler 72, editor 266, harness/report 17 and five matched cases pass untimed. Five consumers refreshed; SQLite evidence retained. No timing.

**Gate 167 accepted:** [Scalar string concatenation](./milestones/167-string-concatenation.md): scalar operands and scalar unions in string addition and compound addition, preserving source ordering and string editor results. Compiler 65, editor 265, registration 1 and five matched cases pass untimed. Five consumers refreshed; verification retained in SQLite. No timing.

**Gate 166 accepted:** [Interfaces](./milestones/166-interfaces.md): `interface` record types with `extends` over record types in any order and across modules, optional fields and type parameters; editor symbols with inherited fields. Compiler 4 focused plus 310 affected, editor 264, grammar 1, harness 393, five matched cases verified untimed. No timing.

**Gate 165 accepted:** [Optional record fields](./milestones/165-optional-fields.md): `name?: Type` fields that literals may omit, with fixed record shape, editor field types and completion. Compiler 3 focused plus 306 affected, editor 263, harness 392, five matched cases verified untimed. No timing.

**Gate 164 accepted:** [String indexing](./milestones/164-string-indexing.md): `text[position]` by Unicode character position with array-consistent range errors, write rejection and editor types. Repairs: absent optional selections compared with `undefined`, hover type of comparison initializers, formatting-insensitive catalog fixture check. Compiler 5 focused plus 151 affected, editor 262, harness 391, five matched cases verified untimed. No timing. The optional form `text?.[position]` is supported.

**Gate 163 accepted:** [Type aliases](./milestones/163-type-aliases.md): `type Name = Type;` for unions, arrays, tuples, primitives, collections and function types, across modules, with editor hover/definition/members. Repairs: owned union call comparison, two depth canaries, linker panic on alias import, archive-dependent harness tests. Compiler 6 focused plus affected batches, editor 2, harness 390, five matched cases verified untimed. No timing. Six baseline editor failures remain open.

**Gate 162 accepted:** [Comma contexts](./milestones/162-comma-contexts.md): named array receivers retain indexed parent loans; selected sentinel/typeof guards invalidate on RHS source mutation. Compiler 58 distinct focused checks, editor 2, harness 389 and matched five-case verification pass untimed; five consumers refreshed. No full regression or timing. Next gate163: choose remaining basic syntax/operations after probing current support.

**Gate 161 accepted:** [Comma expressions](./milestones/161-comma-expressions.md): ordered discard/final values, lexical borrows, guard facts, owned discard and preserved destructuring in for headers. Compiler 22, editor 1, harness 388 and five-case matched verification pass untimed; five consumers refreshed. No full regression or timing. Next: audit remaining expression contexts and basic operations.

**Gate 160 accepted:** [Numeric literals](./milestones/160-numeric-literals.md): separators and binary/octal/hex forms with preserved int bounds, contextual f64 rounding, source ranges and formatting. Compiler 45 focused checks, editor 3, grammar 1, harness 387 and five-case matched verification pass untimed; five consumers and installed grammar refreshed. No full regression or timing. Next: comma expressions.

**Gate 159 accepted:** [Map.groupBy](./milestones/159-map-group-by.md): ordered independently owned groups from array/string/Set/Map sources, readonly two-position callbacks, defaults/rest and catchable failures. Compiler 10 focused checks, editor 3, verified example, harness 386 and five-case matched verification pass untimed; five consumers refreshed. No full regression or timing. Next: numeric literal separators and radix forms.

**Gate 158 accepted:** [Iterable call spread](./milestones/158-iterable-call-spread.md): ordered string/Set/Map arguments with checked ownership, generics and catchable dynamic arity errors. Optional-chain hint defect repaired. Compiler 73 focused checks, editor 4, seven examples, harness 385 and five-case matched verification pass untimed; five consumers refreshed. No full regression or timing. Next: Map.groupBy.

**Gate 157 accepted:** [Iterable array spread](./milestones/157-iterable-array-spread.md): ordered string/Set/Map expansion, scalar copying and owned tuple transfer; contextual empties and union elements. Parser call-depth overflow repaired. Compiler 41 focused checks, editor 5, eight examples, harness 384 and five-case matched verification pass untimed; five consumers refreshed. No full regression or timing. Next: collection call spread.

**Gate 156 accepted:** [Array.from collections](./milestones/156-array-from-collections.md): independent Set/Map copies, borrowed mapping of noncloneable values and correct two-position defaults/rest. Callable array mapping preserved; typed editor results and full catalog docs/examples updated. Compiler 32 focused checks, editor 4/4, six examples, harness 383/383 and three matched workloads pass untimed; five consumers refreshed. No full regression or timing. Next: iterable array-literal spread.

**Gate 155 accepted:** [String iteration](./milestones/155-string-iteration.md): readonly Unicode characters, once evaluation, source loans and normal completion. Compiler 19 focused checks, editor 2/2, harness 382/382 and matched verification pass untimed; five consumers refreshed. No full regression or timing. Next gate 156: Array.from Set/Map sources.

**Gate 154 accepted:** [Borrowed Map iteration](./milestones/154-borrowed-map-iteration.md): readonly tuple entries and patterns, noncloneable values and typed editor bindings. Final compiler groups 22/22, prior callbacks/snapshots 6/6, editor 6/6, harness 381/381 and matched verification pass untimed; five consumers refreshed. No full regression or timing. Next gate 155: borrowed string iteration.

**Gate 153 accepted:** [Independent Set algebra](./milestones/153-set-algebra.md): union/intersection/difference/symmetricDifference with joined typing, explicit owned copies and Node insertion order. Native precision edge and Set/Map union-array/type-name hover defects repaired. Compiler 26 distinct focused checks, editor 12/12, seventeen catalog examples, harness 380/380 and matched untimed verification pass; five consumers refreshed. No full regression or timing. Next gate 154: borrowed Map iteration.

**Gate 152 accepted:** [Borrowed Set relations](./milestones/152-set-relations.md): isSubsetOf/isSupersetOf/isDisjointFrom across current key types, without owned key copies. Compiler 13/13, editor 6/6, thirteen catalog examples, harness 379/379 plus final registration and matched untimed verification pass; consumers refreshed. No full regression or timing. Next gate 153: Set algebra.

**Gate 151 accepted:** [Borrowed Map callbacks](./milestones/151-map-for-each.md): ordered value/key/source traversal without owned input clones, reusable abstract/default/rest callbacks, immediate return discard and checked errors/finally. Compiler 23/23, editor 7/7, eleven catalog examples, harness 378/378 and matched untimed verification pass; five consumers refreshed. No full regression or timing.

**Gate 150 accepted:** [Checked array callback errors](./milestones/150-array-callback-errors.md): thirteen traversal/sorting methods propagate callback failures into catch/finally. Fallible sorting preserves source order on failure without copying payloads; readonly abstract dispatch repaired. Compiler 34/34, editor 3/3, harness 377/377 and matched untimed verification pass; five consumers refreshed. No full regression or timing. Next gate 151: Map.forEach.

**Gate 149 accepted:** [Borrowed Set callbacks](./milestones/149-set-for-each.md): ordered forEach with duplicate key/readonly source, no input clones, defaults/rest and checked errors. Repairs Array.forEach Result propagation and readonly abstract dispatch. Compiler 7/7 plus Array 4/4, editor 1/1 plus Set 4/4, harness 375/375 and matched untimed verification pass; consumers refreshed. No full regression or timing. Next gate 150: broader array callback error transport.

**Gate 148 accepted:** [Set entry snapshots](./milestones/148-set-entries.md): ordered independent key/key tuples, explicit owned copies, union/enum/generic types and typed editor help. Compiler 2/2, editor 1/1, adjacent collection checks, harness 374/374 and matched untimed verification pass; consumers refreshed. No full regression or timing. Next gate 149: collection callbacks.

**Gate 147 accepted:** [Independent Map snapshots](./milestones/147-map-snapshots.md): typed keys/values/entries arrays, explicit owned copies, insertion order and tuple editor support. Four compiler snapshot tests pass across focused batch/supplement, editor 1/1 plus Map 4/4, harness 373/373 and matched untimed verification pass; consumers refreshed. No full regression or timing. Collection family remains open. Next gate 148.

**Gate 146 accepted:** [Typed Map collections](./milestones/146-maps.md): scalar/enum/union keys, owned values, inferred/contextual construction, explicit copies, SameValueZero and size/set/get/has/delete/clear. Map compiler 6/6, editor 4/4, harness 372/372 and matched untimed verification pass; five consumers refreshed. Collection family remains open; no full regression or timing. Next gate 147.

**Gate 145 accepted:** [Typed Set collections](./milestones/145-sets.md): scalar/enum/union keys, explicit source copies, SameValueZero, mutation, borrowed iteration and array snapshots. Six compiler and four editor checks pass, harness 371/371, matched complete scenario verified untimed, consumers refreshed. Collection family remains open; no full regression or timing. Next gate 146.

**Gate 144 accepted:** [Nominal enums](./milestones/144-enums.md): numeric/string members, constant flags, copyable values, generics, arrays, record fields and imported aliases. Focused compiler/editor checks, all 370 harness checks accounted for, and matched untimed verification pass. No full regression or timing. Next gate 145.

**Gate 143 accepted:** [Checked type assertions](./milestones/143-checked-type-assertions.md): compatible `as`, source-preserving `satisfies`, contextual arrays/records/callables, explicit integer widening and preserved borrows. Six new compiler tests, focused regression canaries, four editor checks, 369 harness checks and matched untimed verification pass. No full regression or timing. Next gate 144.

**Gate 142 accepted:** [Checked exception handling](./milestones/142-exception-handling.md): `try`/`catch`/`finally` and `throw` with completion transport, structured runtime errors, catchable testing failures and editor support for the catch name. Acceptance repaired matcher catching, closure-origin renumbering and three stack-depth regressions. Bounded compiler pass, 289 unit tests, 225 editor tests, 368 harness checks and matched untimed verification pass on refreshed consumers. No timing. Next gate143.

**Gate 141 accepted:** [Callable expression invocation](./milestones/141-callable-expressions.md): factory results, nested calls and arrow/function IIFEs, with defaults/rest/spread and existing ownership contracts. Repairs include temporary capture lifetime, inferred spread/undefined arguments and computed-call editor types/signature help. Eight focused compiler suites (104 checks plus affected supplements), two gate-specific editor checks, matched untimed verification and harness365/365 pass. No full regressions or timing. Next gate142.

**Gate 140 accepted:** [Labeled statements and loop control](./milestones/140-labeled-control.md): labeled blocks/loops and break/continue with target updates, ownership joins and a separate editor namespace. Definite-assignment checks now follow labeled exits and continuing tails. Eight focused compiler suites, two editor checks, three matched untimed workloads and harness363/363 pass. No full regressions or timing. Next gate141.

**Gate 139 accepted:** [Own-key membership and key enumeration](./milestones/139-in-operator.md): `in` on records/tuples/arrays with key-before-value evaluation, and snapshot `for...in` enumeration. Focused native/editor/formatter checks, independent Node assertions, matched untimed verification and harness361/361 pass. Long membership chains reject safely. No full regressions or timing. Next gate140.

**Gate 138 accepted:** [`let` without an initializer](./milestones/138-deferred-initialization.md): `let name: Type;` with a definite-assignment check after analysis; types that include `undefined` start as `undefined`. Focused compiler suites, all editor tests, two matched untimed workloads and the runtime harness pass on refreshed consumers. No timing or full compiler regression. Next gate139.

**Gate 137 accepted:** [Global number names](./milestones/137-global-number-names.md): `parseInt`, `parseFloat`, `isNaN`, `isFinite`, `NaN` and `Infinity` without the `Number.` prefix, with catalog documentation, hover, completion, signature help and highlighting. Repairs: `-Infinity` typing, linker handling of the names in programs with imports, and editor types for signed initializers. Focused compiler suites, all editor tests, two matched untimed workloads and all 357 harness checks pass on refreshed consumers. No timing or full compiler regression. Next gate138.

**Gate 136 accepted:** [Function expressions](./milestones/136-function-expressions.md): `function (...) { ... }` as a value with arrow-function semantics, editor parameter and binding support, and a repair for the command-line broken-pipe panic. Fifteen compiler suites, all 215 editor tests, two matched untimed workloads and all 355 harness checks pass on refreshed consumers. No timing or full compiler regression. Next gate137.

**Gate 135 accepted:** [Switch statements](./milestones/135-switch-statements.md): strict-equality cases in source order, grouped labels, default in any position, loop `continue`, and union narrowing in a matching clause; fall-through with statements and nested `break` are rejected with guidance. Formatter lays out clauses and spaces conditional colons. Twelve compiler suites, all 214 editor tests, three matched untimed workloads and all 353 harness checks pass on refreshed consumers. No timing or full compiler regression. Next gate136.

**Gate 134 accepted:** [Statement bodies without braces](./milestones/134-statement-bodies.md): one statement can be the body of if, else, while, do, for and for...of. Twelve compiler suites and all 213 editor tests pass; three matched workloads verified untimed. Harness: 350 of 351 passed, and the one failure came from an error in the new workload source, repaired with a 22-check affected-group rerun. No timing or full compiler regression. Next gate135.

**Gate 133 accepted:** [Destructuring assignment and shorthand record properties](./milestones/133-destructuring-assignment.md): `[a, b] = [b, a];`, tuple and record pattern assignment to existing places, rest targets, use in classic-for update clauses, and `{ x, y }` record literals. Fifteen compiler suites, all 212 editor tests, three matched untimed workloads and all 349 harness checks pass on refreshed consumers. No timing or full compiler regression. Next gate134.

**Gate 132 accepted:** [Declaration lists and loop update lists](./milestones/132-declaration-lists.md): several declarators per `const`/`let`, including in classic-for initializers, and comma-separated update expressions. Formatter spacing and two long-failing editor catalog tests are repaired. Focused compiler checks, all 211 editor tests, three matched untimed workloads and all 347 harness checks pass on refreshed consumers. No timing or full compiler regression. Next gate133.

**Gate 131 accepted:** [Classic-for pattern initializers](./milestones/131-for-pattern-initializers.md) evaluate the source once and scope the names to the loop, with defaults, rest and nested patterns. Focused native/editor/Node checks, three matched untimed workloads and all 345 harness checks pass on refreshed consumers. No timing or full compiler/LSP regression. Next gate132.

**Gate 130 accepted:** [Nested array-rest targets](./milestones/130-nested-rest-patterns.md) include array/object targets, parameter defaults, tuple positions, arrows and for...of. Focused native/editor/Node checks and five matched untimed cases pass. Harness343 checks accounted for across337 original passes and repaired executable failures. No timing or full compiler/LSP regression. Next gate131.

**Gate 129 accepted:** [Explicit rest and object-spread copies](./milestones/129-rest-and-object-spread.md) cover array/tuple/record rest, fixed-record spread/merges, ownership, IntelliSense and current consumers. Rest/spread23 plus adjacent suites pass; runtime harness341/341 and five matched cases pass untimed. Nested owners copy independently. No timing or full compiler/LSP regression. Next gate130.

**Gate 128 accepted:** [Object.hasOwn](./milestones/128-object-has-own.md) includes compiler/native assertions, full IntelliSense, current consumers and five matched untimed cases. Runtime harness339/339 plus76 affected checks pass. Spread/rest copying is explicitly authorized; mixed default sorting stays unsupported. Next gate129 targets remaining basic syntax. No timing or full compiler/LSP regression.

**Gate 127 current-source acceptance restored (October 5):** All16 default native checks, three editor checks, current consumer rebuilds and five untimed matched cases pass after coordination. Harness evidence covers337 checks across the original335 passes and two repaired checks. See [the final milestone supplement](./milestones/127-destructuring-defaults.md). Older pending-refresh notes below are historical.

**Gate 127 final refresh pending:** Default implementation and 16 focused native checks pass, but concurrent std:testing compiler edits interrupt final debug/release builds. Coordinate one writer, then refresh consumers and verify the affected pair before accepting current source. See [the latest milestone supplement](./milestones/127-destructuring-defaults.md).

**Gate 127 focused acceptance:** [Destructuring defaults](./milestones/127-destructuring-defaults.md) support lazy undefined-only fallbacks, nested patterns and left-to-right parameter initialization without cloning. Fresh borrowed fallback owners persist locally; lexical source loans remain. Five matched cases verified untimed. Rest copying still awaits the owner decision; next independent candidates are last-use borrows and mixed-alternative default sort.

**Gate 126 focused acceptance:** [Borrowed optional string fallbacks](./milestones/126-borrowed-optional-fallback.md) accept `item?.label ?? "none"` without cloning, preserve lexical source loans, and provide a matched comparison verified untimed. Destructuring rest semantics await an owner decision; defaults remain next.

**Gate 125 focused acceptance:** [Parameter patterns and indexed borrows](./milestones/125-parameter-patterns-and-indexed-borrows.md): function and arrow parameters accept patterns that borrow by default, and a const taken from an indexed element borrows it, which also enables array patterns over owned elements. Strict equality with one primitive value narrows a union variable.

**Gate 124 focused acceptance:** [Destructuring and borrowed const bindings](./milestones/124-destructuring.md): array and record patterns in declarations and for...of follow the access of their source, and a const taken from a loop element or ReadOnly parameter borrows it. One matched comparison verified untimed.

**Gate 123 focused acceptance:** [Record display order and union strict equality](./milestones/123-record-display-order-and-union-equality.md): console display follows each record's constructor order, `===`/`!==` compare union values with primitives and primitive unions, and untyped mixed array literals take a union element type.

**Gate 122 focused acceptance:** [Object.values and Object.entries](./milestones/122-object-values-entries.md) copy record fields in constructor order, with a union element type for mixed fields, complete editor documentation and one matched comparison verified untimed.

**Gate 121 focused acceptance:** [Union array elements](./milestones/121-union-array-elements.md) store mixed alternatives per element with narrowing, optional results, primitive-union search, text conversion and cloning, with editor support and one matched comparison verified untimed. Object.values and Object.entries are next.

**Gate 120 focused acceptance:** [General data unions](./milestones/120-data-unions.md) preserve mixed data tags, ownership, recursive copying, typeof narrowing and mutable spread loans, with verified editor examples and one matched comparison verified untimed. Union array leaves are next.

**Gate 119 focused acceptance:** [Object.keys](./milestones/119-object-keys.md) preserves per-value record constructor order through clones and fixed/recursive storage, with array/tuple/UTF-16 string keys, complete editor documentation and one matched comparison verified untimed. Heterogeneous value representation is the next enumeration dependency.

**Gate 118 focused acceptance:** [Array.entries](./milestones/118-array-entries.md) returns eager independent typed index/value pairs, with generic tuple indexing repairs, complete editor documentation and two matched comparisons verified untimed. Record enumeration foundations are next.

**Gate 117 focused acceptance:** [Fixed positional tuple types](./milestones/117-tuple-types.md) supply structural positional types, aggregate ownership, tuple arrays, generic/imported contexts and editor documentation. One matched comparison passes untimed; Array.entries is next.

**Gate 116 focused acceptance:** [String.normalize](./milestones/116-string-normalize.md) implements all four forms with pinned Unicode data, complete conformance evidence, optional-form handling, editor documentation and one registered comparison verified untimed.

**Gate 115 focused acceptance:** [Array.toLocaleString](./milestones/115-array-locale-string.md) adds borrowed recursive display for every supported array leaf kind, with editor documentation and one registered comparison verified untimed.

**Gate 114 focused acceptance:** [Numeric toLocaleString](./milestones/114-number-locale-string.md) adds fixed decimal grouping and rounding for numbers and exact grouping for integers, with Node-checked fixtures, editor documentation and one registered comparison verified untimed.

**Gate 113 focused acceptance:** [String.raw](./milestones/113-string-raw.md) joins raw string-array segments with converted values and copied string-array spread, with editor documentation and one registered comparison verified untimed.

**Gate 112 focused acceptance:** [String toLocaleLowerCase and toLocaleUpperCase](./milestones/112-string-locale-case.md) apply the locale-independent case mapping and reject locale arguments, with editor documentation and one registered comparison verified untimed.

**Gate 111 focused acceptance:** [String codePointAt and charCodeAt](./milestones/111-string-code-at.md) read codes by character position with `int | undefined` and `number` results, with editor documentation and one registered comparison verified untimed.

**Gate 110 focused acceptance:** [String.fromCodePoint and String.fromCharCode](./milestones/110-string-from-codes.md) build strings from int codes, including spread arrays, with Node-checked results, editor documentation and one registered comparison verified untimed.

**Gate 109 focused acceptance:** [Math.random](./milestones/109-math-random.md) returns a uniform number from 0 up to 1 with a per-run sequence, with editor documentation and one registered comparison verified untimed.

**Gate 108 focused acceptance:** [Math.f16round](./milestones/108-math-f16round.md) rounds to half precision with Node-checked results, editor documentation and one registered comparison verified untimed. The gate also fixed tied shortest digits in number text and number text inside template literals.

**Gate 107 focused acceptance:** [Object.is](./milestones/107-object-is.md) compares two values with same-value rules for numbers and storage identity for arrays and records, with editor documentation and one registered comparison verified untimed.

**Gate 106 focused acceptance:** [number.toString](./milestones/106-number-to-string.md) formats numbers in radix 2 through 36 with output identical to Node on the reference fixtures, editor documentation and one registered comparison verified untimed.

**Gate 105 focused acceptance:** [Number.parseInt](./milestones/105-number-parse-int.md) reads integer prefixes in radix 2 through 36 with Node-checked results, editor documentation and one registered comparison verified untimed.

**Gate 104 focused acceptance:** [String.at](./milestones/104-string-at.md) selects a character from either end with a `string | undefined` result, with editor documentation and one registered comparison verified untimed.

**Gate 103 focused acceptance:** [Array keys and values](./milestones/103-array-keys-values.md) return ordinary independent arrays, with editor documentation and two registered comparisons verified untimed.

**Gate 102 focused acceptance:** [Array.from](./milestones/102-array-from.md) copies arrays, splits strings into characters and applies an optional mapping function, with editor documentation and four registered comparisons verified untimed.

**Gate 101 focused acceptance:** [Array-literal spread](./milestones/101-array-literal-spread.md), with heavy regression and native/compiled-Node timing deferred until a meaningful feature-family milestone.

**Regression repairs before gate 101:** [Callable and depth regression repairs](./milestones/callable-regression-repairs.md) fix forwarded mutable callbacks, nullable callable and capture lowering, lost arrow parameter diagnostics and normal-stack depth, and migrate fixtures that still assumed borrowed callable defaults. Targeted suites only.

**Completed call-spread focused implementation:** [Milestone 100](./milestones/100-call-spread.md) accepts dense homogeneous call spread across current callable and built-in targets, with a verified [guide](./call-spread.md) and thirteen registered comparisons verified untimed. [Evidence](../benchmarks/runtime/diagnostics/gate100-call-spread-implementation.json) lists focused checks and outstanding items. Gate 99 is closed; full regression and accumulated measurements remain deferred to a completed feature-family milestone.

**Closed rest-parameter gate:** [Milestone 99](./milestones/99-rest-parameters.md) has focused acceptance for homogeneous owning rest packs across current callable contexts. [Evidence](../benchmarks/runtime/diagnostics/gate99-rest-parameter-implementation.json) preserves integrated checks and 13 registered comparisons; [Exact archival and gate closure](../benchmarks/runtime/diagnostics/gate99-focused-closure.json) are complete. Full regression and measurements are queued for checkpoint 101.

**Completed optional/default parameter focused implementation:** [Milestone 98](./milestones/98-optional-default-parameters.md) accepts lazy defaults and omission across current data and callable kinds, generic/imported signatures and verified editor help. [Acceptance evidence](../benchmarks/runtime/diagnostics/gate98-optional-default-implementation.json) preserves focused checks and original failures. Thirteen comparisons are registered; full regression and measurements remain deferred to a completed feature-family milestone.

**Completed nullable callable focused implementation:** [Milestone 97](./milestones/97-nullable-callable-transport.md) accepts typed nullable transport across current callable ownership and invocation contracts, verified editor/native examples and ten registered comparison pairs. [Acceptance evidence](../benchmarks/runtime/diagnostics/2026-10-04T04-22-29-000Z-nullable-callable-implementation.json) retains scoped checks and failures; measurements and full regression remain deferred to a completed feature-family milestone. [Milestone 98](./milestones/98-optional-default-parameters.md) scopes optional/default parameters across current data and callable kinds.

**Checkpoint96 accepted:** [Completed evidence](../benchmarks/runtime/diagnostics/2026-10-04T01-06-08-000Z-checkpoint96-completed-proof.json) binds 1,033 retained baseline checks and six new focused concat-refinement groups, without a full restart. Forty-eight new pairs and 18 older affected refreshes are measured; 670 of 674 registered pairs have historical measurements, with four peer rows excluded. Original records and refreshed samples remain distinct in canonical SQLite analytics. Native/compiled-Node ratios review whole-workload costs; no universal win is claimed. The current batch is gates 97–101, with the next combined checkpoint at 101.

**Completed optional chaining focused implementation:** [Milestone 96](./milestones/96-optional-chaining.md) accepts lazy typed contiguous chains, readonly selected views and compiler-derived successful-hop editor help. [Accepted evidence](../benchmarks/runtime/diagnostics/2026-10-03T23-30-44-000Z-optional-chaining-implementation.json) binds retained checks and six O0/O3 language examples; registered comparison sources are measured at checkpoint 96.

**Completed nullish assignment focused implementation:** [Milestone 95](./milestones/95-nullish-assignment.md) accepts lazy typed commit, readonly owned-result aliases, scoped absence/postcommit facts and verified editor views. [Accepted evidence](../benchmarks/runtime/diagnostics/2026-10-03T21-49-17-000Z-nullish-assignment-implementation.json) binds four retained compiler groups, scoped supplements, six O0/O3 language examples and eight registered comparisons across 665 source rows. Combined regression and measurements are accepted at checkpoint 96.

**Completed nullish coalescing focused implementation:** [Milestone 94](./milestones/94-nullish-coalescing.md) accepts lazy typed selection, scoped RHS absence facts and exact checked editor results. [Accepted evidence](../benchmarks/runtime/diagnostics/2026-10-03T20-48-38-000Z-nullish-coalescing-implementation.json) binds four retained compiler groups, the editor/RHS supplement, six O0/O3 language examples and eight registered comparisons across 656 source rows. Full regression and measurements are accepted at checkpoint 96.

**Completed constructor focused implementation:** [Milestone 93](./milestones/93-array-of.md) accepts inferred/contextual or explicit typed homogeneous construction, ordered ownership transfer and checked editor result types. [Accepted evidence](../benchmarks/runtime/diagnostics/2026-10-03T19-34-31-000Z-array-of-implementation.json) binds four compiler groups, six native catalog examples and eight registered comparisons across 648 source rows. Full regression and measurements are accepted at checkpoint 96.

**Completed flattening focused implementation:** [Milestone 92](./milestones/92-array-flattening.md) accepts typed flattening, fixed callback results and independent owned output. Four compiler groups and one editor group pass; eight catalog examples and the affected flat supplement pass at O0/O3. [Accepted evidence](../benchmarks/runtime/diagnostics/2026-10-03T19-01-32-000Z-array-flattening-implementation.json) binds sixteen registered comparisons and 640 source rows. Full regression and measurements are accepted at checkpoint 96.

**Checkpoint 91 accepted:** [Completed evidence](../benchmarks/runtime/diagnostics/2026-10-03T18-25-41-000Z-checkpoint91-completed-proof.json) binds 890 distinct tested-revision checks with retained passes and targeted repairs, without a full restart. Sixty-eight native/compiled-Node comparisons and 204 disposed profiles are accepted; native is faster on 34/68. SQLite preserves exact records and restoration. There are 622 historically measured workloads among 623 registered sources; the peer path row remains unmeasured. The subsequent gates 92–96 batch is accepted at checkpoint 96.

**Completed sorting focused implementation:** [Milestone 91](./milestones/91-array-sorting.md) accepts stable native default/comparator ordering, mutating sort and independent toSorted copies. Four compiler groups and one editor group pass; six catalog examples produce seven exact lines at O0/O3. [Accepted evidence](../benchmarks/runtime/diagnostics/2026-10-03T17-35-53-000Z-array-sorting-implementation.json) binds 28 registered comparisons. The combined gates 87–91 regression and measurements are accepted at checkpoint 91.

**Completed Array.splice focused implementation:** [Milestone 90](./milestones/90-array-splice.md) accepts signed range removal, same-type insertion, once-ordered arguments and independent removed results. Four compiler groups plus a source supplement and one editor group pass; six catalog examples produce nine exact lines at O0/O3. [Accepted evidence](../benchmarks/runtime/diagnostics/2026-10-03T16-42-24-000Z-array-splice-implementation.json) binds eight registered comparisons. Full regression and measurements are accepted at checkpoint 91.

**Completed Array.unshift focused implementation:** [Milestone 89](./milestones/89-array-unshift.md) accepts strict variadic front insertion, new int length, once-ordered arguments and zero-argument Read behavior. Four unique compiler groups and one editor group pass; five catalog examples produce six exact lines at O0/O3. [Accepted evidence](../benchmarks/runtime/diagnostics/2026-10-03T16-15-35-000Z-array-unshift-implementation.json) binds eight registered comparisons. Full regression and measurements are accepted at checkpoint 91.

**Completed Array.find/findLast focused implementation:** [Milestone 88](./milestones/88-array-find.md) accepts strict reusable Boolean predicates, forward/reverse early exit, original indices and scoped optional result views. Four compiler groups plus a literal-context supplement and one editor group pass; six exact catalog examples produce ten lines at O0/O3. [Accepted implementation evidence](../benchmarks/runtime/diagnostics/2026-10-03T15-50-48-000Z-array-find-implementation.json) binds sixteen registered source comparisons; full regression and measurements are accepted at checkpoint 91.

**Completed Array.at focused implementation:** [Milestone 87](./milestones/87-array-at.md) accepts strict signed indexing, canonical optional results, scoped read-only owned-element views and complete executable IntelliSense examples. Four compiler groups and affected supplements pass; one editor group executes three catalog examples with five output lines at O0/O3. [Accepted implementation evidence](../benchmarks/runtime/diagnostics/2026-10-03T15-09-13-000Z-array-at-implementation.json) binds the actual attempts and example outputs. Eight comparison sources are registered, with measurements and the combined regression accepted at checkpoint 91. SQLite remains canonical analytics; Markdown exports are optional.

**Checkpoint 86 accepted:** [Completed evidence](../benchmarks/runtime/diagnostics/2026-10-03T14-02-43-000Z-checkpoint86-completed-proof.json) binds 806 named checks on the tested revision without a full restart. [Runtime evidence](../benchmarks/runtime/diagnostics/2026-10-03T13-57-02-000Z-checkpoint86-runtime-proof.json) accepts 41 selected comparisons, 30,504 timed batches and 123 fully disposed profiles. Native leads compiled Node on 18/41 and the fastest scripting runtime on 11/41; the report contains 554 individually dated measured rows. Latest concurrent project/editor revisions are outside this checkpoint claim. The following batch, gates 87–91, is accepted at checkpoint 91.

**Completed optional array removal implementation:** [Milestone 86](./milestones/86-array-pop-shift.md) accepts native `pop()` and `shift()`, checked optional results, owned transfer and complete IntelliSense examples. Four compiler groups and one editor group pass. Sixteen comparisons are measured at checkpoint 86.

**Completed nullable narrowing implementation:** [Milestone 85](./milestones/85-nullable-flow-narrowing.md) accepts strict null/undefined guards, checked payload projection, sound mutation invalidation and cached editor views. Four retained compiler groups, one editor group and eleven native documentation lines pass. Six comparisons and the retained checkpoint regression are accepted.

**Completed nullable foundation:** [Milestone 84](./milestones/84-nullable-type-foundation.md) accepts canonical nullable types, owned transport and whole-value observations. Three arena groups, four compiler groups and verified editor/native examples pass. General payload unions and broader narrowing guards remain separate work. Six comparisons are measured at checkpoint 86.

**Completed Array.toString implementation:** [Milestone83](./milestones/83-array-to-string.md) adds strict default-comma array text and removes diagnostic allocations from existing String(array). Four compiler groups and seventeen native catalog examples pass; six new comparisons and the affected array-conversion refresh are measured at checkpoint 86.

**Completed Array.join implementation:** [Milestone 82](./milestones/82-array-join.md) adds direct borrowed iteration into standard Rust String, with supported homogeneous leaves and nested arrays. Four compiler groups, complete editor contracts and eleven native documentation examples pass. Six matched comparisons are measured at checkpoint 86.

**Checkpoint 81 accepted:** gates 77–81 retain complete IntelliSense examples and pass 700 compiler integration groups / 735 workspace groups without a full restart. [Runtime evidence](../benchmarks/runtime/diagnostics/2026-10-03T08-44-00-000Z-checkpoint81-runtime-proof.json) accepts 29 selected comparisons and 87 disposed allocation samples; 514 report rows retain their actual run dates. The following gates 82–86 are accepted at checkpoint 86.

**Completed String.substr:** [Milestone 80](./milestones/80-string-substr.md) accepts optional strict int byte bounds, UTF-8 boundary errors, Cow ownership and complete executable IntelliSense documentation. Four comparisons are accepted at checkpoint 81.

**Completed split foundation audit:** [Milestone 79](./milestones/79-split-storage-and-documentation.md) preserves u32 limits, borrows static pieces, copies owned pieces independently and accepts complete split IntelliSense documentation/examples. New comparisons and affected historical rows are accepted at checkpoint 81.

**Completed template implementation:** [Milestone 78](./milestones/78-template-literals.md) accepts ordinary nested/multiline templates and primitive interpolation with focused ownership, disposal, diagnostics, editor and native example evidence. Two comparisons are accepted at checkpoint 81.

**Acceptance cadence:** [Implementation-first focused TDD and complete built-in IntelliSense documentation](./development-workflow.md) are required as features land. Heavy compiler/LSP regressions and benchmark reviews wait until a meaningful feature-family milestone is complete. The former five-gate schedule is superseded; historical accepted checks remain retained.

**Completed quoted String implementation:** [Milestone 77](./milestones/77-quoted-string-literals.md) accepts both quote delimiters, strict escapes, Unicode code points and line continuations through focused compiler/editor/native checks and verified catalog examples. The existing full regression and current editor checks pass; two performance comparisons are accepted at checkpoint 81.

**Completed native String gate:** [Milestone 76](./milestones/76-native-string-well-formedness.md) accepts strict zero-argument well-formedness methods through valid UTF-8 and standard Cow ownership. 661 compiler groups and 680 workspace groups pass in one full regression; four comparisons and twelve disposed profiles pass. Native leads compiled Node on 4/4 rows and the fastest scripting runtime on 4/4.

**Completed prefix parsing gate:** [Milestone 75](./milestones/75-number-prefix-parsing.md) accepts strict-string `Number.parseFloat` using a longest-prefix adapter and standard Rust float parsing. 657 compiler groups and 675 workspace groups pass with one full regression; six comparisons and eighteen disposed profiles pass. Native leads compiled Node on 6/6 rows and the fastest scripting runtime on 6/6. Rates measure complete pool setup, selection, parsing and Number result retention.

**Completed decimal formatting gate:** [Milestone 74](./milestones/74-number-decimal-formatting.md) accepts fixed, exponential and significant Number formatting using standard Rust formatters with explicit native rounding differences. 653 compiler groups and 670 workspace groups pass with one full regression; six comparisons and eighteen disposed profiles pass. Native leads compiled Node on 3/6 rows and the fastest scripting runtime on 0/6. Rates include pool setup, selection, formatting and complete result retention.

**Completed integer formatting gate:** [Milestone 73](./milestones/73-integer-formatting.md) accepts signed-i64 formatting in every radix 2–36, including both extremes and strict optional arguments. 649 compiler groups and 665 workspace groups pass in one full regression; four comparisons and twelve disposed profiles pass. Native leads compiled Node on 4/4 rows and the fastest scripting runtime on 3/4. Rates describe the complete workload, including pool setup, selection, formatting and result retention.

**Completed primitive identity gate:** [Milestone 72](./milestones/72-primitive-identity.md) accepts six zero-argument primitive methods with once-only receiver evaluation, standard Cow copies/static borrowing and native bool/f64/i64 values. 645 compiler groups and 660 workspace groups pass; eight whole-workload comparisons and twenty-four fully disposed profiles preserve prior 460 sections. Native leads Node on 8/8 and the fastest scripting counterpart on 4/8. Numeric formatting and boxed objects remain separate.

# Proposed scope and roadmap

[Documentation index](./README.md) · [Open decisions](./open-decisions.md) · [Original prototype requirements](./core-and-backend-prototypes.md)

This is a proposed sequence for scoping work, not a commitment to implement every item. The original vision remains a research agenda. Research can run ahead of implementation, especially where desktop use cases expose ownership or async constraints.

**Performance work:** Use standard Rust structures and methods first, with edge tests and accurately matched benchmarks. Historical primitive tuning and UTF-16 comparisons remain archived; the user does not require general JavaScript semantic parity or every engine comparison to be a native win.

**Completed string migration:** [Milestone 33](./milestones/33-native-utf8-strings.md) replaces the earlier UTF-16 implementation with standard Rust UTF-8 strings and methods. Obsolete runtime/cache/storage code is archived outside the crates. Compiler acceptance, equivalent-operation timing, allocation profiles, and primitive memory replay pass. See [string semantics](./string-semantics.md).

**Completed numeric compound value gate:** [Milestone 68](./milestones/68-numeric-compound-assignment-values.md) accepts all twelve int and twelve Number compound operators across supported expression contexts. 629 compiler groups and 644 workspace groups pass; one wrapper-only focused repair retains three passes before the exact fourth-group retry. Twenty-four matched rows and seventy-two fully disposed profiles pass; native leads compiled Node on 24/24 rows and the fastest scripting runtime on 24/24. Earlier 410 tables, nine primitive memory tables and auxiliary baselines remain unchanged. Owned results and nullish operations remain pending.

**Completed conditional gate:** [Milestone 69](./milestones/69-conditional-expressions.md) accepts strict-boolean lazy selection across compatible existing scalar, owned and same-origin callable types, with contextual typing and conservative ownership/loan joins. One full regression passes 633 compiler and 648 workspace groups; a style-only Clippy repair retains those passes. Ten instrumented comparisons and thirty fully disposed profiles pass. Native leads Node and the fastest scripting runtime on 7/10 and 7/10 rows respectively; fresh String and String-array construction lag. Earlier 434 sections remain unchanged. Truthiness, union/nullish types and new callable representations remain pending.

**Completed string method gate:** [Milestone 70](./milestones/70-string-concat-padding.md) accepts strict zero/variadic String.concat and directional padding with native byte lengths, checked UTF-8 filler prefixes and once-only borrowed operands. Standard Cow clones preserve static no-op borrows and independent owned copies. One full regression passes 637 compiler and 652 workspace groups; eight affected editor groups verify completion, hover and signature help. Ten whole-workload comparisons and thirty disposed profiles pass. Native leads compiled Node on 5/10 rows and the fastest scripting runtime on 3/10; Unicode target mapping/setup costs are included. Earlier 444 sections remain unchanged.

**Completed substring gate:** [Milestone 71](./milestones/71-string-substring.md) accepts strict required/optional int bounds with native byte clamping and swapping, standard Cow slicing and once-only borrowed operands. Empty ranges skip boundary access, static partial results borrow and owned partial results copy independently. One full regression passes 641 compiler and 656 workspace groups with editor discovery included. Six whole-workload comparisons and eighteen disposed profiles pass; native leads compiled Node on 6/6 rows and the fastest scripting runtime on 6/6. Unicode endpoint-tuple setup/selection costs are timed. Earlier 454 sections remain unchanged.

**Current measurements:** Accepted measurements through checkpoint 96 live in [canonical SQLite analytics](../benchmarks/storage/README.md), including individual samples, min/avg/max throughput and normalized allocation probes. Top-level [BENCHMARKS.md](../BENCHMARKS.md) remains a historical export through checkpoint 86. New or changed workloads remain ready for a completed feature-family milestone; unsupported APIs are not benchmark deliverables yet.

**Previous completed slice:** [Milestone 32: primitive array search](./milestones/32-array-search.md) passes source, ownership, resource, and comparison acceptance. Earlier engine rankings belong to their recorded semantic epoch. Current performance work follows the native Rust contract and accurately matched operations.

**Completed comparison gate:** [Milestone 34: string comparisons](./milestones/34-string-comparisons.md) adds standard Rust content equality and lexical ordering, with borrowed operands and no implicit cloning. All 481 compiler groups and required Rust checks pass. Two matched cross-runtime workloads and allocation/disposal profiles complete acceptance.

**Completed uppercase gate:** [Milestone 35: uppercase](./milestones/35-string-uppercase.md) adds `toUpperCase()` through Rust's standard conversion. All 484 compiler groups and required Rust checks pass in one regression run. Both matched runtime comparisons and disposal profiles complete acceptance.

**Completed array gate:** [Milestone 36: array reversal and temporary length](./milestones/36-array-reversal.md) uses standard slice reversal and expands read-only length to temporary arrays. All 491 compiler groups, required Rust checks, three matched reversal workloads, and disposal profiles pass.

**Completed search gate:** [Milestone 37](./milestones/37-string-search.md) adds four standard Rust search methods with byte positions and borrowed inputs. All 495 compiler groups, required checks, eight matched comparisons, and allocation/disposal profiles pass.

**Completed control-flow gate:** [Milestone 38](./milestones/38-control-flow-loops.md) adds standalone blocks, classic `for`, `do...while`, and `else if`. All 502 compiler groups, required Rust checks, twelve control/primitive comparisons, and allocation/disposal profiles pass. Standalone lexical scope and cleanup have explicit compile-only acceptance.

**Completed number gate:** [Milestone 39](./milestones/39-native-number.md) integrates native `f64`, number arrays, and explicit primitive conversion. All 509 compiler groups, required Rust checks, sixteen matched comparisons, allocation profiles, and the normalized number memory probe pass.

**Completed classification gate:** [Milestone 40](./milestones/40-number-predicates.md) adds four static Number classification methods using standard Rust predicates. All 512 compiler groups, required Rust checks, four matched runtime comparisons, and allocation/disposal profiles pass.

**Completed Math gate:** [Milestone 41](./milestones/41-math-unary.md) adds five standard Rust Math transforms. All 515 compiler groups, required Rust checks, five matched comparisons, and allocation/disposal profiles pass.

**Completed slice gate:** [Milestone 42](./milestones/42-array-slice.md) adds owned array slicing across current element types and ranks. All 519 compiler groups, required Rust checks, four matched comparisons, and allocation/disposal profiles pass.

**Completed temporary indexing gate:** [Milestone 43](./milestones/43-temporary-array-indexing.md) adds temporary indexing and safe owned transfers for chained array expressions. All 523 compiler groups, required Rust checks, four matched comparisons, and allocation/disposal profiles pass.

**Completed transform gate:** [Milestone 44](./milestones/44-array-map-filter.md) adds typed map/filter callbacks across current array elements and ranks. All 527 compiler groups, required Rust checks, eight matched comparisons, and allocation/disposal profiles pass.

**Completed fixed-storage gate:** [Milestone 45](./milestones/45-fixed-number-arrays.md) uses standard Rust arrays for proved number literals and direct scalar literal indexing. All 531 compiler groups, corrected release artifact checks, 24 targeted comparisons, and allocation/disposal profiles pass.

**Completed predicate gate:** [Milestone 46](./milestones/46-array-predicates.md) passes all 535 compiler groups, required Rust checks, 24 matched workloads, and disposal profiles.

**Completed string construction gate:** [Milestone 47](./milestones/47-string-construction.md) adds repetition, replacement, and splitting. All 539 compiler groups, required Rust checks, twelve matched comparisons and disposal profiles pass.

**Completed exact Math gate:** [Milestone 48](./milestones/48-math-rounding-and-integer-functions.md) adds seven Math methods and sixteen constants. All 544 compiler groups, required Rust checks, seven matched comparisons and zero-allocation profiles pass.

**Completed approximate Math gate:** [Milestone 49](./milestones/49-native-math-functions.md) adds 22 standard Rust methods. One 549-group regression, independent numerical checks across eight runtimes, 22 matched comparisons, and zero-allocation profiles pass.

**Completed Number operator gate:** [Milestone 50](./milestones/50-number-operators.md) adds eight operators through standard Rust methods and integer operators. One 553-group regression, independent edge/effect checks, eight matched comparisons and zero-allocation profiles pass.

**Completed reduction gate:** [Milestone 51](./milestones/51-array-reduction.md) adds both directions across current data types/ranks. One 558-group regression, complete native semantic checks, 28 matched comparisons and 84 zero-final-disposal profiles pass. Native leads Node and the fastest scripting runtime on 18 of 28 rows; measured ownership gaps remain visible.

**Completed traversal gate:** [Milestone 52](./milestones/52-array-for-each.md) passes one 562-group regression, native scalar/order/disposal checks, 16 comparisons across eight runtimes and 48 zero-final-disposal profiles. Native leads Node on 15/16 and the fastest scripting runtime on 12/16; setup and owned-copy costs remain explicit.

**Completed concatenation gate:** [Milestone 53](./milestones/53-array-concat.md) passes one 566-group regression and targeted scalar-record layout repair, complete native output/order checks, 25 comparisons across eight runtimes and 75 zero-final-disposal profiles. Native leads Node on 8/25 and the fastest scripting runtime on 6/25; independent owned copying costs remain explicit. Earlier measurements were not repeated.

**Completed declared-function gate:** [Milestone 54](./milestones/54-declared-function-values.md) passes one 571-group regression, complete native output/order/IEEE checks, twelve comparisons across eight runtimes and 36 zero-final-disposal profiles. Native leads compiled Node on 9/12 rows and the fastest scripting runtime on 7/12. Earlier measurements remain unchanged.

**Completed higher-order gate:** [Milestone 55](./milestones/55-higher-order-function-values.md) passes one 577-group regression, complete native checks, 24 comparisons and 72 zero-final-disposal profiles. Native leads compiled Node on 18/24 and the fastest scripting runtime on 17/24.

**Completed range mutation gate:** [Milestone 56](./milestones/56-array-fill-copy-within.md) passes one 581-group regression, sixteen comparisons across eight runtimes and 48 zero-final-disposal profiles. Native leads compiled Node on 10/16 rows and the fastest scripting runtime on 8/16; scalar rows lead Node by roughly 8–16×.

**Completed copying gate:** [Milestone 57](./milestones/57-array-copying-updates.md) passes one 585-group regression, sixteen comparisons across eight runtimes and 48 zero-final-disposal profiles. Native leads compiled Node on 8/16 rows and the fastest scripting runtime on 6/16. Earlier 314 runtime tables, primitive memory and auxiliary baselines remain unchanged.

**Completed insertion/deletion copy gate:** [Milestone 58: array toSpliced](./milestones/58-array-to-spliced.md) passes one 589-group regression, eight comparisons across eight runtimes and 24 zero-final-disposal allocation profiles. Every complete result array is retained by normal ownership transfer. Native leads compiled Node on 3/8 rows and the fastest scripting runtime on 3/8. Earlier 330 runtime tables, primitive memory and auxiliary baselines remain unchanged.

**Completed reverse search gate:** [Milestone 59](./milestones/59-array-last-index-of.md) passes one 593-group regression, eight default/bounded comparisons across eight runtimes and 24 zero-final-disposal allocation profiles. Every returned index is retained through lossless index + 1 encoding. Native leads compiled Node on 8/8 rows and the fastest scripting runtime on 8/8. Earlier 338 runtime tables, primitive memory and auxiliary baselines remain unchanged.

**Completed string reverse-search gate:** [Milestone 60](./milestones/60-string-last-index-of.md) passes one 597-group regression, four default/bounded ASCII/Unicode comparisons across eight runtimes and twelve zero-final-disposal allocation profiles. Every byte index is retained through lossless index + 1 encoding; Unicode coordinate translation costs remain timed. Native leads compiled Node on 4/4 rows and the fastest scripting runtime on 3/4. Earlier 346 runtime tables, primitive memory and auxiliary baselines remain unchanged.

**Completed directional string-trimming gate:** [Milestone 61](./milestones/61-string-trim-sides.md) passes one 601-group regression, eight ASCII/Unicode comparisons across eight runtimes and twenty-four fully disposed allocation profiles. Each method and alias has separate coverage. Receiver setup, selection, trimming and complete string-array retention are timed. Native leads compiled Node on 0/8 rows and the fastest scripting runtime on 0/8. Earlier 350 runtime tables, primitive memory and auxiliary baselines remain unchanged.

**Completed string-addition gate:** [Milestone 62](./milestones/62-string-addition.md) passes one 605-group regression, four binary/chained ASCII/Unicode comparisons across eight runtimes and twelve fully disposed resource profiles. Strict String/String addition borrows operands and constructs independent owned results, including chain intermediates. Native leads compiled Node on 0/4 rows and the fastest scripting runtime on 0/4. Earlier 358 runtime tables and auxiliary baselines remain unchanged.

**Completed primitive compound-assignment gate:** [Milestone 63](./milestones/63-compound-assignment.md) accepts 609 distinct regression groups from one full invocation with retained passes and targeted continuations. Five checked int, twelve Number and String/String compound forms have nineteen matched rows across eight runtimes and fifty-seven fully disposed resource profiles. Targets and old values evaluate in source order under existing loans and strict types. Native leads compiled Node on 17/19 rows and the fastest scripting runtime on 16/19. Earlier 362 runtime tables and auxiliary baselines remain unchanged.

**Completed numeric update gate:** [Milestone 64](./milestones/64-update-expressions.md) accepts all eight int/Number prefix/postfix forms, 613 distinct Rust groups with retained/resumed acceptance, eight matched rows across eight runtimes and 24 fully disposed profiles. Targets evaluate once and preserve old/new results, existing loans and checked/IEEE policies. Native leads compiled Node and the fastest scripting runtime on 8/8 rows. Earlier 381 tables, nine primitive memory tables and auxiliary baselines remain unchanged.

**Completed native integer gate:** [Milestone 65](./milestones/65-native-int-operators.md) accepts sixteen typed i64 forms and 617 distinct Rust groups from one full invocation with retained passes and targeted resumption. Sixteen matched rows across eight runtimes and 48 fully disposed profiles pass. Native leads compiled Node and the fastest scripting runtime on 16/16 rows. Exact common-domain comparisons preserve native 64-bit semantics and separate wide/trap evidence; earlier 389 tables and auxiliary baselines remain unchanged.

**Completed boolean logical assignment gate:** [Milestone 66](./milestones/66-boolean-logical-assignment.md) accepts strict-boolean `&&=` and `||=` in supported expression contexts, statements and for clauses. All 621 distinct Rust groups pass with retained continuation after the diagnosed parser frame repair. Two matched rows across eight runtimes and six fully disposed profiles pass; native leads compiled Node on 2/2 rows and the fastest scripting runtime on 2/2. Earlier 405 tables and auxiliary baselines remain unchanged. Nullable assignment, other assignment-valued expressions and coercions remain pending.

**Preceding slice:** [Milestone 31: record field moves](./milestones/31-record-field-moves.md) is complete. Direct named string/record/array field paths transfer without cloning while siblings remain available. Field or parent replacement restores whole-record use; joins and loop edges track missing storage. Whole-root parameter/capture contracts, detached argument values, indexed/iteration borrow restrictions, conservative source loans, and native bounds failure order remain enforced. All 404 regression groups pass. Earlier [array removal](./milestones/30-array-removal.md), [recursive callable results](./milestones/29-recursive-callable-results.md), and preceding ownership gates remain complete. Mixed/unresolvable callable origins, recursive inline environment storage, disjoint field loan inference, partial environment captures, mutable/consuming iteration, optional/range removal, mixed callable environment types, callable fields, shared capture cells, async, modules, broader core-proof measurements, and the future Rust language server remain separately scoped work.

**Completed scalar assignment value gate:** [Milestone 67](./milestones/67-scalar-assignment-values.md) accepts ordinary `=` values for int, Number and boolean across supported expression contexts, statements and for clauses. All 625 distinct compiler groups and 640 workspace groups pass with retained continuation after 4 stale scalar bounds-order fixture expectations were corrected and 1 indexed-record analyzer repair restored RHS growth across 4 encountered regression failures. Three matched rows across eight runtimes and nine fully disposed profiles pass; native leads compiled Node on 3/3 rows and the fastest scripting runtime on 3/3. Earlier 407 tables and auxiliary baselines remain unchanged. Owned ordinary and compound assignment results remain pending.

**Gate policy:** Prioritize implementation with focused TDD. Finish compiler behavior, LSP integration, diagnostics, verified IntelliSense examples and benchmark readiness as each feature lands. Register matched counterparts, independent oracles, current fingerprints and guards immediately. Do not launch heavy regressions or measurements by gate count. SQLite remains canonical; preserve original evidence and treat implementation acceptance separately from deferred measurements.

**Comprehensive coverage:** The [feature and comparison inventory](./feature-coverage.md) tracks all selected pinned TypeScript standard-library declarations and explicit language constructs, including primitive operations, loops, iterators, generators, decorators, and switch. Implementation and comparison status are independent; missing isolated comparisons stay visible even when a compound workload exercises the feature. Add related methods in coherent gates and update the inventory as they land.

**Regression cadence:** Complete a meaningful feature-family scope across compiler, LSP and benchmark support before heavy checks. Then run complete regressions and matched comparisons, review comparable signals, and optimize. For example, complete the scoped Node built-in array methods before the array milestone run. Preserve completed checks; repair/rerun affected checks and resume only unrun groups. The former five-gate schedule is superseded.

## Groundwork: establish the design questions

**Deliverables:** A product thesis, an agreed decision priority, a minimum language subset, a research register, and independent benchmark specifications. Begin competitive research and identity research; do not finalize a name before the required positioning and feasibility work.

**Workstreams:** [Product vision](./product-vision.md), [principles and boundaries](./principles-and-boundaries.md), [research program](./research-program.md), [competitive landscape](./competitive-landscape.md), [identity](./identity.md), [benchmarks](./benchmarks.md), and [security](./security-and-supply-chain.md).

**Exit evidence:** Identify which assumptions need experiments, which decisions block the first prototype, and which remain open. Record provisional choices as reversible experiment constraints. Preserve the original research-package requirement before full language implementation.

**Deferred implementation:** Registry, automatic npm conversion, desktop renderer, broad library coverage, and final branding.

## Core proof: ownership and native compilation

**Parallel editor track:** [VS Code foundation E1–E4](./editor-foundation.md) is implemented against the current in-process compiler API; [acceptance evidence](./editor-foundation-acceptance.md) covers the installed local extension. E5 adds source-assisted documentation hover, signature help, completion, navigation, icons and snippets without claiming C4 compiler-proven metadata. Compiler progression must preserve C0 and record editor impact in each gate. Recovery (C1), diagnostic aggregation (C2), cooperative cancellation (C3), source-indexed metadata (C4), and module/project analysis (C5) are separately scoped handoffs; they do not block the bounded first editor release. See the plan for contracts, sequencing and acceptance evidence.

**Deliverables:** The original `User`/`normalize` example compiles to a native executable. A second example proves inferred borrowing, a mutation example proves exclusive access, assignment transfers ownership, use after transfer fails, and explicit cloning permits an independent value.

**Workstreams:** [Ownership and memory](./ownership-and-memory.md), [types](./type-system.md), [primitives and collections](./primitives-and-collections.md), [errors](./error-handling.md), [compiler and modules](./compiler-and-modules.md), [standard library](./standard-library.md), [toolchain](./toolchain-and-development.md), and [prototype requirements](./core-and-backend-prototypes.md).

**Exit evidence:** Accepted and rejected source programs establish observable semantics. Diagnostics identify original source locations and explain transfers/access conflicts. Generated Rust compiles. Record compile time and allocation/clone behavior for the supported subset.

**Design gates:** Decide parameter access contracts, assignment behavior, mutation permission, numeric/string basics, and the minimum error model. Test escaping closures before claiming that inference supports application callbacks generally.

**Deferred implementation:** Broad generics, macros, inheritance, automatic collection fusion, HTTP, JSX, and desktop controls unless a concrete core experiment requires them.

## Backend proof: async, typed data, and HTTP

**Deliverables:** A native `/hello` endpoint, followed by a scoped typed-JSON/validation endpoint and middleware example. Use a restricted Rust wrapper boundary for required native dependencies. Add a database integration as a distinct follow-up slice.

**Workstreams:** [Async/runtime](./async-and-runtime.md), [interop](./interop-and-targets.md), [serialization](./metadata-and-serialization.md), [HTTP/Hono](./http-and-hono.md), [validation](./validation.md), [databases](./databases.md), [deployment](./deployment.md), [migration](./source-migration.md), and [backend ecosystem](./backend-ecosystem.md).

**Exit evidence:** Typed parsing rejects invalid input. Original compatibility tests cover the implemented routing and middleware subset. Tests exercise async captures, shared state, cancellation, and cleanup. Measure startup, binary size, memory, latency, throughput, and compile time with documented workloads.

**Design gates:** Choose a compatibility level and error/cancellation contract. Specify ownership of request context and captured application state. Document which interop APIs are supported.

**Deferred implementation:** Full Hono source compatibility, all database engines, complete Web-standard APIs, automatic npm conversion, and advanced network protocols.

## Web proof: typed server HTML

**Deliverables:** A separate `Html` type, JSX component composition, dynamic-value escaping, and a native response writer. Scope streaming and async components as later increments within this phase.

**Workstreams:** [Server HTML](./server-html.md), [HTTP/Hono](./http-and-hono.md), [migration](./source-migration.md), and [benchmarks](./benchmarks.md).

**Exit evidence:** Original rendering tests cover nested components, dynamic text, attributes, and explicitly trusted markup. Confirm source diagnostics and observable output for the selected syntax. Measure the original HTML workload.

**Deferred implementation:** Browser reconciliation, hydration, client-side React compatibility, and a shared desktop renderer abstraction.

## Ecosystem proof: packages and migration inspection

**Deliverables:** A content-addressed package-store prototype shared by two projects, dependency identity and lockfile rules, and an inspection-only npm analyzer. Grow native packages only when a demonstrated workload requires them.

**Workstreams:** [Packages/registry](./packages-and-registry.md), [npm analysis/porting](./npm-analysis-and-porting.md), [security](./security-and-supply-chain.md), and [backend ecosystem](./backend-ecosystem.md).

**Exit evidence:** Demonstrate shared physical storage, reproducible dependency resolution, offline behavior, and integrity checks. Analyzer reports point to unsupported constructs and state their analysis limits. A successful inspection must not imply proven semantic equivalence.

**Dependencies:** Stable enough module/type identity and a usable CLI. Package research can begin during groundwork; a hosted registry is not required to prove the compiler.

**Deferred implementation:** Hosted registry operations, automatic translation, upstream merge automation, and compatibility percentages without a defined measurement method.

## Desktop proof: native components and deterministic cleanup

**Deliverables:** Research rendering and accessibility early. Implement the original counter only after the ownership, capture, async, and reactive models support it. Then scope the user manager and shared server/desktop example as separate milestones.

**Workstreams:** [Architecture](./desktop-architecture.md), [reactivity/ownership](./desktop-reactivity.md), [controls/accessibility](./desktop-controls.md), [async/platform integration](./desktop-async-and-platform.md), [tooling/benchmarks](./desktop-tooling-and-benchmarks.md), and [prototypes/shared code](./desktop-prototypes-and-shared-code.md).

**Exit evidence:** The counter opens a real OS window, updates the affected text binding, uses no browser engine or tracing GC, and releases associated resources on close. Inspection identifies the binding updated. Later examples prove dynamic lists, component cancellation, input, validation, and shared domain code.

**Design gates:** Choose a first platform and renderer experiment. Define signal assignment, event capture ownership, conditional/list lifetimes, accessibility requirements, and teardown behavior. Distinguish reactive binding updates from necessary layout or paint work.

**Deferred implementation:** Simultaneous platform parity, a custom GPU renderer, broad OS integrations, advanced packaging, state-preserving hot reload, and terminal UI.

## Work-item template

Use this checklist when converting a workstream into an implementation task:

- **Outcome:** One observable capability or a research conclusion.
- **Scope:** The exact source constructs, APIs, platforms, or workloads included.
- **Prerequisites:** Decisions and capabilities required first.
- **Acceptance evidence:** Programs, diagnostics, measurements, or a source-backed decision record.
- **Deferrals:** Related features explicitly outside this task.
- **Open questions:** Unresolved questions that could change the approach.

Keep the original section IDs in task references so later scope changes remain traceable.

Captured rest function values have a focused gate173 checkpoint. Generic values and the full signature/context audit remain pending; family measurements are still deferred.

Gate173 now has contextual generic function values with checked type relations, captured callbacks and explicitly returned fixed factories. Unannotated polymorphic values and the full inference/context audit remain open.

Gate173 now isolates concrete generic value signature solving before environment
construction. Default forwarding removes only undefined from generic inference,
preserving null checks. Deferred polymorphic values and the full audit remain open.

Gate173 now has capture-free deferred generic origins with checked concrete
methods across data types. Ownership/default/rest/failure witnesses and matched
workload support are ready. Captured environments, complete generic editor help
and full signature/context acceptance remain open.

Gate173 now retains captured deferred generic environments across data types and
independent inferred factory instances. Five original red witnesses pass; O0/O3
cleanup, ownership rejection and concrete editor facts have focused controls.
Matched captured-environment support is ready for untimed verification. Full alias
help, result-only contexts, callback/storage adaptation and the wider audit remain
open. Family measurement remains deferred.

Gate173 now propagates expected result types through generic aliases and keeps
zero-argument methods distinct by concrete result. Captured environments remain
shared. Full alias help, callback/storage adaptation and the wider audit remain
open; no family measurement is claimed.

Generic alias editor progress: local hover and completion retain generic source
contracts and documentation; concrete invocation help retains default/rest
contracts. Cross-module alias metadata and polymorphic storage/callback adaptation
remain open under gate173. See handoff-2026-10-06-gate173-local-alias-help.md.

Project generic alias progress: imported aliases and captured factory results
retain generic signatures and docs. Concrete invocation help includes checked
ownership and imported nominal aliases. The wider editor audit and real callable
storage/callback adaptation remain open.

Generic alias header progress: fixed nominal annotations now use source identities
for imported aliases, applied records and private results. Labels and default
literals stay intact. Conditional/optional aliases and callable adapters remain
open under gate173.

Existing generic value adapters now support explicit contracts, array transform
callbacks and typed record/array storage with ownership/disposal controls. The
full adapter audit and benchmark family remain open; see generic-value-adapters-plan.md.


Generic factory parameter help now consumes additive partial call facts when the
returned generic environment has no concrete display contract. Checked seed types,
parameter access and original declaration locations remain available. An unknown
result adds no result annotation; it never becomes void or an empty callable.
Conflicting specializations at one source span remain distinct and suppress
ambiguous editor overrides. Source and project projection tests cover string/int
variants and imported aliases with Unicode source offsets. Full Gate173 acceptance
and the remaining adapter contexts are still open.


Selected fresh factory calls now adapt their branches before joining callable
origins. The predicate runs first; only the selected factory and its arguments run.
Each branch must satisfy the concrete signature, and an owned storage family keeps
the selected environment. Traversal accepts this storage through the existing
callback tuple and family error contract, including map, search, reduction,
flatMap and sorting. Immediate staging borrows end after factory evaluation;
returned owned environments still cannot retain borrowed factory parameters.
Nested selected callees and heterogeneous inferred result joins remain unaudited.
Full adapter and Gate173 acceptance remain open.


Nested selected factory callees now lower through an iterative branch walk. Each
leaf must resolve to a deferred generic factory; the original expression remains
unchanged if that requirement fails. Predicate order and short-circuit selection
remain intact, and each chosen invocation evaluates its arguments once. Traversal
solves its concrete context from a factory leaf and checks every branch against
that contract. No recursive lowering frame or unchecked origin join is introduced.
The nested-selected-factories fixture observes four predicate evaluations, two
argument evaluations and independent selected environments. Heterogeneous result
joins and the remaining storage, inference and ownership context audit stay open.


Fresh record fields and typed callable arrays pass native acceptance directly,
including selected factory expressions and independent retained counters. No
compiler repair was needed for those aggregate destinations. Fresh traversal of a
generic function with an omitted result annotation did fail at its unresolved
prototype. Result discovery now checks the original generic body using supplied
type hints and its retained capture context before constructing the concrete
adapter. It does not execute arguments, defaults or factory environments. Explicit
result mismatches remain type errors. Nested callable results, parameter patterns,
nullable/access contexts and heterogeneous result joins still require their audit.

## Nested callable reconstruction checkpoint

Fresh generic factory values now preserve nested callable input and result
contracts. Checked reconstruction retains invocation ownership, names, access,
omission and rest metadata; SharedRead remains shared through internal source
annotations and project facts. Exact round trips pass for all six invocation kinds.
Void callback results and optional/default/rest inputs execute correctly. Borrowed
callbacks remain usable by their owners. Single-use results cannot satisfy reusable
contracts. Concrete nested signature hints constrain generic inference without
executing arguments. Deferred invocation checks any further body specialization
selected by callable arguments before retaining its method.

All 34 adapter tests, four shared runtime tests and eight depth canary suites pass;
whole editor passes 305 tests. Matched native/Node-after-tsc verification passes five
cases with 88 observations. Canonical SQLite record:
benchmarks/runtime/results/2026-10-07T01-20-05-313Z-verify.json.
Original failed witnesses, repaired checks and metadata round-trip evidence are
retained exactly in SQLite. Five executable consumers were refreshed atomically
and hash verified. No timing was run. This accepts the nested callable contract
chunk; nullable contexts, heterogeneous joins and the wider Gate173 audit remain
open. No full Gate173 acceptance is claimed.

## Nullable contexts and heterogeneous factory results

Fresh generic factory values now acquire nullable owned callable contracts.
Present optional calls retain factory state; absent calls skip argument evaluation.
Selected factories returning T and T[] acquire an int | int[] result contract
without changing the inferred input payload. Argument constraints establish the
payload before contextual result compatibility is checked. Adapter result views
validate the materialized source result first, including invocation ownership.
A single-use callable result still cannot satisfy a reusable result contract.
Generic nullable constraints accept an existing payload member without inventing
a union for incompatible inputs.

All 37 adapter tests, the repaired bounded generic/default controls, two result-only
signature controls, eight depth suites and 307 editor tests pass. Both strict
TypeScript reference fixtures match native execution. Matched coverage appends five
observations for nullable presence/absence and heterogeneous results, preserving
previous observations. All five native/Node-after-tsc cases pass with 93 observations.
Fallible workload ABI registration accepts the checked Result signature and rejects
unknown error signatures. Canonical SQLite record:
benchmarks/runtime/results/2026-10-07T01-30-18-635Z-verify.json.
Original failures and final checks are stored with byte-exact readback. Five
consumers were refreshed atomically and hash verified. No timing was run. This
accepts the nullable/heterogeneous context chunk; full Gate173 and its remaining
ownership/context audit stay open.

## Generic callable payloads

Focused callable payload support is accepted separately from the remaining Gate173
context audit. Generic T can retain a complete callable contract across argument
inference, source specialization and returned values. Keys preserve invocation
ownership, argument access, omission and rest metadata. Equivalent bare/owned and
optional/defaulted contracts normalize without including parameter spelling or
source spans. Distinct invocation and required-position contracts remain distinct.
Checked borrowed closures retain their borrowed invocation kind.

Native witnesses retain one factory environment across int and string payload
signatures. Readonly and mutable borrowed payloads leave the owner's callable
available. Once payloads remain single-use. Argument access mismatches and repeated
once calls remain errors. Success and factory failure dispose exactly four original
captured records, with identity mask 30, at O0 and O3. No implicit cloning is added.
Editor hover, signature help and original factory navigation retain concrete
returned callable contracts.

Verification passes 45 adapter tests, 72 bounded generic/default/value controls in
total, all 63 depth canaries and all 308 editor tests. Matched support adds three
observations, for 96 per scenario, with guards for int/string payload results and
retained factory state across a third specialization. All five native/Node-after-tsc
cases pass untimed. Exact record
`benchmarks/runtime/results/2026-10-07T01-48-46-795Z-verify.json` is stored in SQLite.
Registration and metadata controls pass. Mutable, readonly and once reference
fixtures retain strict TypeScript counterparts; callable invocation references use
a TypeScript generic constraint where required by that checker.

No timing or complete compiler regression is claimed. The wider generic context
and alias/editor audit remains open, and full Gate173 acceptance is still pending.

## Explicit applications on retained generic values

Retained named generic values now accept explicit type applications, including
callable payloads. Explicit arguments are kept by enclosing function identity and
call span, so distinct enclosing specializations substitute their own type bindings.
Checked preparation resolves the original generic declaration and captured context,
materializes the explicit signature, and then uses ordinary argument, result and
ownership checking. It does not replace supplied arguments with inferred types or
reconstruct a factory environment. Nongeneric values, wrong type-argument counts,
conflicting arguments and reused owners remain errors.

Explicit and inferred invocations retain one captured environment across scalar,
string and callable payload types. Enclosing generic substitutions, result-only
arguments, defaults and rest packs have focused controls. Defaults execute once at
invocation; rest packs retain their normal ownership. Checked method identity also
includes generic application arguments, keeping unused explicit type arguments
separate while reusing equivalent applications. Existing cache controls pass.
Editor signature help and result hover remain concrete, and invocation navigation
points to the local alias declaration.

The original committed red witness now passes. Fifty adapter tests plus the added
consuming-capture control cover 51 unique adapter tests. Seventeen generic value
controls and all 63 final depth canaries pass. The whole editor crate is verified by
preserving passed suites, repairing an accidental existing-test assertion edit,
and resuming the unrun suites. The expanded matched workload contains 101
observations per scenario; all five native/Node-after-tsc cases pass untimed.
Exact record benchmarks/runtime/results/2026-10-07T01-56-31-516Z-verify.json and original
failed/repaired logs are retained byte-for-byte in SQLite. No timing or complete
compiler regression is claimed. Full Gate173 and the broader context audit remain
open, including field/optional generic applications and remaining parameter-pattern
and borrowed-result contexts.

## Optional and selected generic invocation

Focused optional/selected generic invocation now preserves the original deferred
environment instead of reading its empty prototype. Named receiver access and
selected-expression access share specialization and argument checking, while
selected receivers retain real ownership roots and selected_access checks. A fresh
receiver already checked during construction is not checked again; an owned factory
argument is transferred once. Conditional invocation forwards expected result context.

Nullable factory alternatives prepare original omitted results before conditional
hint joining, without executing runtime arguments or factories. Optional result-only
context remains complete until the optional checker gives payload context to its
present branch. Real void-union rejection remains intact. Dynamic absent calls skip
argument effects, and present calls retain captured state. Failures propagate without
rebuilding that state. Fresh consuming environments work; named optional selections
still cannot consume a borrowed owner. Type, arity and receiver-loan failures remain
checked even for absent invocation branches.

Editor initializer inference now prefers the complete checked expression type over
selected-method return metadata at the same span. Optional results retain int |
undefined, while signature help reports the concrete invoked method. The original
red witnesses and dynamic strict TypeScript counterpart pass. All 58 adapters pass
in the batch; one additional consuming-capture control passes separately, covering
59 unique tests. Seventeen generic-value controls, two signature controls and all
63 depth canaries pass. Whole editor passes 310 tests. Native O0/O3 disposal observes
three original records exactly once, with identity mask 14, across success/failure.

The matched pair observes 106 values per scenario, retaining the prior 101. Five
native/Node-after-tsc cases pass untimed with independent oracle, fingerprints and
semantic guards. Exact record
benchmarks/runtime/results/2026-10-07T02-10-49-964Z-verify.json is stored in SQLite.
No timing or complete compiler regression is claimed. Full Gate173 remains open;
field applications, parameter patterns, nested callable defaults/rest, alias contexts
and unused generic source-validation controls still require the wider audit.


The Gate173 pattern audit adds owned nested array/record parameter extraction,
concrete adapter editor contracts and discarded callable-call disposal controls.
Matched support now observes 116 results per scenario and passes untimed native/
Node verification. Broader Gate173 acceptance and milestone measurements remain open.


Gate173 adds source cardinality checks for unused immutable generic aliases and
known callback contracts. Mutable/selected target flow and the broader generic
context audit remain open. Compile-time acceptance is separate from runtime timing.


Gate173 source validation now preserves closure callback arity and known argument
bounds through nested dynamic spreads. Mutable/selected callable flow remains open.

Gate173 source validation now retains common mutable/selected contracts and
straight-line callable snapshots across definite assignments. Structured branch/loop
flow, stored fields and concrete environment replacement remain open. See
handoff-2026-10-06-mutable-source-contracts-in-progress.md.

Gate173 source flow now preserves direct replacement through blocks and branches,
including snapshots, branch-local calls and abrupt completion. Distinct contract
sets, loop/exception/expression flow and captured effects remain required. See
unused-generic-call-validation-plan.md.


The source-loop checkpoint and automatic project formatting are documented in
milestones/project-formatting-and-source-loops.md. Finite callable contract sets
now propagate through loops, updates and labeled control. Full Gate173 still needs
exception/finally flow, expression evaluation order, capture writes, stored fields,
concrete environment replacement and the remaining lifetime/context audit. Project
formatting is required before every source checkpoint; exact benchmark source hashes
refresh only after token preservation proof. No performance measurement is claimed.


Gate173 structured source flow now covers try/catch/finally and their abrupt
completions, with native cleanup-order and editor diagnostic verification. See
handoff-2026-10-07-exception-source-flow.md. Expression ordering, capture effects,
stored fields and concrete environment replacement remain required. No timing or
full Gate173 acceptance is claimed.


Gate173 expression source flow and named assignment invocation have focused
acceptance, including checked parameter help, native failure/consumption/disposal
and 120-observation matched adapter verification. See
handoff-2026-10-07-expression-order-and-assignment-invocation.md. Captured effects,
optional expression regions, stored fields and concrete environment transport
remain required; full Gate173 and performance measurements remain open.

Gate173 lexical generic transport now preserves enclosing type bindings independently
of nested explicit arity, including linked modules, recursive peers and escaped
function values. Scoped native and disposal controls pass. Matched support appends
four observations for 124 per scenario without changing operation or sample counts.
Captured invocation effects retain eight red witnesses; the remaining Gate173 audit
and milestone measurements stay open.


Gate173 now includes inferred common carriers for distinct concrete owned
conditional callable alternatives. Native ownership/disposal and editor help pass,
and the matched generic adapter scenario retains 129 values with five untimed
native/Node-after-tsc cases verified. Borrowed carriers, polymorphic replacement
and checked mutable source environments remain open. No new timing or complete
gate173 acceptance is claimed. See conditional-callable-carriers-plan.md.

Gate173 now has scoped monomorphic borrowed conditional carriers alongside owned
carriers. Local capture lifetimes, readonly/mutable callback loans, defaults and
callback failures survive selection. Scope escape and ownership transfer guards
remain enforced. Matched workload support retains 131 observations; timing remains
pending. Polymorphic replacement, consuming borrowed sums and point-sensitive
mutable environment restoration remain open.

Consuming borrowed conditional carriers and inferred scoped Once parameters now
have focused acceptance. Concrete non-retaining functions accept borrowed consuming
environments; retainers and unknown boundaries preserve owned environments. The
matched adapter scenario has 133 observations with untimed native/compiled-TypeScript
verification. Existing Once syntax remains sufficient. Scoped aggregate/builtin
transport, polymorphic replacement and mutable environment restoration remain open.
See milestones/173-scoped-once-parameters.md.

Scoped monomorphic families now cross reusable collection traversals and nullable
readonly/mutable callback boundaries. Optional invocation, typeof, failures and
escape diagnostics have focused acceptance. Matched adapters retain 136 observations
with untimed native/compiled-TypeScript verification. General scoped aggregate storage,
distinct-family coalescing, polymorphic replacement and mutable restoration remain open.
See milestones/173-scoped-callable-transport.md.

## Benchmark layout contracts

Gate 174 repairs the benchmark harness after project formatting. Token-based source
guards and mutations retain semantic checks. Manifest metadata remains pinned
independently of source fingerprints, and historical numerical policies retain
immutable bytes with explicitly proven formatting identities. Representative native
and Node-after-tsc verification is untimed. The named-declaration family milestone
and callable replacement witnesses remain separate work. See
[milestone 174](milestones/174-benchmark-layout-contracts.md).


## Generic callable replacement milestone

Finite replacement carriers and point-specific checked invocation now preserve saved values, different declarations and arities, defaults/rest, capture access and selected failures. Owned mutable factories retain independent capture state across repeated calls, loop-created array storage, reinsertion and nested child capture graphs. Recursive array analysis widens request inputs to keep analysis finite. Existing borrowing and explicit-copy rules remain unchanged.

The family regression milestone covers 2,325 compiler passes after affected publication repairs with zero failures and two intentional subprocess-worker ignores, plus 358 editor passes. Original interrupted recursion evidence and repaired checks remain in SQLite. The nineteen-observation replacement workload has 186 execution samples and a reviewed native median advantage of approximately 10.02 on this machine. All five consumers rebuilt from ea6bda3 pass combined CLI/native/checked-editor smoke. Imported assertions and stored native aliases preserve point-sensitive contracts. Broader scoped aggregate composition remains separate work. See milestones/callable-replacement-family.md.
