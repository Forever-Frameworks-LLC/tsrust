# Callable ownership by default

User approved this language change: bare callable types own their environments; `read` and `mutate` describe borrowing, and `once` describes an owned environment consumed by invocation. This migration is part of S06 and does not replace the standard-library roadmap.

## Contract

- `() => int`: ownership transfers on passing/assignment; repeated calls temporarily borrow the environment mutably. Moving remains explicit through ordinary assignment, not a keyword.
- `once () => int`: ownership transfers and invocation consumes the environment.
- `mutate () => int`: scoped mutable borrowing; captured state may change, but the borrowed environment cannot escape or be consumed.
- `read () => int`: scoped readonly borrowing; calling must not mutate or consume captured state.
- Existing `owned` and `owned once` spellings remain accepted aliases during migration. Public examples and help use the new defaults.
- Callable environments remain noncloneable. Ordinary data ownership, `clone()`, nominal identity and exact callable argument contracts remain unchanged.
- Returned callable values already own escaping environments. Preserve inference of reusable versus consuming invocation for unqualified factory results; an explicit `once` result must agree with the body.

## Milestone gates

1. Explicit borrowing representation and syntax. Keep old defaults until both borrowing forms have exact argument/invocation modes. Cover grouped annotations, nested callback parameters, generic signatures and source spans. Verify readonly calls reject mutations while mutable borrowing permits them. Stored borrowed fields remain rejected.
2. Default switch. Change both declaration parser and structured annotations together. Bare callable parameters and fields use owned contracts; `once` uses owned-once. Preserve factory-result inference and old aliases. Execute the user's counter/countdown program without `owned` and test transfers, repeated calls and consuming calls.
3. Migrate existing borrowed APIs and fixtures. Annotate intentional scoped callback parameters explicitly. Preserve borrowed callbacks in array methods and recursion; do not silence ownership failures by converting every callback to owning. Verify native optional function adapters, generics, callable fields and imported private types.
4. Editor and documentation parity. Syntax highlighting, hover, completion, parameter help, formatting, quick fixes and project metadata share the new meaning and display spellings. Update built-in signatures only where they describe borrowed/owning callbacks; preserve complete behavioral docs and execute affected examples.
5. Acceptance and checkpoint evidence. Run focused affected checks during each increment. Register semantically matched workloads for new default callable paths; preserve old run records and queue full regressions/performance for the authoritative five-gate checkpoint. Track correctness separately from pending measurements.

## Compiler integration points

`type_syntax.rs` owns structured callable annotation parsing. `syntax.rs` also parses parameter and result callbacks and converts between annotation forms. `callable_annotations.rs` bridges contracts; `callable_resolution.rs`, `callable_contracts.rs` and `callable_compatibility.rs` resolve and validate ownership/access. `analysis.rs` establishes parameter invocation modes, borrowing, transfers and escaping captures. `projections.rs` and `project_facts.rs` preserve type labels and module identity. Native emission must distinguish readonly and mutable reference arguments. LSP parsing/display and the VS Code grammar must agree with compiler syntax.

## Progress

Planning and source inspection complete. Existing bare parameter callbacks currently mean scoped mutable borrowing; bare returned callable values use factory inference. These distinct paths must not be changed by a blind global keyword replacement. Implementation and acceptance remain in progress.

### Increment 1a: explicit mutable borrowing

Both structured and declaration parameter parsers accept `mutate (...) => ...` as the existing reusable scoped borrowed contract. Grouped forms preserve the same contract; source spans include the modifier. Public native tests verify captured-state mutation, caller reuse after borrowed calls and rejection of return/record escape. LSP recognizes the modifier in callable annotations, preserving hover, parameter labels and chained return-type help. Existing highlighting already includes `mutate`.

Evidence: callbacks (16), type_syntax (7 existing plus the new explicit-span case), and the focused explicit-mutate IntelliSense case pass. No native runtime behavior changes in this syntax increment, so it introduces no separately measurable workload. Readonly callable borrowing, the bare-type default switch, fixture migration and canonical display changes are still pending. Bare callable parameters still borrow in this intermediate state.

### Increment 1b: readonly borrowing

