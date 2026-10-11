# Standard testing library implementation plan

Status: active. The first implementation targets synchronous Vitest-style `expect` and Node assertions. Full Node/Vitest compatibility is not claimed. This document tracks observable parity separately from compiler prerequisites and acceptance evidence.

## Acceptance rules

Each API starts with a focused failing test, then receives compiler/native implementation, diagnostics, LSP completion/hover/signature help, executable examples, and behavioral documentation in `crates/tsrust-lsp/src/builtins.json`. Keep benchmark sources, independent oracles, source hashes, ABI support, guards, and registrations current as implementation lands. Native output and failure exit codes must be verified. Preserve exact originals under `archive/testing-library-start/` before risky edits.

A feature-family milestone requires full compiler/LSP regressions, matched native versus Node-after-tsc measurements, and a correctness/performance review in the canonical SQLite store. Focused checks do not establish measured performance. No testing-library benchmark has been measured yet.

## Scope and module surface

Expose familiar testing API names through the single public import `std:testing`. Node and Vitest are parity references, not dependencies. Keep names, argument order, defaults, return behavior, and failure behavior aligned with the corresponding API where supported. Explicitly reject unsupported options and APIs rather than silently approximating them. Existing relative `.tsr` imports and `std:path` must retain their behavior.

Provide a `tsrust test` command with explicit file selection and conventional `.test.tsr` / `.spec.tsr` discovery. Test sources should not require a hand-written `main`. The runner must report test names, passing/failing/skipped counts, and a nonzero exit code for failures, compilation errors, or invalid usage. Assertion failures stop the current test while later tests still run. Suites and hooks must preserve documented order and isolation.

## Parity ledger

| Area | Target API | State | Compiler or runtime prerequisite |
| --- | --- | --- | --- |
| Module imports | Named imports from `std:testing` | Implemented; focused native and editor acceptance | Bundled source modules and private native bindings |
| Test files | Top-level test declarations without `main` | Implemented for explicit files and conventional `.test.tsr` / `.spec.tsr` entries, including nested suites | AST entry-point handling preserves original source spans |
| Runner | `tsrust test`, discovery, reporting, exit status | Implemented for synchronous entries; focused acceptance | One linked native registry, deferred callbacks and recoverable failure boundaries |
| Test registration | `test(name, callback)`, `it(name, callback)` | Implemented; focused acceptance | Owned synchronous reusable callbacks and deferred registry |
| Suites | `describe(name, callback)`, `suite` alias; nested suites | Implemented; focused acceptance in progress | Immediate borrowed registration callbacks and nested deferred traversal |
| Selection | `test.skip`, `test.only`, `describe.skip`, `describe.only`, `test.todo` | Pending | Callable properties or compiler-recognized API dispatch; selection across files |
| Hooks | `beforeAll`, `afterAll`, `beforeEach`, `afterEach`; `before`, `after` aliases | Implemented for synchronous void callbacks; focused acceptance | Owned reusable callbacks, implicit file scopes, stack ordering and recoverable teardown |
| Scalar identity | `expect(actual).toBe(expected)`, `strictEqual`, `notStrictEqual` | Implemented; focused native/editor acceptance and matched untimed workload checks | Borrow-safe generic matcher dispatch; Object.is numeric semantics, including NaN and signed zero |
| Structural equality | `toEqual`, `toStrictEqual`, `deepStrictEqual`, `notDeepStrictEqual` | Pending | Type-directed recursive comparison for arrays, tuples, records and unions; no stringification-based equality |
| Negation | `expect(actual).not.<matcher>` | Implemented for current matchers; focused native/editor and matched untimed acceptance; chained `.not.not` pending | Shared readonly owned environments; chained view representation still needed |
| Truthiness | `toBeTruthy`, `toBeFalsy`, `ok` | Implemented; focused native/editor and matched untimed acceptance | Reuse compiler truthiness for supported source values; do not restrict `ok` silently to booleans |
| Nullish predicates | `toBeNull`, `toBeUndefined`, `toBeDefined` | Implemented for supported data unions and concrete values; focused native/editor and matched untimed acceptance | Generic/union matcher dispatch |
| Numeric matchers | `toBeNaN`, four relational matchers, `toBeCloseTo` | NaN, relational and proximity methods implemented with negation; focused native/editor and matched untimed acceptance | Bigint support remains unavailable; int-to-number precision limits are documented |
| Runtime type matcher | `toBeTypeOf` | Implemented with negation; focused native/editor and matched untimed acceptance | String-literal expected type restrictions await compiler support |
| Collection matchers | `toHaveLength`, `toContain`, `toContainEqual` | Length implemented for strings, arrays, tuples, numeric record properties and unions with negation; focused native/editor and untimed workload acceptance; containment pending | Containment needs borrowed comparison dispatch |
| Errors | `toThrow`, `assert.throws`, `doesNotThrow`, `fail` | Pending | Assertion error representation and recoverable native failure boundaries; general thrown errors need compiler support |
| Parameterization | `test.each`, `describe.each` | Deferred | Typed heterogeneous row application, callable properties, runner expansion |
| Test context | Node `t.test`, `t.skip`, `t.todo`, diagnostics | Deferred | TestContext type, callback parameters, dynamic subtest registry |
| Async tests/hooks | Promise return values, `await`, done callbacks | Blocked by compiler support | Async/await, Promise types, task scheduling, rejection propagation |
| Async assertions | `resolves`, `rejects`, `assert.rejects`, `doesNotReject` | Blocked by compiler support | Promise types and asynchronous failure capture |
| Timeouts/concurrency | Test options, cancellation, concurrent suites | Blocked by compiler/runtime support | Async scheduler, cancellation, timeout semantics and shared-state rules |
| Mocks/spies | `vi.fn`, `vi.spyOn`, Node mocks, call history | Deferred; some forms blocked | Typed callable wrappers and call history; dynamic member replacement and module mocking need additional compiler support |
| Fake timers | `vi.useFakeTimers`, clock advancement | Blocked by platform support | Timer APIs and injectable scheduler |
| Snapshots | `toMatchSnapshot`, inline snapshots | Deferred | Stable value serializer, snapshot storage/update protocol; inline source edits |
| Coverage/watch | Coverage reports, watch mode | Deferred tooling | Instrumentation/source maps, filesystem watching and incremental test builds |
| Dynamic JavaScript values | Prototypes, symbols, sparse arrays, arbitrary objects, asymmetric matchers | Blocked or outside current language model | Corresponding compiler value types and object semantics |
| Node assertion namespace | Default `assert`, callable `assert(...)`, `assert.*` | Deferred | Default/namespace imports and callable objects; named strict exports lead |

“Pending” means implementation is planned, not accepted. “Deferred” identifies a later scope, not necessarily a missing compiler capability. “Blocked” requires the named language/platform prerequisite. Confirm suspected blockers with focused compiler witnesses before closing a scope item.