`read (...) => ...` now has a distinct callable contract and invocation/access mode. Both declaration and structured parsers retain it, including nested callback parameters; nested modifiers belong to the callback rather than being mistaken for data-parameter access. Readonly callbacks cannot mutate or consume captured environments, cannot escape, and are not valid stored owned fields. Native code uses `ReadCallback::invoke_read(&self, ...)`, implemented only for readonly environments, with shared parameter references. Mutable/once native traits remain unchanged.

Focused source/native evidence: callbacks (18), callable_fields (8), type_syntax (8) pass; new readonly tests cover state preservation, mutable/consuming rejection, forwarding, nested callback parameters and same-parameter conditional invocation. The focused readonly LSP case passes, and production compiler/LSP Clippy passes with warnings denied. A condition choosing between separately declared callback parameters still hits the existing declaration-origin join restriction; this was observed and remains pending, not accepted as supported. Ownership defaults have not switched yet. The eventual new callable default workload remains required before migration gate closure and checkpoint timing remains separate.

### Increment 2: default switch and first migration batch

Bare parameter and stored-field signatures now use owned callable contracts; `once` uses owned-once. Structured syntax distinguishes the unqualified `Default` spelling from explicit borrowing so unqualified factory return annotations still infer invocation rather than becoming callback parameters accidentally. Legacy `owned` aliases remain accepted. Canonical projected callback types and checked generic editor facts show default ownership without an `owned` prefix and borrowed reusable contracts with `mutate`.

Native public tests verify the user's countdown transfer path prints 10 without `owned`, subsequent caller use is rejected, explicit readonly borrowing preserves reuse, and bare reusable/once fields execute. The existing callback test family was migrated to explicit mutable borrowing to preserve its original loan/lifetime contracts. Once-local examples now explicitly clone data when they need the original after an owned capture. The mixed recursive borrowed-vector tests likewise keep borrowed specialization limits. The std:path executable documentation example explicitly borrows its reusable adapter; generic editor expectations use the new canonical spelling.

Passed focused suites across repaired runs: callable_ownership_defaults (2), callable_fields (8), callbacks (18), callable_locals (6), recursive_owned_callbacks (15), type_syntax (8), intelligence (63), project_intelligence (28). Production compiler/LSP Clippy passes; development binaries rebuilt. Original failed and repair logs are retained under /tmp/tsrust-owned-default-*.log for this working increment.

The migration gate remains open. Remaining compiler/internal/project fixtures, documented borrowing examples, explicit consuming factory result contracts, and matched ownership-default benchmark admission require further work. These focused results do not claim full regression acceptance or measured performance. Full regressions remain scheduled at the authoritative checkpoint, preserving existing runs and passing evidence.

### Increment 3a: internal/project and borrowing-suite migration

Migrated internal borrowed-field admission tests to `read`/`mutate`, and projected type expectations to canonical ownership-default spelling. Native optional adapters used repeatedly now explicitly borrow at their callback boundary. Project factory metadata uses `Default` for inferred returned callables while retaining nominal identity. All compiler internal tests (72) and project tests (72) pass across repaired runs.

Callback move, recursive borrowing and escape tests now declare scoped borrowing explicitly. Tests formerly treating bare `once` as borrowed now reflect owned-once semantics: factory return/capture is allowed, caller reuse is rejected, and preserving original captured data requires an explicit copy. Scoped borrowed-capture lifetime checks remain under `mutate`. Passed focused suites: callback_moves (10), callback_returns (10), recursive_callbacks (14), retained_callbacks (10), single_use_callbacks (11). The local callable guide's table, examples and lifetime explanation now match these contracts; its typed counter fixture uses default ownership and is verified in debug and optimized native execution.

Preserve original and repair logs under /tmp/tsrust-owned-default-*.log. No full checkpoint regression was restarted. The gate remains open for additional affected fixtures, explicit factory-result contracts, owned-parameter reborrowing ergonomics, remaining documentation and runtime-workload migration/admission. Existing owned callable workloads cover the same native ownership model; changing a spelling alone is not evidence of a runtime speed change. Newly changed workload semantics still need matched correctness and checkpoint registration before closure.

### Increment 3b: scoped reborrowing of owned parameters