## Implementation sequence

1. Finish the independent generic formatter defect and retain compiler/native and LSP protocol evidence.
2. Implement bundled testing modules and scalar Node assertions. Verify aliases, optional messages, arity/type errors, evaluation order, repeated use of owned inputs, NaN, and signed zero.
3. Implement Vitest `expect`, scalar predicates, numeric matchers and negation. Preserve borrowing and temporary lifetimes. Verify all documented examples and editor surfaces.
4. Add recursive structural comparison and collection matchers. Use independent Node/Vitest result oracles for supported values, including nested arrays, tuples, records, unions, empty containers, and Unicode strings.
5. Implement synchronous test/suite registration, automatic test entry points, runner discovery, failure isolation, skip/only/todo, and hooks. Verify teardown on assertion failure and continued execution of later tests.
6. Complete diagnostics, editor integration, examples, benchmark support and the parity review. Run the family milestone regressions and measurements. Record accepted scope and remaining compiler blockers without relabeling unsupported APIs as complete.
7. Revisit deferred features only after their prerequisites land. Add focused witnesses and acceptance evidence for each new capability.

## Current evidence

- Formatter defect reproduced by `readonly_tuple_generic_delimiters_stay_tight`: the previous output retained `ReadOnly < [string, string] >`.
- Formatter detection now uses the parameter annotation parser and includes the outer ownership wrapper span. This retains compact `ReadOnly<[string, string]>` while preserving comparison/shift operator spacing.
- `cargo test -p tsrust-compiler --test formatting`: 9 passed, including native output `3\n`, token preservation, and idempotence.
- `cargo test -p tsrust-lsp --test stdio formatting_readonly_tuple_generics_returns_tight_idempotent_edit -- --exact`: 1 passed. Formatting returns the compact type, and a second formatting request returns no edits.
- Testing-library focused test added: `node_assert_strict_exports_accept_values_and_optional_messages`. Implementation acceptance remains pending.
- Original module-path validation rejected `node:assert/strict` before type checking. Bundled module integration now addresses this witness.

### Foundation implementation progress

The initial implementation loaded compatibility aliases `node:assert/strict` and `std:assert` (now replaced by `std:testing`) through the ordinary module graph. Initial exports are `ok`, `strictEqual`, `notStrictEqual`, and `fail`. Assertions use a bundled native failure primitive; failures stop execution with `AssertionError: <message>`. This foundation is not the test runner and does not yet isolate test failures.

`cargo test -p tsrust-compiler --target-dir /tmp/tsrust-testing-acceptance --test testing_library`: 3 passed. Checks cover scalar assertions, optional custom messages, NaN equality, signed-zero inequality, null/undefined, owned Unicode input reuse, and stopping execution after failure. A separate target directory avoided an existing build-directory lock. The original red test rejected `node:assert/strict` with `module-path`; retain that evidence.

Remaining before these exports are accepted: complete catalog documentation, LSP imported symbol/help checks, diagnostic boundaries, matched benchmark support, and module regression checks. The ledger remains Pending until those pass.

The initial compiler witness at `/tmp/tsrust-expect-proof.tsr` rejected direct `expect(3).toBe(3)` when `expect` returns a generic record containing a callable field (`unsupported: Unsupported call or method`). Existing callable-field invocation dispatch relies on a type hint that does not resolve this generic call result. Permanent regressions and generic result/member resolution now address this witness, as recorded below. Do not change the public API to require an intermediate variable.

### Vitest identity matcher and editor progress

The initial `expect` implementation used `vitest` and `std:test` aliases; current imports use only `std:testing`. Direct `expect(value).toBe(expected)` calls and stored assertion objects execute natively. Checks cover integers, booleans, Unicode strings, NaN, signed zero, repeated checks, assertion failure, retained string input through explicit `clone`, borrowed expected inputs, and invalid arity/types/exports. This does not yet implement test registration, a runner, negation, other matchers, or hooks.

The expectation object follows Tsrust ownership: scalar actual values copy; data-owning actual values transfer into its captured environment. Do not claim JavaScript reference retention across mutations of a transferred owner. The full matcher design must preserve observable identity; cloning captured arrays/records merely to populate many matcher closures is not an acceptable substitute.

Compiler prerequisites implemented:

- Generic factory receivers resolve before callable-field dispatch. Preparation establishes type information without evaluating arguments.
- Generic callable `ReadOnly<T>`, `Mutable<T>` and `Owned<T>` parameter annotations copy primitive scalar specializations. Explicit concrete scalar ownership wrappers remain invalid. String and other data specializations retain their ownership contract.

Editor integration implemented:

- Bundled source modules receive `tsrust-stdlib:///...` identities for imported documentation and source ranges.
- Imported bundled declarations and fields use the behavioral catalog documentation.
- Direct generic factory results participate in concrete field materialization, even without a local variable.
- Generic callable parameter help removes ownership wrappers for scalar specializations, matching compiler signatures.

Evidence on the current implementation:

- `testing_library`: 10 tests passed before the additional invalid-API test; that focused test also passed. The suite contains API checks and compiler prerequisite regressions.
- `callable_fields`: 35 passed.
- `type_substitution::tests`: 3 passed.
- `native_bindings`: 1 passed.
- LSP `project_intelligence`: 31 passed, including imported assertions and direct generic matcher hover, completion and parameter help.
- Seven assertion catalog examples and three expectation catalog examples compiled and ran with expected output. Exact source hashes and outputs are retained in `archive/testing-library-start/evidence/assertion-catalog-examples.json` and `expect-catalog-examples.json`. Catalog hashes identify the original catalog state for each check; unchanged example sources retain their individual fingerprints.
- Compiler `project`: 72 passed, 3 failed against old unknown-generic diagnostics. An isolated source baseline restoring all pre-task compiler snapshots reproduced exactly the same three failures (4 other structured checks passed). The failures expected the former unsupported-generic diagnostic; current unknown constructors correctly report `type` and point at `Box`. The original tests are retained in the archive. Only the three affected expectations were repaired. All seven structured checks pass on the current source; the original 72 successful checks remain evidence.

Acceptance remains open. Benchmark counterparts, oracles, ABI/driver support, registrations and untimed validation must be added before these APIs are marked accepted. No testing-library timings or completed family milestone are claimed.

### Next implementation work

Complete the synchronous runner before expanding matcher breadth: typed native callback registration, test/suite execution with recoverable assertion boundaries, teardown after failures, automatic test entry points, discovery, reporting and exit status. Keep Node/Vitest names and callback ordering. Avoid rerunning a file separately for each test as a substitute for a registry: that would repeat registration side effects and lose state shared by tests and hooks.