Reusable owned callback parameters can satisfy a scoped mutable borrowed callback argument without losing ownership. One directional compatibility rule is shared by named-expression checks and structural callable matching. Borrowed-to-owned conversion, once-to-reusable conversion and assuming a readonly contract for an abstract mutable owned environment remain rejected. Native lowering reuses ordinary argument borrowing; no wrapper allocation or capture copying is introduced by this conversion.

Public native checks verify repeated reborrowing followed by invocation, reborrowing followed by ownership transfer into a returned callable, and overlap rejection when mutable callback arguments are actually used. An initial unused-argument overlap expectation was corrected: existing access inference treats unused borrowed parameters as readonly; that unchanged behavior is not evidence of a mutable-loan failure. Callable ownership defaults (3 earlier cases plus the overlap/return case), retained_callbacks (10), callback_moves (10), callbacks (18), single_use_callbacks (11) pass across focused runs. Production compiler/LSP Clippy passes. The focused reborrowing signature-help test preserves the distinct borrowed and owned declarations. No full checkpoint regression or timing measurement is claimed.

Factory result syntax, remaining migration coverage and matched workload admission remain pending; the overall language-change and S06 gates are open.

### Increment 4a: explicit factory result contracts

Structured and declaration result parsing now share the annotation parser. Explicit `once` and the legacy `owned once` result forms retain an owned-once structural contract; bare callable results retain inference, and `owned` results remain accepted aliases. Grouping and original source coordinates are retained. Result discovery and definitive return checking adapt reusable environments to consuming wrappers consistently. Borrowed callable result annotations produce `callback-escape` before native emission.

A factory-only once wrapper exposed missing native trait emission when no callback parameters existed. Trait need is now computed once from callback declarations and wrapper origins, avoiding repeated scans; readonly/mutable/consuming traits are emitted when the wrappers require them. No allocation or environment copy is introduced by the invocation restriction itself.

Focused native checks verify capture-free/scalar once results, grouped owned string results, old ownership aliases, second-call rejection, forwarding owned parameters, and nominal generic payloads. The generic payload fixture uses a typed source record because anonymous object literals cannot establish nominal generic inference. Forwarding/generic execution passes at optimization levels 0 and 3 with exact output `7 Pat`. Internal library (72), callable_results (6), callback_returns (10), type_syntax (8) pass. The focused explicit-once result IntelliSense case passes, preserving source labels and chained string methods. Production compiler/LSP Clippy passes. Original failures and repairs are retained under /tmp/tsrust-once-result-*.log.

The overall migration remains open for remaining source/editor fixtures and matched workload admission; no full regression checkpoint or timing measurement is claimed.

### Increment 5a: matched workload admission and checkpoint oracle wiring

Migrated seventeen existing higher-order workloads' intentional scoped callback parameters to `mutate`, preserving their TypeScript counterparts and result protocols. Original sources, manifests and policy records are retained in `target/editor/callable-default-workloads-l3s5i4dq`; reviewed source fingerprints were updated only for the changed sources. All seventeen native ABIs pass; eleven scalar workloads pass thirty-three independently checked native execution cases. The remaining six workloads have ABI evidence here, not complete execution evidence.

Registered `default_conditional_callable_fields` and `once_factory_results` for checkpoint 101. Their focused checks verify the standard native workload ABI and exact results against compiled TypeScript and independent arithmetic specifications in debug and release execution. These checks establish correctness, not throughput or allocation measurements. Checkpoint 96 is accepted under the current workflow; source migrations and these new registrations are queued for 101 without altering historical SQLite records.

Admission inspection found missing central correctness-oracle dispatch for these workloads and earlier `owned_callable_fields`, `annotated_once_locals` and `generic_function_record_pipeline` registrations. A separate callable-foundation oracle now supplies all five specifications. Focused tests compare central dispatch and arithmetic results against independent per-iteration calculations over zero, boundary and 20,000-iteration inputs. Both new native workload tests also assert the central oracle before checking execution. All three focused tests pass. Remaining migration coverage and the overall S06/standard-library roadmap remain open; no full checkpoint regression or timing run was restarted.

### Increment 5b: typed closure argument fixture migration

The focused `closure_arguments` suite exposed four old-default assumptions. Intentional mutable borrowing in scalar calls, factory-result reuse and forwarding now uses `mutate`. The scoped borrowed-return rejection remains explicit; a positive executed owned-once callback capture/return case replaces the obsolete owned capture rejection. All twelve tests pass. Original fixture and failing evidence remain in `/tmp/tsrust-closure-arguments-before-default.rs` and `/tmp/tsrust-default-remaining-callables.log`; repaired evidence is `/tmp/tsrust-default-closure-arguments-repaired.log`.

The next focused higher-order suite remains failing: its reusable callback fixtures still use bare annotations, and its recursion/depth witness aborts on the normal worker stack. Preserve `/tmp/tsrust-default-higher-recursive.log`. An isolated non-depth test confirms the first reusable-forwarding failure is `closure-consumed` after a repeated call, recorded in `/tmp/tsrust-default-higher-flat.log`. The combined command stopped before `recursive_callable_results`; that suite has no new acceptance evidence here. Next work must migrate deliberate borrowed contracts and isolate the depth witness by phase without increasing the worker stack. Overall acceptance remains open.

### Increment 5c: reusable higher-order data-rank contracts

Migrated only the deliberate reusable-borrowing case in `higher_order_function_values` to explicit `mutate` contracts, including nested callback inputs and inline function consumers. Scalar/data access annotations and owning factory result annotations retain their meanings. The isolated test now passes its exact native outputs and existing observer assertions at optimization levels 0 and 3. Evidence: `/tmp/tsrust-default-higher-flat-repair.log`; original test source: `/tmp/tsrust-higher-order-before-default.rs`. The other five cases, including the previously observed depth abort, remain unaccepted and require separate investigation. This fixture migration does not establish a new runtime throughput measurement.

### Increment 5d: returned higher-order consumers and callback kind fixtures

Made deliberate borrowing explicit in the recursive-signature fixture's consumer inputs, nested callback inputs, inline consumers and two-callback declarations. Returned callable annotations remain owning factory values; existing owned keeper inputs retain their contracts. Also corrected the separately named `borrowed` declaration in the all-callback-kinds fixture to `mutate`, retaining once/owned/owned-once cases. Both isolated tests pass their existing exact output and native observer checks at optimization levels 0 and 3. Original failures and repaired outputs are retained in `/tmp/tsrust-default-higher-signatures*.log` and `/tmp/tsrust-default-higher-kinds*.log`. Three higher-order cases remain unaccepted, including the depth witness. No checkpoint timing or full regression evidence is claimed.

### Increment 5e: native ownership cleanup and representation witness

Made the cleanup fixture's `drive` callback explicitly mutable-borrowed, restoring the generic native function used by its observer. Made its separate conservative array-representation callback explicitly borrowed as well; a bare owned contract specializes differently and cannot serve as the former borrowing witness. The existing assertions remain intact: pointer preservation for ownership transfers, distinct storage for explicit copies, IEEE bit preservation, cleanup after unwinding, zero-size capture-free environments, Send/Sync, and conservative versus fixed array representation. The isolated test passes at optimization levels 0 and 3. Preserve initial native compilation and intermediate representation failures in `/tmp/tsrust-default-owned_results_ieee_bits_cleanup_and_standard_capture_layout.log` and `/tmp/tsrust-default-owned-results-repair.log`; final evidence is `/tmp/tsrust-default-owned-results-final.log`.

The alias/contract-rejection case still has obsolete default assumptions (its first contract assertion observes Consume rather than Mutate), and the depth witness remains unresolved. No overall suite acceptance or timing measurement is claimed.

### Increment 5f: alias and structural rejection contracts

Migrated the alias/capture fixture's deliberate scoped callback contracts to `mutate`, preserving explicit owning cases and factory-result ownership. Readonly callable syntax is now supported, so its obsolete syntax rejection was replaced with a readonly-borrowed-to-owned rejection. Nested mutable-versus-owning input incompatibility now correctly expects `callback-access`, which reports the precise mutate/consume argument-access mismatch. Existing ownership escape, alias origin, consuming reuse, evaluation order, live loan, arity, type and data-access checks remain asserted. The isolated case passes exact native output at optimization levels 0 and 3 plus all rejection checks. Preserve the intermediate diagnostic mismatch in `/tmp/tsrust-default-higher-aliases-repair.log`; final evidence is `/tmp/tsrust-default-higher-aliases-final.log`. Only the depth case remains unaccepted in this higher-order suite; no complete suite acceptance or checkpoint timing is claimed.