Then add negation and the remaining synchronous matcher/assertion families while preserving receiver ownership and identity. Keep the parity ledger split by individual supported APIs rather than treating one passing matcher as complete assertion parity.

## Canonical namespace and module layout

The only public testing import is `std:testing`. Node and Vitest describe API parity targets, not runtime dependencies. The toolchain never loads Node, Vitest, JavaScript packages, or a separately installed source language to execute this library.

Compiler sources live in `crates/tsrust-compiler/src/stdlib/testing/`. `api.rs` embeds the checked generic declarations as one bundled module. Rust bindings and runtime code live beside that Rust source file. The private `std:testing-runtime` bridge is available only to the bundled module. The existing path module is similarly organized under `src/stdlib/path/`. The compiler-wide native registry retains shared types and references module-owned bindings.

Namespace acceptance: the combined assertion, expectation and registration import has a focused native regression. Previous compatibility import names are removed from the public source registry. Full testing parity remains in progress; this organization change does not imply suite, hook, skip/only, async, mock, snapshot or coverage support.

Namespace acceptance results: all 13 existing testing native checks passed after module organization; the additional private/legacy import rejection check passed. Both affected LSP hover/signature/completion checks passed with `std:testing`. These are focused correctness checks, not full milestone regressions or performance measurements.

## Test entry implementation

`tsrust test <file.tsr>` accepts top-level synchronous registrations, declarations and setup statements. It synthesizes an AST entry function without rewriting the user's source, runs the deferred registry once, continues after assertion failures, prints results, and exits with status 1 if any test fails. Empty files report zero tests and exit successfully. Helper functions and copied scalar setup captures are supported. Explicit `main` declarations are rejected in test entry files.

Conventional `.test.tsr` and `.spec.tsr` entry paths receive the same parsing in ordinary compiler/project/editor checks. Their diagnostics retain original lines and columns. Explicit `test` commands can use other filenames. Imported ordinary modules retain declaration-only parsing. The private compiler finish binding owns process status; no user import or external test runtime is required. Empty entries link the runner without unused callback adapter code.

Focused evidence: all 17 compiler testing checks passed, including failed-test continuation, success/failure status, empty entries, helper/setup capture, `main` rejection and original diagnostic line locations. The LSP top-level test entry check passed for imported test hover and generic matcher hover/signature help. Discovery, multiple-file selection, suites, hooks and skip/only/todo remain pending. No performance measurement is claimed.

Affected prerequisites were also checked: seven native import/callback project checks and all eighteen callback contract checks passed. These bounded checks validate the changed native callback contracts without replacing the pending family milestone.

## Discovery and multiple-file registry

`tsrust test` discovers `.test.tsr` and `.spec.tsr` recursively below the current directory. Explicit files and directories may be supplied in argument order. Directory results sort lexicographically; canonical file identities are deduplicated. Explicit files may use any `.tsr` name. Discovery ignores hidden child directories, `target`, `archive`, `node_modules`, and symlinks; explicit symlink file arguments are resolved normally. No selection is an error. Discovery does not load a foreign runtime or execute sources.

All selected entries link and compile before any setup or registration code runs. Each file's registration runs once, followed by one shared native registry traversal and one summary. Shared imported modules link once. A type error in a later file prevents earlier setup from executing and points at the original later file. This architecture supports later selection across files without replaying registration.

Focused discovery evidence: 21 compiler/native testing checks passed, including default discovery order, explicit order/deduplication, compilation before registration, ignored build/archive/hidden trees, a symlink cycle, empty discovery, invalid options and missing paths. The affected top-level LSP check also passed after the linker change.