### Increment 5g: depth witness phase isolation

Ran the existing depth witness alone on the normal test-worker stack with temporary phase tracing. Its initial recursive native execution completes. At expression depth 100, `check(&source)` succeeds; the abort occurs after entering `Analysis::emit_rust()`, before nested annotation cases are reached. Preserve `/tmp/tsrust-default-depth-phase.log` and `/tmp/tsrust-default-depth-check-emit.log`. Temporary tracing was removed by restoring the exact pre-probe fixture, retaining all preceding migration repairs. This is emission-phase evidence, not a parser/analysis failure or a fix. Next action is inspecting emitter recursive compiled frames and reducing the responsible frame/dispatch state without increasing worker stack size or reducing the accepted source-depth witness.

### Increment 5h: bounded scalar unary emission

Inspected compiled ARM64 emitter frames using `otool`; retained `/tmp/tsrust-emitter-depth-disassembly.txt`. Ordinary nested scalar unary expressions recursively retained the large `expr_inner` frame. Scalar positive/negate/not/bit-not emission now dispatches to a small dedicated helper, preserving checked integer negation errors and number behavior. Number bit-not and nullish/reflection/callable unary paths retain their specialized handling. The unchanged depth-100 expression now emits successfully on the normal worker stack and depth-200 rejection remains checked. Subsequent failure exposed an old borrowing assumption in the native-instantiation budget witness; its deliberate borrowed inputs now use `mutate`, preserving the original distinct-origin and deduplication budgets. All six `higher_order_function_values` tests pass, including native debug/optimized execution, depth limits, structural rejections and cleanup observers. Evidence: `/tmp/tsrust-default-depth-small-unary.log` (intermediate budget failure) and `/tmp/tsrust-default-higher-suite-final.log` (complete focused suite pass). Broader affected unary suites and production checks remain pending; no checkpoint timing is claimed.

### Increment 5i: unary verification and recursive result migration

Focused integer operators (4) and number operators (4) pass after scalar unary emission extraction, including depth, checked traps, numeric widths and native behavior. Production compiler/LSP Clippy passes with warnings denied. The resumed `recursive_callable_results` suite exposed two old scoped-borrowing assumptions; only those callback declarations now use `mutate`, preserving the borrowed capture mutation case and borrowed environment escape rejection. All sixteen recursive result tests pass after repair. Original fixture is `/tmp/tsrust-recursive-results-before-default.rs`; failing/partial evidence is `/tmp/tsrust-default-unary-recursive-checks.log`; repaired suite is `/tmp/tsrust-default-recursive-results-final.log`; production lint evidence is `/tmp/tsrust-default-unary-clippy.log`. This is focused acceptance evidence, not a full regression checkpoint or performance measurement. Remaining callable/editor migration coverage and S06 implementation remain open.

### Increment 5j: current editor parity and development binaries

Current intelligence (65), project intelligence (28) and code actions (4) pass, including callable default/read/mutate/once help, imported nominal types, clone-fix validation and executable built-in examples. VS Code grammar checks pass. Rebuilt development compiler/LSP binaries under `target/editor` successfully using the actual compiler package's `tsrust` binary; the initial nonexistent `tsrust-cli` package invocation was corrected without source changes. Evidence: `/tmp/tsrust-default-editor-parity.log`, `/tmp/tsrust-default-grammar.log`, and `/tmp/tsrust-default-editor-build-repaired.log`. A running VS Code server must be restarted through `Tsrust: Restart Language Server` to load the rebuilt process. These checks do not close remaining migration coverage or the overall S06 roadmap and do not establish checkpoint measurements.

### Increment 5k: captured callable and move fixtures

Explicitly borrowed callback inputs now preserve nested captured callback loans and the rejection of escaping those loans in `callable_captures`. Its six tests pass. The reusable-forwarding declaration in `callable_moves` now uses `mutate`; all ten move tests pass, retaining owning and once transfer cases. `mutable_callables` passes all ten cases unchanged. Evidence is `/tmp/tsrust-default-remaining-ownership-repaired.log` and `/tmp/tsrust-default-ownership-followup.log`; original captured-callable fixture is `/tmp/tsrust-callable-captures-before-default.rs`. The subsequently reached `owned_closure_arguments` suite has eight passes and six failures requiring investigation; no acceptance is claimed for that suite. These are focused migration checks, not a full regression checkpoint or timings.

### Increment 5l: owned data arguments with borrowed callback environments

All six failures in `owned_closure_arguments` were old environment-borrowing assumptions. Made callback environment borrowing explicit in the affected fresh-consumed-data, temporary string, factory-result reuse, mixed data-access and forwarding fixtures. Data argument `read`/`mutate`/`consume` contracts retain their exact meanings; once environment tests and explicitly owning declarations remain intact. The nonescaping-return rejection now explicitly borrows its callback parameter. All fourteen tests pass, including independent environment versus data ownership, backend-free diagnostics, exact access matching, argument evaluation loans and native outputs. Original fixture is `/tmp/tsrust-owned-arguments-before-default.rs`; repaired evidence is `/tmp/tsrust-default-owned-arguments-repaired.log`. This focused migration repair adds no new runtime method or representation and no timing measurement; broader roadmap acceptance remains open.

### Increment 5m: closure corpus and supported callable fields

The standalone closure suite retained an obsolete rejection of a bare callable record field, now a supported owning field. Replaced that single obsolete rejection with executed construction/invocation (`7`); remaining unsupported routes and diagnostics remain checked. All fourteen closure tests pass; both handwritten closure witness tests passed in the initial focused command and were preserved. Originals/evidence: `/tmp/tsrust-closures-before-default.rs`, `/tmp/tsrust-default-closure-tail.log`, `/tmp/tsrust-default-closure-tail-repaired.log`. The subsequently reached nested closure suite has eight passes and five failures requiring investigation. No acceptance is claimed for that suite or the full migration; no checkpoint regression/timing was restarted.

### Increment 5n: nested closure environment migration

Made deliberate scoped callback environment borrowing explicit in nested closure fixtures while preserving data-access annotations and owned factory results. Replaced the obsolete owned-once capture rejection with executed nested capture/invocation (`7`); the explicitly borrowed nested capture rejection remains. All thirteen nested closure tests pass, retaining ancestor capture propagation, argument loan/conflict spans, fresh-data consumption, reusable outer environments, lexical depth guards and backend-free diagnostics. Original fixture is `/tmp/tsrust-nested-before-default.rs`; repaired evidence is `/tmp/tsrust-default-nested-repaired.log`. No full checkpoint regression or performance measurement is claimed; the broader S06 and standard-library roadmap remain active.

### Increment 5o: public borrowing explanation

Added a public callable guide example distinguishing borrowed callback reuse from bare owning callback parameters and explaining callback environment versus data argument ownership. Executed the exact example through the current development CLI: output `3`, `3`, `7` on separate lines. Source is retained at `target/editor/callable-guide-borrowing.tsr`. Corrected the callable-field guide opening typo. Nullable callable transport is currently a separate in-progress compiler gate 97; its existence is not acceptance evidence, so the guide retains its pending status. No new built-in or timing workload is introduced by this documentation clarification.

### Increment 5p: authoritative foundation status reconciliation

Reconciled stale opening summaries in the S06 and callable-container ledgers with their existing accepted evidence. They now distinguish implemented structured annotations/generic records/callable fields from remaining generic inference acceptance, direct callable arrays, recursive/nested layout cleanup and Result work. Historical evidence and pending measurement records remain intact. This prevents future sequencing from restarting already implemented field storage or treating partial generic work as full S06 acceptance. The next container implementation remains gate 4's direct callable array path, with its explicit ownership, indexing, extraction, clone-operation and editor requirements.

Further fixture migration: `capture_replacement`, `declared_function_values`, `conditional_expressions`, `recursion`, `closures` and `closure_arguments` now state scoped borrowing explicitly, and forwarded `mutate` callbacks are emitted and counted again; see [callable and depth regression repairs](./callable-regression-repairs.md).