Suite parity references: [Vitest describe](https://vitest.dev/api/describe) and [Vitest hooks](https://vitest.dev/api/hooks). The synchronous supported surface uses string names and zero-argument void callbacks. `describe` and `suite` register nested groups immediately and defer test bodies; options, async registration and timeouts remain unsupported. Hook ordering and teardown are still pending implementation.

## Nested suite focused evidence

The registry stores suite/test entries in source order and traverses nested suites deterministically. `describe` and `suite` invoke registration callbacks immediately, then test bodies execute after all registration. Results include the full suite path. Assertion failure does not stop later nested or root tests. Attempted suite registration from a running test fails that test and later tests continue.

Registration callbacks use `Mutable<Function<void>>` so named reusable callbacks are borrowed rather than consumed and may update their environment across registration calls. Test callbacks still transfer into the deferred registry. The suite name is copied and leaves its source owner available. Native ownership rules still govern captures; no JavaScript shared-reference behavior is implied.

Current evidence: all 26 testing native checks passed. Suite hover/signature help passed with the mutable borrowed callback contract. Five catalog examples for test/it/runTests/describe/suite compiled and executed with expected output; exact source, source hashes and catalog hash are retained at `archive/testing-suites-start/evidence/catalog-examples.json`. Top-level return statements, including returns nested in registration control flow, are rejected before execution while callback returns remain valid.

Focused implementation is complete for these suite APIs. Family acceptance remains open: hooks, selection, broader matchers, matched benchmark support, full regressions and timing review are pending. No testing workload has been measured.

## Synchronous hooks and file scopes

Hook callbacks transfer into the current suite or implicit file scope. Before hooks run in registration order; after hooks run in reverse order. Per-test setup traverses ancestors before children; teardown traverses children before ancestors. Reused owned mutable callback environments retain their state. Separate selected files have separate implicit scopes, so file-level hooks never leak into another file.

A beforeEach failure skips the body, but all afterEach callbacks for the test's full scope chain still run. Body and teardown errors are retained together; the test counts as one failure. A beforeAll failure blocks descendant bodies and later setup in that scope, but the scope's afterAll callbacks still run and sibling suites/files continue. Descendant scopes whose setup was blocked do not run hooks. Every afterAll callback is attempted, even after another teardown failure. Empty scopes do not invoke hooks.

Suite hook errors are reported after their phase. The summary adds a separate hook-failure count when beforeAll/afterAll fails. `runTests` returns failing test count plus these hook failures; the test command exits 1 if either is nonzero. Registration and recursive execution from a running test or hook are rejected at the recoverable boundary. The normal success summary is unchanged.

The supported callbacks have no parameters and return void. Hook timeout/options, context arguments, synchronous cleanup return values and asynchronous callbacks remain unsupported; arity/types reject them. Async forms still require compiler scheduling and Promise support. Cleanup return values are deferred native-binding/signature work rather than claimed as implemented. Default stack ordering follows the documented [Vitest hook behavior](https://vitest.dev/api/hooks); configurable ordering is deferred.

## Benchmark support in progress

The `testing_scalar_assertions` runtime workload now calls actual `ok`, `strictEqual` and `notStrictEqual` exports from `std:testing`. Its TypeScript counterpart calls actual Node strict assertions for developer measurement; this creates no compiler/library runtime dependency on Node. The workload dynamically selects squared integer values and compares them with independently computed expectations, retaining a bounded checksum. It includes pool setup, indexing, arithmetic, assertion dispatch and observation; it is not an isolated matcher-latency measurement.

Matched sources, manifest registration, exact source hashes, real-API guards, independent BigInt checksum oracle and ordinary scalar ABI/driver integration are present. Three focused harness tests pass, including nested compiler-source fingerprint coverage. Ten representative native/compiled-Node result cells pass without timing samples. The quick check uses existing drivers with one untimed warmup and zero timing samples; it does not change the measurement protocol. Evidence is retained at `archive/testing-hooks-start/evidence/assertion-benchmark-correctness.json`.

Compiler input identities now include nested `.rs` backends and bundled `.tsr` library surfaces. Root-only Rust fingerprinting would have missed the relocated path runtime and new testing source. Existing measurements and original source identities are preserved. Expectation and runner/hook benchmark counterparts remain to be added before the family milestone; no timing or complete regression milestone is claimed.

Hook focused acceptance evidence: all 35 testing compiler/native checks passed. The owned reusable callback hover/signature check passed. Six hook catalog examples compiled and ran with exact expected output, with source and catalog hashes retained at `archive/testing-hooks-start/evidence/hook-catalog-examples.json`. Invalid arity, unsupported callback results/parameters, and escaped borrowed captures produce source diagnostics before native compilation. These checks do not close the pending family milestone.

Supplemental benchmark evidence now captures recursive compiler input hashes as well as the current compiler binary identity: `archive/testing-hooks-start/evidence/assertion-benchmark-current-correctness.json`. The original quick-check record is retained unchanged. Both records contain correctness cells only; neither is a performance measurement.


## Installed CLI and presentation acceptance

The installed CLI was an older build whose usage omitted `test`. The current
optimized executable now replaces that installation, with the previous binary
retained byte-for-byte in `archive/cli-presentation-start/installed-tsrust`.
Help groups commands, options and examples. Missing commands and invalid options
produce concise errors with a help hint rather than labeling usage as a toolchain
failure. Help, check/build success labels, source diagnostics and native testing
labels support terminal-aware color and explicit overrides. `--json`, emitted Rust
and formatted source do not acquire presentation escapes.

Three focused CLI tests, all 35 testing checks and nine formatting checks passed.
The installed executable discovers `examples/testing/basic.test.tsr` with plain
`tsrust test`, producing two passes. This closes the stale-installation repair and
focused CLI presentation acceptance; broad matcher parity, selection APIs and the
pending testing-family measurements remain tracked separately above.


## Rust-only implementation layout

The testing implementation directory now contains only `.rs` files. `api.rs`
embeds the checked public module declarations in a Rust string constant;
`mod.rs` exposes that constant to the existing bundled-module loader. The
standalone assertion, expectation and registration `.tsr` files are removed.
The declarations still supply generic signatures, ownership analysis and source
locations to the compiler and LSP; native runtime execution remains Rust.
The combined module source is byte-identical to the previous composition, so
`std:testing` imports and observable behavior remain unchanged. Exact original
files and the combined source hash are preserved in
`archive/testing-rust-layout-start/`.

Rust-layout focused acceptance: all 36 testing compiler/native checks, three CLI checks and four bundled testing LSP checks pass. No new performance measurements were required for the byte-preserving source relocation.


## Truthiness matcher progress

`Expectation<T>` now exposes zero-argument `toBeTruthy()` and `toBeFalsy()`.
The factory captures Boolean conversion before transferring actual into the
identity matcher. Predicate callbacks retain copied booleans, so reusable string,
array and record expectations do not clone owners or change identity behavior.
Failure uses the existing recoverable assertion boundary. All 38 testing checks
pass, covering falsy primitive edge cases, truthy arrays and Unicode strings,
repeated stored expectations and continuation after failures. Both catalog
examples were checked and executed; evidence is retained in
`archive/testing-truthiness-start/catalog-examples.json`.
Matched expectation benchmark support and complete family measurements remain
pending; this is focused implementation progress rather than completed parity.
The API behavior follows the [Vitest matcher reference](https://main.vitest.dev/api/expect).

Truthiness editor acceptance: hover and completion include both methods and behavioral documentation; signature help exposes zero parameters. The focused LSP check passes.


## Nullish matcher progress

Zero-argument `toBeNull()`, `toBeUndefined()` and `toBeDefined()` now use
predicates captured before actual transfers into the identity matcher. Null is
defined; only undefined fails `toBeDefined`. Falsy defined values, including
false, zero, NaN and empty strings, pass. The object remains reusable without
cloning data. Assertions recover at the existing test boundary; wrong arity is
rejected before native compilation. All three documented examples execute with
exact expected output; evidence is retained in
`archive/testing-nullish-start/catalog-examples.json`.

The expectation's existing Object.is closure still prevents nullable-union and
callable actual types even when only a predicate matcher is used. Removing that
restriction requires type-directed identity support or a matcher representation
that checks each invoked method separately; the restriction is tracked rather
than described as full nullish parity. Benchmark support and family measurements
remain pending. Semantics follow the [Vitest matcher reference](https://main.vitest.dev/api/expect).

Nullish focused acceptance: all 40 testing compiler/native checks and six bundled testing editor checks pass. The nullish editor check covers hover documentation, completion and zero-argument signature help for all three methods. No performance measurements are claimed.


## Nullable expectation support

Object.is now dispatches on active data-union alternatives. Numeric payloads use
SameValue rules, including NaN equality and signed-zero distinction; matching
integer payloads retain signed 64-bit precision. Strings compare text. Array and
record alternatives compare payload storage identity, including borrowed union
views. Null and undefined compare only with themselves. Both operands evaluate
once, left to right, with no cloning. Callable alternatives and void remain
unsupported and are rejected before emission.

This removes the prior nullable-union restriction from strict assertions and
expectations, including predicate-only expectations. Owned actual values still
transfer into their expectation; inspect any needed branch predicates before that
transfer. The prior nullish limitation paragraph records the earlier state and
is superseded by this implementation. All 41 testing checks and three Object.is
checks pass. A focused nullable expectation completion/hover check passes. The
new Object.is catalog example executes with exact output, retained under
`archive/testing-union-identity-start/`. Family parity and performance acceptance
remain open.

Supplemental untimed scalar assertion ABI/oracle check: 10 native/compiled-Node cells pass with current compiler input fingerprints. Evidence: `archive/testing-union-identity-start/assertion-benchmark-correctness.json`. This validates existing workload support and is not union-specific performance evidence. The installed optimized CLI is updated.


## Expectation benchmark support and scalar-union specialization

`testing_expectation_predicates` exercises actual stored std:testing expectations
across null, undefined, zero, NaN, negative zero and seven. Each iteration checks
nullish state, truthiness and SameValue identity, then observes the active value
category in a checksum. The TypeScript counterpart uses actual Node strict
assertions as a semantic baseline; this is not a Vitest latency comparison.
Whole-workload costs include matcher construction, copied predicate captures,
tagged union dispatch, assertion calls and observation.

Matched sources, manifest registration, exact fingerprints, real-API guards,
independent BigInt category oracle and ordinary native/TypeScript ABI drivers
are ready. Ten representative native/compiled-Node cells pass with zero timing
samples. Four assertion/expectation harness checks pass, including hand-computed
oracle cells and source mutation rejection. Evidence is retained in
`archive/testing-expectation-benchmarks-start/correctness.json`. Original sample
counts and timing protocols remain unchanged; no performance measurement is
claimed. Runner/hook counterparts remain pending before the family milestone.

The workload exposed an additional prerequisite: generic ReadOnly<T> callable
parameters must normalize to copy for scalar-only union specializations, as they
already do for individual scalars. Structured substitution now handles grouped
scalar unions without relaxing explicitly declared ReadOnly scalar contracts.
The editor mirrors this normalization. All 42 testing checks and the focused
copy-union matcher signature-help check pass. Owned data unions retain their
read contract. This supplements the earlier nullable expectation evidence.


## Negation representation prerequisite and NaN matcher

Full `.not.toBe()` remains active work. Current expectations store actual in one
owned closure. Returning a second identity closure would transfer the same owner
twice; cloning actual would change array/record identity. Callable environments
also cannot be cloned. A shared actual representation or compiler-supported
matcher receiver methods must therefore precede complete negation. Predicate-only
negation would leave the main identity API unsupported and is not claimed as
negation parity.

`toBeNaN()` is implemented independently using a copied SameValue predicate
captured before the actual owner transfers. Numeric NaN passes; strings including
"NaN", integers, null and other data alternatives fail without coercion. Reused
expectations preserve their actual value. Failure remains recoverable at the test
boundary. Zero-argument completion, hover and signature help are documented and
verified. The expectation workload now checks NaN explicitly instead of identity
for its NaN category, retaining exactly three assertions per iteration. Matched
Node checks use Number.isNaN with Node assertions; fingerprints and registrations
are updated. Ten untimed native/compiled-Node cells pass. Original workload sources
and evidence are retained in `archive/testing-nan-start/` and earlier archives.

NaN acceptance evidence: all 43 testing checks pass; the expanded focused NaN check also covers undefined, infinity, arrays and records. The editor predicate help check, catalog example and two expectation harness checks pass. No timing measurements were performed.


## Shared comparison callback foundation for negation

The Rust backend adapter `TsrSharedReadCallback<C>` retains one immutable callback
environment through reference-counted handles. Its Clone implementation clones
only the handle and does not require C or any capture to implement Clone. It
forwards readonly invocation, including supplied-argument counts when the generated
callback traits use that protocol. A mutable-call interface delegates to readonly
invocation; it does not enable mutation of the shared environment. The final
handle drops the original environment exactly once.

Four focused native checks pass: noncloneable capture identity and exact disposal,
optional argument count preservation, compatibility with uncounted callback traits,
and an actual compiler-generated owned array comparison closure. The last check
shares the generated environment, observes identical payload storage through both
handles, drops one handle, and confirms remaining identity checks still work.
No actual array/record cloning, unsafe casts, global matcher registry or foreign
runtime is introduced. Helpers remain under `src/stdlib/testing/` as Rust files.

This is backend foundation, not exposed `.not` support. The compiler still needs
an explicit shared immutable callable storage contract before the adapter can be
used by checked module declarations. Next implementation steps are:

1. Represent shared callable storage separately from ordinary owned mutable/once
   storage. Admit only reusable readonly environments without borrowed captures;
   reject mutation, consuming invocation and escaping borrowed environments.
2. Lower shared construction and handle cloning to this adapter. Preserve concrete
   argument/result contracts, generic specialization, ownership and argument counts.
3. Build positive and negative matcher objects around cloned comparison handles,
   retaining one actual owner. Verify direct and stored `.not.toBe()` calls for
   scalar, nullable and container values, failure continuation and disposal.
4. Complete negated matcher documentation/editor coverage and matched benchmark
   guards/oracles before closing negation acceptance. No new source API or public
   sharing primitive is claimed by the backend-only checks.

Native helper and acceptance-test hashes are retained in
`archive/testing-shared-callback-start/acceptance.json`. No performance measurement
or completed testing-family milestone is claimed.


## Shared callable storage admission

Compiler storage contracts now distinguish ordinary owned storage from explicitly
shared readonly storage. Ordinary `checked()` admission still rejects borrowed
Read signatures. Shared admission is a separate internal operation and requires
an explicit readonly signature plus an owned reusable readonly environment.
Consuming invocation, mutable captures and borrowed captures produce existing
source diagnostic categories. Full family admission additionally validates
concrete layout, parameter/result types and argument access exactly.

The canonical registry can intern shared signatures separately from ordinary
owned signatures while retaining source declaration provenance. Origin admission
uses shared validation before committing any candidate. Failed mutable, borrowed
or missing origins leave the registered origin set unchanged; repeated valid
origins remain deduplicated. Existing compaction and ownership behavior are
preserved. These operations are internal groundwork and do not expose a public
sharing primitive or permit normal borrowed callbacks to escape.

Nine callable storage checks, five native callable dispatch checks, four native
shared-adapter checks and all 43 testing checks pass. Exact changed source
snapshots and current hashes are retained in
`archive/testing-shared-storage-start/`. The next work is native shared-family
lowering and checked construction/handle cloning, followed by positive/negative
matcher integration. `.not` remains pending; no completed family milestone or
performance measurement is claimed.

Readonly native family lowering also passes a focused generated-code check at optimization levels 0 and 3. Shared families emit immutable call receivers, so handles can invoke without interior mutation. All six callable dispatch checks pass after this change. Native construction/handle-clone compiler expressions and public negation integration remain next; no public `.not` support is claimed. Supplemental evidence is retained in `archive/testing-shared-storage-start/readonly-emission.json`; the original admission record is unchanged.


## Shared native family construction and cloning

Shared readonly storage families now emit a reference-counted outer handle around
one concrete or tagged environment. Each admitted origin has a typed constructor.
The handle's Clone implementation clones only the shared callback handle; it never
clones the environment or captured actual. Readonly calls delegate through the
shared environment. Both ordinary and supplied-count calls preserve their argument
protocol. ReadCallback and Callback adapters expose the readonly implementation
when callback traits are needed.

The production emitter recognizes shared declarations, includes the Rust ownership
helper once, and lowers callable-storage construction through shared-aware origin
constructors. Ordinary owned and once constructors retain their existing layout.
Default/supplied callback conversion sites use the same constructor selection.
A checked IR fixture validates this production path, including returned shared
handles, cloning, invocation after an original handle drops and passing the handle
to a generated readonly callback parameter. The fixture explicitly registers a
shared contract after source checking; it does not claim frontend source syntax or
private bridge support is implemented.

Native checks cover multiple admitted origins, singleton construction, both
optimization levels, supplied-count dispatch through cloned handles and existing
owned/once dispatch behavior. All 43 testing checks and four shared-adapter checks
remain passing. Exact originals and source hashes are retained under
`archive/testing-shared-lowering-start/`. The next step is checked frontend
construction and cloning through the private testing bridge, with ownership
metadata that distinguishes shared owned readonly environments from borrowed Read
callbacks. Public `.not` remains pending and no performance milestone is claimed.


## Owned readonly shared callback metadata

Internal shared storage now carries a distinct `SharedRead` callback kind. It owns
its environment, invokes through readonly access, and can lend ordinary reusable
callback views. Borrowed readonly callbacks and mutable owned callbacks cannot
implicitly become shared storage. Admission still rejects borrowed, mutable and
consuming environments before native emission. No public source wrapper is added.

The checker permits cloning a direct shared stored handle, retaining one captured
environment. Ordinary stored callbacks, callback parameter views and callable
containers retain their existing clone restrictions. The private frontend sharing
bridge and public `.not` matcher remain pending; this metadata closes an internal
ownership distinction without claiming source-level sharing is available.

Acceptance: 55 focused callable unit checks, one callback-kind ownership check,
18 callback regressions, four native shared-adapter checks and all 43 testing
checks pass. The LSP builds successfully. Exact originals and current hashes are
retained in `archive/testing-shared-ownership-start/`. No performance measurements
or completed feature-family milestone are claimed.


## Shared storage through source checking

A source-checking acceptance fixture now installs an internal shared field
contract before running the ordinary analyzer. Source closure construction moves
an owned string environment into that field, `clone(holder.action)` creates a
shared handle, and both original and cloned handles invoke successfully in native
execution. This covers the normal checker expression path rather than appending
handwritten Rust clone calls after checking. Contract installation remains internal;
no public sharing syntax or private import bridge is claimed.

The fixture first exposed a panic in callback compatibility: shared owned callbacks
were incorrectly validated through ordinary owned storage. Compatibility now
selects shared storage admission for shared contracts. Focused source tests reject
mutable and borrowed environments with the appropriate diagnostics. The existing
three callback compatibility unit checks pass. Exact originals and source hashes
are retained in `archive/testing-shared-checker-start/`. Public `.not` and the
private generic sharing bridge remain pending. No performance measurement is claimed.


## Private sharing intrinsic checker integration

`std:testing-runtime` now registers a private polymorphic `shareCallback` intrinsic.
Direct calls check the source callback, move its environment, infer a shared
readonly storage contract and lower to the existing shared constructor. The
intrinsic emits no runtime foreign-language call. Ordinary source imports of
this private module remain rejected. Shared callback type hints defer to concrete
checking rather than attempting a scalar native result conversion.

A focused source AST fixture resolves the intrinsic's internal symbol, constructs
a handle capturing an owned string, clones that handle with the source `clone`
expression and invokes both handles through generated native code. This check
first failed with unknown intrinsic resolution, then exposed an invalid scalar
native type-hint conversion; both paths are repaired. All 43 testing checks pass
and the LSP builds. Exact originals are retained in
`archive/testing-sharing-bridge-start/`.

The current intrinsic requires concrete closures and fixed required parameters.
Optional/rest callback construction, first-class use of the intrinsic and
polymorphic callback environments remain unsupported. First-class use reports a
diagnostic. Bundled generic matcher integration and public `.not` remain next;
the direct checker fixture alone does not prove those paths. No benchmark or
feature-family completion is claimed.


## Bundled negation integration in progress

`expect(actual).not` now provides negated identity, truthiness, falsiness, null,
undefined, defined and NaN matchers. The generic bundled implementation constructs
one readonly shared comparison callback retaining actual, clones its handle and
captures separate handles in positive and negative identity matchers. No actual
payload is cloned. Predicate state is captured before actual transfers. A native
source check verifies repeated positive and negative string identity calls and
all seven negated matcher names. All 44 testing checks pass after integration.

The IntelliSense catalog now documents the negated view and its seven methods
with executable examples. Editor completion/help checks, example execution,
negative failure recovery and ownership cases, benchmark support updates and
installed CLI refresh remain pending acceptance work. Chained `.not.not` requires
further representation support and is not implemented. Exact originals are
retained in `archive/testing-negation-start/`. No completed gate or performance
measurement is claimed at this checkpoint.


Negation supplemental acceptance: editor completion exposes all seven negated
methods; hover uses negative condition documentation and parameter help preserves
the expected argument. Eight catalog examples run successfully in generated
native execution. A runner fixture verifies all seven failing negated matchers
stop their test body and subsequent tests continue, including NaN identity and
signed-zero negated equality. Array/record and union ownership checks, benchmark
updates and installed CLI refresh remain pending.


Negation ownership and workload acceptance: arrays and records compare identity
through repeated negated calls while expected owners remain available afterward.
Nullable string unions retain correct negated nullish behavior. Invalid arity,
expected types, reuse of transferred actual arrays and chained `.not.not` reject.
All four focused negation tests pass.

The existing expectation workload now executes six matched positive/negative
assertions per iteration across null, undefined, zero, NaN, negative zero and
seven. Both actual API counterparts, exact hashes, guard expressions, operation
counts and manifest registration are updated. The independent checksum oracle
is unchanged because observed values are unchanged. Ten native/compiled-Node
quick cells pass; no timings are collected. Original workload snapshots and
current correctness evidence remain under `archive/testing-negation-start/`.

The verified release CLI is installed at `/Users/seancotter/.cargo/bin/tsrust`.
An installed-binary execution of the updated expectation workload returns
859908182. Source API, editor and native negation checks are accepted. Chained
negation, structural equality and remaining parity features stay pending; full
feature-family regressions and performance measurement remain separate work.


## Runtime type matcher implementation in progress

`toBeTypeOf(expected)` and its negated counterpart compare the captured runtime
`typeof` category without coercion. A focused native test covers integer and
number including NaN, boolean, string, null, undefined, arrays, records and
nullable string unions. Both methods borrow expected strings; repeated checks
preserve expected owners and the retained identity matcher. The actual payload
is never cloned; only the small captured type-name string is cloned between
positive and negative views.

The API follows Vitest's typeof behavior, including null as object (reference:
https://main.vitest.dev/api/expect.html#tobetypeof). Expected is currently string
because string-literal unions are unavailable. Callable actual values remain
unsupported by the existing expectation contract. Focused native acceptance
passes; catalog documentation is present. Failure recovery, wrong arity/type
diagnostics, editor checks, examples, benchmark support and installed CLI refresh
remain pending. Exact originals are retained in `archive/testing-typeof-start/`.
No complete feature gate or performance measurement is claimed.


Runtime type matcher acceptance completed: both positive and negative failure
conditions stop their test bodies, later registered tests continue, and invalid
arity/types reject. Completion, hover and parameter help pass for both views.
Two catalog examples execute natively. All 49 testing checks pass. The matched
expectation workload now performs eight successful assertions per iteration,
including both runtime-type views; exact source hashes, guarded expressions,
operation counts and registrations are current. Ten independent-oracle
native/compiled-Node quick cells pass without timings. The verified release CLI
is installed. Evidence and exact originals remain under
`archive/testing-typeof-start/`; broader parity and family measurements remain
pending.


## Numeric matcher prerequisite: private numeric observation

A private `numericValue` compiler intrinsic now observes supported actual values
without consuming or coercing them. Numeric alternatives return their numeric
value; nonnumeric alternatives return NaN as an internal sentinel. Public matcher
construction will separately retain a numeric-type predicate so unsupported actual
values fail even under negation. This sentinel is not a public numeric conversion
API and does not silently add coercion semantics.

The checker borrows its argument and the emitter handles plain scalars, copy unions
and borrowed data unions directly without foreign-language calls. Two generated
native tests cover nullable numeric unions, mixed string/number/null unions,
integers, booleans, arrays and records. Expected data owners remain usable after
observation; strings such as "42" and booleans remain nonnumeric. The mixed-union
test exposed a missing dereference in borrowed numeric alternatives, now repaired.
The LSP builds with private observation metadata. Exact originals remain under
`archive/testing-numeric-extraction-start/`. Public relational and proximity
matchers, their documentation and benchmarks remain pending. No completed public
feature or measurement is claimed.


## Relational numeric matcher implementation in progress

`toBeGreaterThan`, `toBeGreaterThanOrEqual`, `toBeLessThan` and
`toBeLessThanOrEqual`, plus their negated forms, now compare captured numeric
state against a number threshold. Both views require a numeric active actual
alternative and fail for nonnumeric values before checking the relation. NaN
makes positive relations false; negations succeed. Signed zeros compare equal
and infinities follow normal numeric ordering. Native int actuals convert to
number, with rounding possible beyond exact number precision; bigint remains
unsupported.

A focused generated native fixture passes all eight methods, equality boundaries,
NaN, infinity, signed zero and nullable numeric union cases. Complete catalog
documentation and executable examples are present. Nonnumeric failure recovery,
argument diagnostics, editor checks, example execution, matched workload updates
and installed CLI refresh remain pending. Exact originals remain under
`archive/testing-numeric-matchers-start/`. No completed public gate or performance
measurement is claimed.


Relational numeric matcher acceptance completed: all four positive NaN failures
and all nonnumeric failures in both views recover within the runner; later tests
continue. Invalid arity and expected types reject. All eight completion/hover/
parameter-help combinations pass. Eight catalog examples execute natively. The
prior 50-test testing pass remains evidence, and the new recovery/diagnostic test
passes separately. The matched workload now performs sixteen assertions per
iteration, including all four relations and negations on the category index.
Exact sources, counterpart hashes, guarded expressions, operation counts and
registration are current. Ten native/compiled-Node quick cells pass without
timings. The installed release CLI executes the workload with checksum 859908182.
Evidence remains in `archive/testing-numeric-matchers-start/`. Broader structural,
collection, proximity, selection and compiler-dependent parity remain pending;
no full family regression or performance milestone is claimed.


## Proximity matcher implementation in progress

`toBeCloseTo(expected, numDigits?)` and its negated view now use absolute tolerance
`10 ** -numDigits / 2` with a strict comparison and default precision 2. Matching
infinities succeed directly; NaN and opposite infinities do not match. Negative
and fractional precision are supported. Supported actual values must be numeric
in both views; int actuals convert to number with the documented precision limit.
The formula and infinity behavior follow Vitest's implementation reference:
https://github.com/vitest-dev/vitest/blob/main/packages/expect/src/jest-expect.ts

A focused native check covers defaults, explicit precision, negative/fractional
precision, exact tolerance boundary, floating addition, NaN, infinities and signed
zero. It exposed callable-storage admission using a default parameter's body type
rather than accepted input type; admission now uses `input_ty` where present.
Catalog documentation and examples are present. Failure recovery, explicit
undefined/default protocol checks, editor acceptance, matched workload support,
related compiler regressions and installed CLI refresh remain pending. Originals
are retained in `archive/testing-close-to-start/`. No completed gate or performance
measurement is claimed.


Proximity acceptance completed: explicit undefined restores default precision;
NaN and positive/negative infinite precision follow tolerance rules; exact
boundaries and failures recover without skipping later tests. Wrong argument
types and arities reject. Both editor views expose completion, hover and optional
precision help. Two documented examples execute natively. All 53 testing checks
pass; the nine callable dispatch checks passed after the input-type admission
fix. The expectation workload includes two proximity checks, updates operation
count to eighteen assertions per iteration, and retains exact guarded native/Node
source hashes and independent checksum observation. Ten matched quick cells pass
without timings. The installed release CLI runs this workload with checksum
859908182. Exact originals and evidence remain in `archive/testing-close-to-start/`.
Structural, collection, selection and compiler-dependent parity remain pending;
full feature-family regression and performance measurement are separate work.


## Length matcher implementation in progress

`toHaveLength(expected)` and its negated view now compare captured length against
a number. Strings count UTF-16 code units for API parity, arrays count elements,
tuples count slots, and records with concrete int/number length properties use
that property. Unions dispatch on their active alternative. Unsupported actual
values fail in both views; union-typed record length properties remain pending.
Actual payloads are not cloned. The private observation bridge evaluates an
argument once and borrows it.

A focused native fixture verifies a supplementary-plane emoji counts as two,
repeated positive/negative calls retain identity checks, arrays and tuples,
fractional record length and nullable strings. Reusing the ordinary native string
length initially exposed byte counting; this matcher now counts UTF-16 directly
without changing ordinary string operations. Complete catalog docs/examples are
present. Failure recovery, diagnostics, editor checks, documented example execution,
matched workloads and installed CLI refresh remain pending. Exact originals are
retained in `archive/testing-length-start/`. No completed gate or performance
measurement is claimed.


Length acceptance completed for the documented source types: failures and
unsupported values recover in the runner, invalid arguments reject, both editor
views expose completion/hover/parameter help, and two documentation examples
execute natively. All 55 testing checks pass. The matched expectation workload
now performs twenty assertions per iteration, including positive and negated
UTF-16 length checks on a category-selected emoji or ASCII string. Guarded
expressions, exact hashes, operation counts and registration are current. Ten
native/compiled-Node quick cells pass without timings. The installed release CLI
is refreshed. Originals and acceptance evidence remain under
`archive/testing-length-start/`. Union-typed record length properties, containment,
structural comparison and other parity remain open; no completed feature-family
regression or performance milestone is claimed.


## Union-typed record length acceptance

Length observation now dispatches through the active alternative of record length
properties. Numeric alternatives are supported, including NaN; nonnumeric
alternatives fail in both views without coercion. The focused native fixture
first failed on numeric alternatives and now passes numeric/NaN cases, rejects
string/undefined alternatives and continues subsequent tests. All three focused
length tests pass. Existing length editor checks pass against updated docs, and
two new union-property examples execute natively. The previous 55-test full
testing pass remains evidence; this additional fixture is separate acceptance.

The matched expectation workload adds positive and negated union-record length
checks, with twenty-two assertions per iteration. Exact source hashes, guarded
expressions, operation counts and registrations are current. Ten native/compiled-
Node quick cells pass without timings; the installed release CLI returns checksum
859908182. Exact originals and evidence remain under
`archive/testing-union-length-start/`. The previously documented union-property
limitation is resolved. Containment, structural equality, selection and other
parity remain open; no full family measurement is claimed.


## Containment parameter specialization prerequisite

An internal annotation projection now specializes a containment expected parameter
from an actual container type. Arrays remove one rank, strings retain string,
nonempty tuples form an element union, and data unions specialize their
alternatives. Scalar-only results copy; data-owning results retain borrowed
parameter access. Annotation provenance remains inherited from the actual type.
The marker is unspellable in user source and does not expose a new public
ElementOf type constructor. Ordinary constructor annotations remain unchanged;
invalid internal projection arity rejects.

Five type-substitution tests pass, including focused projection checks for scalar
arrays, nested arrays, strings, tuple unions and nullable arrays. Existing
substitution ownership/provenance, constructor diagnostics and nesting checks
remain passing. Exact originals are retained under
`archive/testing-containment-types-start/`. This is annotation transformation
groundwork only: bundled marker resolution, generic declaration validation,
empty-tuple and nominal tuple handling, shared actual comparison dispatch, public
containment methods, editor behavior and benchmark support remain pending. No
public containment API or completed gate is claimed.


## Containment projection generic validation

Generic function and record declaration validation now recognize the internal
containment projection and validate its operand instead of treating it as an
unknown generic record constructor. Exactly one type argument is required.
Unknown operand types still reject, and ordinary source ElementOf constructors
remain unsupported. Existing type-parameter constructor rules retain precedence
for ordinary annotations.

A production analyzer/native fixture installs the private marker in a generic
record callback parameter, specializes an int-array actual to an int expected
parameter, constructs the callback object and invokes it successfully. This
fixture first failed with unknown generic record validation, then passed after
the validation integration. A separate lexical validation check passes valid
operands and rejects unknown types, wrong arity and ordinary constructors. Exact
originals are retained in `archive/testing-containment-validation-start/`.

Bundled source marker resolution, nominal/empty tuple rules, mixed comparison
argument dispatch sharing one actual environment, public containment APIs, editor
surfaces and benchmark support remain pending. The fixture installs private AST
metadata directly; it does not prove bundled source marker resolution or public
containment. No completed public gate or performance measurement is claimed.


## Bundled containment projection source linking

The project linker now recognizes ContainmentValue<T> only in the bundled
std:testing source scope and rewrites it to the unspellable internal element
projection. Ordinary modules retain ordinary type constructor resolution. Lexical
type parameters and declared/imported names can shadow the internal source name;
source coordinates and operand names retain their usual mapping. No public
ElementOf or ContainmentValue constructor is exposed.

Two linker annotation checks pass, covering ordinary scope, bundled scope,
shadowing and coordinate preservation. A separate synthetic std:testing module
graph fixture passes through production project linking, generic validation,
record specialization, callback construction and generated native execution. The
fixture uses a small private record/function module with a generic containment
parameter and emits 7. It does not alter the actual bundled API or claim public
matcher support. Originals and evidence remain under
`archive/testing-containment-linking-start/`.

Shared actual comparison dispatch, nominal/empty tuple handling, public
containment implementations, editor signatures and benchmark support remain
pending. No completed public gate or performance measurement is claimed.


Containment specialization supplemental evidence: a second production analyzer/
native fixture specializes a string-array actual to a borrowed string expected
parameter. Repeated invocation preserves the expected owner, verified by printing
it afterward. Both generic record projection fixtures pass. An additional
substitution check proves projected element coordinates come from the supplied
actual annotation, preserving diagnostic provenance. Public matcher and nominal/
empty tuple integration remain pending; these checks strengthen prerequisite
ownership evidence without claiming public containment.


## Private containment runtime comparison

A private `containsValue(actual, expected)` compiler bridge now supports strings
and matching primitive array element types. String comparison searches substring
text, including empty needles and supplementary-plane Unicode. Array comparison
reuses native indexOf strict equality, so NaN does not match and signed zeros
match. Both arguments are borrowed and evaluated through the existing search
paths; actual and expected owners remain usable afterward. Private observed-value
checking now validates each binding's declared arity instead of assuming every
observation takes exactly one argument.

A focused production analyzer/native fixture first failed on unknown bridge
resolution, then passed strict NaN/zero cases, absent elements, Unicode substring
matching, empty needles and owner reuse. The bridge currently explicitly rejects
union expected parameters, nested/data array elements and tuple containers; no
silent comparison fallback is used for unsupported shapes. Shared actual
comparison dispatch, these additional container shapes, public containment
methods, editor acceptance and benchmark support remain pending. Exact originals
are retained in `archive/testing-containment-runtime-start/`. No public matcher
implementation or completed feature gate is claimed.
## Private containment over primitive union arrays

The private `containsValue` bridge now accepts primitive union arrays with a
scalar expected value. It uses the existing strict array-search comparison:
NaN does not match, signed zeros match, strings compare text, and different
active alternatives do not match. Both arguments remain borrowed. Unsupported
data alternatives and union-typed expected values remain rejected rather than
silently returning false.

A focused failing compiler test first rejected the union array. After the
checker change, four containment-focused compiler tests pass, including native
execution of mixed number/string/null arrays and reuse of borrowed owners.
The exact original checker source is retained in
`archive/testing-containment-scalar-unions-start/analysis.rs`.
This is internal acceptance only; public `toContain` and `toContainEqual`,
editor documentation, public API benchmarks, and union-valued expected dispatch
remain pending. No performance measurement was performed.
