# Tsrust

An experimental TypeScript-familiar language with inferred ownership and native compilation through Rust. The name, extension, syntax subset, and ownership rules remain provisional.

Source files use `.tsr`. The compiler is written in Rust. Its reusable analysis library handles parsing, type checking, ownership inference, diagnostics, and Rust generation. The CLI invokes `rustc` only when building native executables. The Rust language server in `crates/tsrust-lsp` shares this library; its [VS Code extension](./editors/vscode/README.md) supplies highlighting, project-wide diagnostics and IntelliSense, including unsaved source and dependencies.

`main` is optional. Files containing only declarations, and empty files, can be
checked, built and run. Running without `main` exits successfully without calling
any declared functions. When present in the entry file, `function main()` runs
with no arguments and must return no value. A dependency's `main` is not invoked.


The compiler, acceptance tests, and handwritten-witness checks are all Rust. Cargo runs the complete verification suite.

[Rest parameters](./docs/rest-parameters.md) collect remaining arguments into fresh owning arrays. Focused compiler, native and editor acceptance covers current homogeneous data and callable kinds.

[Call spread](./docs/call-spread.md) expands dense homogeneous arrays into call arguments. Focused compiler, native and editor acceptance covers current callable and built-in targets; array-literal and object spread remain separately scoped.

[Recursive records](./docs/recursive-records.md) support owned array and nullable links, including shared lifetimes for recursive byte-view storage.

## Start a project

```sh
tsrust new hello
cd hello
tsrust dev
```

Development rebuilds and restarts the native program on source changes. Failed
builds keep the last working program running. Use `tsrust build`, `run`, `check`,
`test` and `format` from the project or its subdirectories. See the
[project CLI guide](docs/guides/project-cli.md) for configuration and supported boundaries.

## Repository layout

```text
Cargo.toml                       Workspace definition
Cargo.lock                       Reproducible dependency resolution
rust-toolchain.toml               Pinned Rust release and development components
crates/
  tsrust-compiler/
    Cargo.toml                   Compiler library and tsrust binary
    src/                         Compiler implementation and CLI
    tests/
      analysis.rs                In-process analysis tests
      acceptance.rs              CLI and native-output tests
      integers.rs                Integer semantics and native-output tests
      operators.rs               Scalar operators, evaluation order, and overflow checks
      string_operators.rs        Native string equality, ordering, and operand loans
      arrays.rs                  Scalar array ownership, indexing, and bounds checks
      owned_arrays.rs            Owned elements, borrowing, cloning, and iteration
      nested_arrays.rs           Nested types, parent loans, mutation, and depth
      array_fields.rs            Owned fields, mixed storage paths, and recursive records
      array_search.rs            Primitive includes/indexOf, loans, offsets, and resource models
      array_removal.rs           Owned extraction, stable selections, and bounds checks
      nested_records.rs          Inline nested records, layout validation, and field loans
      primitive_storage.rs       Array growth, layout, string lifetime, and cleanup
      recursive_storage.rs       Deep clone/cleanup, recursive components, and field moves
      fixed_storage.rs           Proved literal storage, fallback, and bounded specialization
      field_moves.rs             Partial availability, restoration, joins, and owned fields
      recursion.rs               Recursive contracts, captures, and native behavior
      recursive_callbacks.rs     Recursive callback forwarding and type instances
      recursive_owned_callbacks.rs Owned recursion, retention, and specialization
      recursive_callable_results.rs Recursive factories and definitive result origins
      for_of.rs                  Borrowed iteration, scope, and loop ownership checks
      control_flow.rs            Branch ownership and boolean tests
      loops.rs                   Iteration ownership, break, and continue checks
      mutable_bindings.rs        Local replacement and ownership reinitialization checks
      capture_replacement.rs     Writable captures and environment state checks
      mutable_callables.rs       Concrete callable replacement and transfer checks
      callable_moves.rs          Declaration moves and returned origin checks
      callback_moves.rs          Callback contract and local borrow transfer checks
      callback_returns.rs        Direct owned returns and single-use preservation checks
      witnesses.rs               Handwritten Rust reference checks
      closures.rs                Local and returned closure acceptance tests
      closure_witnesses.rs        Independent closure lowering checks
      callbacks.rs               Synchronous reusable callback checks
      single_use_callbacks.rs    Single-use callback ownership checks
      closure_arguments.rs       Scalar closure argument checks
      owned_closure_arguments.rs Owned argument access checks
      nested_closures.rs         Nested local capture and invocation checks
      callable_captures.rs        Concrete callable capture and lifetime checks
      callable_results.rs         Recursive callable result checks
      retained_callbacks.rs       Owned callback retention and specialization checks
      support/                   Shared helpers and typed case expectations
      fixtures/ownership/        Original .tsr and .rs case pairs
      fixtures/integers/          Original numeric User example
      fixtures/operators/         Retained counter with arithmetic and comparisons
      fixtures/arrays/            Array sum and boolean element updates
      fixtures/owned-arrays/      Owned records and borrowed iteration
      fixtures/nested-arrays/     Nested mutation, iteration, and clone independence
      fixtures/array-fields/      Record collections and array-field clone independence
      fixtures/array-removal/     Ordered record removal and independent ownership
      fixtures/nested-records/    Inline profile records, mutation, and cloning
      fixtures/field-moves/       Detached nested fields and restored whole records
      fixtures/recursion/         Recursive owned-tree traversal and mutation
      fixtures/recursive-callbacks/ Borrowed callbacks through recursive traversal
      fixtures/recursive-owned-callbacks/ Owned state through recursive moves
      fixtures/recursive-callable-results/ Recursive returned counters
      fixtures/for-of/            Borrowed iteration with break and continue
      fixtures/control-flow/     Branch mutation and reuse example
      fixtures/loops/             Counter with break and continue
      fixtures/mutable-bindings/  Repeated ownership transfer and reinitialization
      fixtures/capture-replacement/ Independent returned counters
      fixtures/mutable-callables/ Reinitialized single-use environments
      fixtures/callable-moves/    Returned callable declaration chains
      fixtures/callback-moves/    Retained state through moved owned parameters
      fixtures/callback-returns/ State retained through direct owned returns
      fixtures/closures/         Closure source and handwritten Rust witnesses
      fixtures/callbacks/        Capture lifetime, ownership, and argument examples
    examples/analyze_bench.rs     In-process analysis benchmark
    examples/field_move_bench.rs  Partial field state and branch-restoration benchmark
    examples/recursion_bench.rs   Recursive access-inference benchmark
    examples/recursive_callback_bench.rs Callback inference and type-vector benchmark
docs/                            Design, milestones, and experiment notes
```

The library and CLI stay in one crate while they share a release cycle. The language server lives in a separate crate and depends on the shared compiler library.

## Edit in VS Code

The [local extension setup](./editors/vscode/README.md) covers building `tsrust-lsp`, packaging/installing the VSIX, and configuring the server path. [Editor acceptance](./docs/editor-foundation-acceptance.md) records tested capabilities and remaining compiler handoffs.

## Try the ownership prototype

Requirements: Rustup with Rust 1.96.1, rustfmt, and Clippy installed. The workspace pins that release. The compiler crate has no external dependencies; the language-server crate adds pinned protocol/JSON dependencies. Run `cargo fetch --locked` once to populate them before offline workspace builds. `tsrust check/emit` need no external compiler; native `build/run` commands explicitly select Rust 1.96.1 through Rustup. See the [setup and upgrade policy](./docs/rust-toolchain-policy.md).

Run these commands from the project root:

```sh
cargo build --offline
./target/debug/tsrust check crates/tsrust-compiler/tests/fixtures/ownership/transitive_mutation.tsr
./target/debug/tsrust run crates/tsrust-compiler/tests/fixtures/ownership/clone_independence.tsr
./target/debug/tsrust check crates/tsrust-compiler/tests/fixtures/ownership/use_after_call.tsr
```

After the build, `check` reports inferred parameter access. `run` prints `Pat` and `Sam`, showing independent values after explicit cloning. The final command intentionally fails with a source diagnostic identifying a transfer and the later invalid use.

To inspect generated Rust or create a native executable:

```sh
./target/debug/tsrust emit crates/tsrust-compiler/tests/fixtures/ownership/read_reuse.tsr
./target/debug/tsrust build crates/tsrust-compiler/tests/fixtures/ownership/read_reuse.tsr -o /tmp/tsrust-demo
/tmp/tsrust-demo
./target/debug/tsrust build crates/tsrust-compiler/tests/fixtures/ownership/read_reuse.tsr --release -o /tmp/tsrust-demo-release
```

`build` and `run` use an unoptimized backend by default for faster development builds. Add `--release` to compile generated Rust at optimization level 3, matching the runtime benchmarks. Both profiles enable overflow checks and retain source-level arithmetic and bounds diagnostics. Building the compiler itself with `cargo build --release` does not select the generated program's profile; pass `--release` to `tsrust` separately.

## Verify

```sh
cargo test --offline
cargo fmt --all --check
cargo clippy --offline --all-targets -- -D warnings
```

`cargo test` covers 509 integration groups for compiler semantics, ownership, native execution, storage boundaries, and cleanup. Test helpers enforce subprocess timeouts and remove temporary executables.

## Scope and next work

The current compiler supports primitive array `includes`/`indexOf` with optional signed offsets, one file, named records with scalar, string, array, and named-record fields, strings, standard 64-bit floating-point numbers, signed 64-bit integers, booleans, owned scalar/string/record arrays of exact nested types, named calls with self and mutual data/callback recursion, local bindings, field replacement, explicit cloning, and owned returns. Braced `if`/`else`, `else if` chains, and standalone blocks support nested branches, local scope, and early returns. It infers read, mutate, or consume access for owned parameters; number, integer, and boolean parameters use a copy contract. A value remains available after a branch only when every continuing path retains it. Closures with zero or more typed scalar, string, record, and array arguments support local captures, repeated or consuming calls, and returned owned environments. Arrow bodies can create and invoke local nested closures, including local factory results and synchronous callbacks. Capture effects propagate through enclosing bodies. Capture storage, invocation access, argument access, and lexical depth are separately inspectable. Arrow bodies infer owned argument access; callback and factory signatures require explicit `read`, `mutate`, or `consume` modifiers on owned parameters. Functions accept synchronous reusable or single-use callbacks, including named closures, inline arrows, and factory results; callback parameters can be forwarded without escaping, including through self and mutual recursion with non-callable results. Recursive callback functions allow at most 128 distinct concrete callback type vectors each. The provisional `Once<Function<Result, [n: int]>>` annotation transfers the environment and permits at most one invocation. Closures also capture concrete callable environments and return other owned closures. Explicit `Function<Result, [...]>` and `Once<Function<Result, [...]>>` parameters permit retained callbacks through returned wrappers. Owned callback calls, including self and mutual recursion with non-callable results, use bounded cached specialization; template metadata remains inspectable. Diagnostics point to source locations before invoking Rust.

Integer arithmetic uses explicit overflow checks in every build profile. Number, integer, and string comparisons, strict equality, boolean negation, and short-circuit logic support compound conditions. String operators use standard Rust content equality and UTF-8 lexical ordering. Operands evaluate once in source order; possible conditional transfers still affect ownership contracts. See [string comparison acceptance](./docs/milestones/34-string-comparisons.md).

Arrays support owning `slice`, in-place `reverse`, and typed `map`, `filter`, `some`, `every`, `findIndex`, and `findLastIndex`. Reusable callbacks accept zero through three arguments for value, int index, and readonly source; predicate methods short-circuit in the specified order. Index searches return int -1 for a miss. `reduce` and `reduceRight` accept reusable callbacks across current data types and ranks, with an independently typed initial accumulator or an owned/copy seed when omitted. Callbacks receive accumulator, value, original int index and optional readonly source. Empty arrays without an initial value fail after receiver/callback evaluation. [Reduction acceptance](./docs/milestones/51-array-reduction.md) records 28 comparisons and complete cleanup checks. `forEach` visits every element in ascending original-index order, accepts value/index/readonly-source callbacks and immediately disposes ignored owned results. [Traversal acceptance](./docs/milestones/52-array-for-each.md) records 16 comparisons and 48 complete disposal profiles. Proven stable scalar arrays use ordinary Rust arrays; resizing and escaping values keep existing growable storage. See the [current feature inventory](./docs/feature-coverage.md) for supported domains and comparison evidence.

Homogeneous `concat` accepts same-rank arrays and non-array scalar elements, evaluates operands once in order, and returns independent owned storage. Zero arguments still copy; typed result context supports empty literals. [Concatenation acceptance](./docs/milestones/53-array-concat.md) records 25 matched comparisons and 75 complete disposal profiles. JavaScript shares owned element references where TSRust copies their contents.

Strings support `repeat`, string-pattern `replace`/`replaceAll`, and zero through two-argument `split`. Results own independent UTF-8 storage; empty patterns and separators use Unicode scalar boundaries. Replacement templates preserve original-source substitutions and split limits wrap modulo 2^32. See [string construction acceptance](./docs/milestones/47-string-construction.md).

`number` uses standard Rust `f64` arithmetic, comparisons, and IEEE behavior, including NaN, infinities, and signed zero. Decimal/scientific literals infer `number`; bare integer literals retain the checked `int` extension and accept a `number` context. `Number()` returns zero; explicit conversion accepts integer, number, boolean, or string values. String conversion uses Rust trim and `f64` parsing: trimmed empty strings return zero and failed parses return NaN. Nonliteral integers require explicit conversion. Static `Number.isFinite`, `isNaN`, `isInteger`, and `isSafeInteger`, and `Math.abs`, `floor`, `ceil`, `trunc`, `sqrt`, `round`, `sign`, `fround`, `min`, `max`, `clz32`, and `imul` are implemented, together with the eight Math and eight Number constants. [Exact Math acceptance](./docs/milestones/48-math-rounding-and-integer-functions.md) includes independent comparisons and zero-allocation profiles; [Approximate Math acceptance](./docs/milestones/49-native-math-functions.md) adds nineteen inverse/trigonometric, hyperbolic, exponential/logarithmic and cube-root functions plus atan2, pow and hypot through standard Rust methods. All 22 matched comparisons pass predeclared bounded numerical criteria; native precision remains platform dependent. Number `&`, `|`, `^`, `~`, `<<`, `>>`, `>>>`, and `**` are also implemented with once-only operand evaluation, standard precedence, and right-associative power. Bitwise conversion uses truncated modulo-2^32 values and wrapped shift counts; power reuses the standard Rust Math implementation. [Number operator acceptance](./docs/milestones/50-number-operators.md) includes eight matched comparisons and zero-allocation profiles. Remaining methods stay in the feature inventory. See [number compiler acceptance](./benchmarks/runtime/diagnostics/2026-10-02T01-29-22-000Z-native-number-compiler-acceptance.json).

Braced `while`, `do...while`, classic `for`, and borrowed `for...of` loops support nearest-loop `break` and `continue`. Classic `for` runs its update after normal completion and `continue`; `do...while` runs its condition after those paths and executes the body at least once. `break` and `return` skip these tails. Values available before a loop must be restored on every repeating path. Loop-local values and capture loans end before the tail. Classic updates use existing assignments or expressions; `++` remains unsupported. See [control-flow acceptance](./docs/milestones/38-control-flow-loops.md).

It does not support update operators such as `++`, source-language labels, mixed-type callable replacement, borrowed or mixed-origin closure results, shared callback aliases, callable fields, async, generics, imports, shared ownership, or JavaScript compatibility generally. Native debugging still refers to generated Rust. The corpus is evidence for this bounded subset, not a general soundness or performance claim.

- [Compiler internals and CLI](./crates/tsrust-compiler/README.md)
- [Analysis performance baseline](./docs/analysis-performance.md)
- [Completed recursive callable result gate](./docs/milestones/29-recursive-callable-results.md)
- [Completed recursive owned callback gate](./docs/milestones/28-recursive-owned-callbacks.md)
- [Completed recursive callback gate](./docs/milestones/27-recursive-callbacks.md)
- [Completed recursive function gate](./docs/milestones/26-recursive-functions.md)
- [Completed nested record field gate](./docs/milestones/25-nested-record-fields.md)
- [Completed array record field gate](./docs/milestones/24-array-record-fields.md)
- [Completed nested array gate](./docs/milestones/23-nested-arrays.md)
- [Completed owned array element gate](./docs/milestones/22-owned-array-elements.md)
- [Completed borrowed for-of gate](./docs/milestones/21-for-of.md)
- [Completed owned scalar array gate](./docs/milestones/20-scalar-arrays.md)
- [Completed direct owned-callback return gate](./docs/milestones/19-callback-returns.md)
- [Completed callback-parameter move gate](./docs/milestones/18-callback-moves.md)
- [Completed callable declaration move gate](./docs/milestones/17-callable-moves.md)
- [Completed mutable callable gate](./docs/milestones/16-mutable-callables.md)
- [Completed capture replacement gate](./docs/milestones/15-capture-replacement.md)
- [Completed mutable binding gate](./docs/milestones/14-mutable-bindings.md)
- [Completed while-loop gate](./docs/milestones/13-while-loops.md)
- [Completed scalar operator gate](./docs/milestones/12-scalar-operators.md)
- [Completed retained callback gate](./docs/milestones/11-retained-callbacks.md)
- [Completed callable result gate](./docs/milestones/10-callable-results.md)
- [Completed callable capture gate](./docs/milestones/09-callable-captures.md)
- [Completed nested local closure gate](./docs/milestones/08-nested-local-closures.md)
- [Completed owned closure argument gate](./docs/milestones/07-owned-closure-arguments.md)
- [Completed scalar closure argument gate](./docs/milestones/06-closure-arguments.md)
- [Completed single-use callback gate](./docs/milestones/05-single-use-callbacks.md)
- [Completed synchronous callback gate](./docs/milestones/04-synchronous-callbacks.md)
- [Closure scope and implementation packages](./docs/milestones/03-closures.md)
- [Closure lowering evidence](./docs/experiments/closures-01.md)
- [Completed branch milestone](./docs/milestones/02-control-flow.md)
- [Original ownership milestone](./docs/milestones/01-ownership-proof.md)
- [Integer rules and original User example](./docs/milestones/01-integer-subset.md)
- [Vision and workstream index](./docs/README.md)
- [Proposed roadmap](./docs/roadmap.md)

The original numeric `User`/`normalize` example now compiles and prints `pat`. Run it with `cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/integers/original_user.tsr`.

Run the branch example with `cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/control-flow/branch_reuse.tsr`. It prints `Pat`, `Pat`, `Sam`, and `Sam`, demonstrating reuse after both read and mutation paths.

Closure gates 03a–03c are complete for the documented subset, including all 15 source cases and 16 independent handwritten Rust witnesses. Run `cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/closures/returned_clone_independent.tsr` to see an escaped closure retain `Pat` while the original changes to `Sam`.

The [synchronous callback gate](./docs/milestones/04-synchronous-callbacks.md) is complete. Run `cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/callbacks/inline_mutation.tsr` to see repeated callback mutation and owner reuse after inline capture loans end. Named closure loans still last until block exit.

The [single-use callback gate](./docs/milestones/05-single-use-callbacks.md) is complete. Run `cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/callbacks/single_use.tsr` to transfer a captured record through a callback and print `Pat`.

The [scalar closure argument gate](./docs/milestones/06-closure-arguments.md) is complete. Run `cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/callbacks/scalar_arguments.tsr` to exercise a two-argument callback and print `7` and `0`.

The [owned closure argument gate](./docs/milestones/07-owned-closure-arguments.md) is complete. Run `cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/callbacks/owned_arguments.tsr` to mutate a borrowed record, then transfer fresh records through a reusable closure. It prints `Sam`, `Sam`, and `Jo`.

The [nested local closure gate](./docs/milestones/08-nested-local-closures.md) is complete. Run `cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/callbacks/nested_local.tsr` to mutate an enclosing closure parameter through an inner closure and print `Sam` three times.

The remaining closure gates are complete: concrete callable captures, callable-valued arrow results, and owned retained callbacks. Run `cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/callbacks/retained.tsr` to print `true` then `false` from state retained after its factory returns.

Borrowed closure returns, mixed closure origins, shared callback aliases and callable fields, mutable/consuming iteration and broader collections, async, modules, and the Rust language server remain separate work. No future feature blocks the completed closure gates.

The [scalar operator gate](./docs/milestones/12-scalar-operators.md) is complete. Run `cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/operators/counter.tsr` to print `6`, `5`, and `ready`. Arithmetic overflow and division/remainder errors fail at runtime with the operator's source byte range. [Milestone 13](./docs/milestones/13-while-loops.md) subsequently adds while loops and repeated-iteration ownership.

The [while-loop gate](./docs/milestones/13-while-loops.md) is complete. Run `cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/loops/count.tsr` to print `5` and `9`, using both `continue` and `break`.

The [mutable binding gate](./docs/milestones/14-mutable-bindings.md) is complete. Initialized `let` bindings support scalar updates and exact-type string/record replacement, including reinitialization after transfer. Capture loans still block outer replacement; read-only scalar captures remain copied snapshots. Run `cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/mutable-bindings/reinitialize.tsr` to print `Pat` followed by `Sam` three times.

The [capture replacement gate](./docs/milestones/15-capture-replacement.md) is complete. Closures can update captured `let` bindings: local writers borrow storage exclusively, while escaping environments own state and copy scalars. Run `cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/capture-replacement/counter.tsr` to print `2`, `5`, and `1` from independent counters. A replacement that consumes the old capture keeps a single-use invocation contract.

The [mutable callable gate](./docs/milestones/16-mutable-callables.md) is complete. Callable `let` bindings accept replacement values from the same concrete environment origin, including after a single-use invocation. Named replacement moves the source value; matching signatures from different origins remain insufficient. Run `cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/mutable-callables/reinitialize.tsr` to print `Pat` followed by `Sam` three times.

The [callable declaration move gate](./docs/milestones/17-callable-moves.md) is complete. `const next = action` and `let next = action` transfer a concrete environment, preserving invocation and capture contracts. Return chains retain the original escape origin, and moved borrowed environments keep lexical loans. Run `cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/callable-moves/returned-chain.tsr` to print `Pat` twice.

The [callback-parameter move gate](./docs/milestones/18-callback-moves.md) is complete. All four callback contracts support local declaration moves. Reusable moves transfer the local borrow while preserving caller ownership; `once` restrictions survive moves and specialization. Owned locals can feed returned wrappers, and mutable locals can restore values derived from the same parameter. Run `cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/callback-moves/retained.tsr` to print `1`, `2`, and `1`.

[Annotated local callables](./docs/callable-locals.md) now support reusable, `once`, `owned` and `owned once` signatures, including grouped annotations and generic nominal results. Signatures validate argument access and return types while retaining concrete environment origins. Callable record fields and arrays remain pending.

The [direct owned-callback return gate](./docs/milestones/19-callback-returns.md) is complete. Functions and arrows can return owned parameters or moved locals through matching callable result annotations. Reusable returns retain their concrete environments; `owned once` returns preserve single-use invocation through cached allocation-free wrappers when needed. Non-owned callback escapes remain rejected. Run `cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/callback-returns/identity.tsr` to print `1`, `2`, and `1`.

The [owned scalar array gate](./docs/milestones/20-scalar-arrays.md) is complete. `int[]` and `boolean[]` support literals, checked indexing and updates, read-only length, push, ownership transfers, explicit cloning, captures, and callback contracts. Arrays use native Rust vectors; elements copy, arrays move. Run `cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/arrays/sum.tsr` to print `10`, `4`, `true`, and `false`.

The [borrowed for-of gate](./docs/milestones/21-for-of.md) is complete. `for (const value of values)` and `for (let value of values)` copy scalar elements while borrowing the source array for the loop. Iterable expressions evaluate once, temporary arrays remain alive, and active iteration prevents source mutation or transfer. Run `cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/for-of/sum.tsr` to print `8`, `5`, `true`, and `false`.

The [owned array element gate](./docs/milestones/22-owned-array-elements.md) is complete. `string[]` and named-record arrays move elements into native vectors. Indexing borrows, indexed record fields support replacement, and explicit cloning creates independent ownership. For-of borrows owned elements with enforced lexical access. Run `cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/owned-arrays/users.tsr` to print `Pat`, `true`, `Sam`, `false`, `Pat`, and `Alex`.

The [nested array gate](./docs/milestones/23-nested-arrays.md) is complete. Exact nested types support row moves, borrowed indexing, indexed push and writes, nested iteration, and explicit independent cloning. Parent loans prevent operand evaluation from invalidating a selected row. Run `cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/nested-arrays/matrix.tsr` to print `16`, `7`, and `9`.

The [array record field gate](./docs/milestones/24-array-record-fields.md) is complete. Records own exact scalar, string, record, and nested arrays, including recursive record arrays. Mixed field/index paths support mutation and iteration while loans protect the root owner. Run `cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/array-fields/catalog.tsr` to print `Pat`, `admin`, `Sam`, `new`, `7`, and `admin`.

The [nested record field gate](./docs/milestones/25-nested-record-fields.md) is complete. Named records own other named records inline, including forward field types. Source analysis rejects direct layout cycles and excessive layouts before Rust emission. Existing ownership, loans, captures, callbacks, and mixed storage paths apply to nested fields. Run `cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/nested-records/profile.tsr` to print `Pat`, `7`, `Sam`, and `9`.

The [recursive function gate](./docs/milestones/26-recursive-functions.md) is complete. Self and mutual data recursion infer stable read/mutate/consume access before definitive ownership checking. Closures and nonrecursive callback APIs preserve those effects. Milestone 27 subsequently extends recursion to ordinary reusable and single-use callback parameters; milestone 28 extends owned callback parameters too, while milestone 29 resolves callable results with one concrete environment origin. Run `cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/recursion/tree.tsr` to print `6` and `9`.

The [recursive callback gate](./docs/milestones/27-recursive-callbacks.md) is complete. Reusable and single-use parameters preserve exact contracts, aliases, transfers, and loans through self and mutual calls. Concrete type-vector validation bounds each recursive callback function before Rust emission. Run `cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/recursive-callbacks/visit.tsr` to print `6` from a borrowed callback traversing an owned tree.

The [recursive owned callback gate](./docs/milestones/28-recursive-owned-callbacks.md) is complete. Owned reusable/single-use environments move through self and mutual calls with preserved retention, access, and invocation contracts. Concrete data-result bodies are checked iteratively, and matching source arrow layouts reuse native types. Growing environments fail before the backend. Run `cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/recursive-owned-callbacks/countdown.tsr` to print `10` and `23` from independent retained counters.

The [recursive callable result gate](./docs/milestones/29-recursive-callable-results.md) is complete. Self/mutual factories and returned closures can recurse while preserving exact contracts and one concrete result origin. Origin discovery precedes final ownership checking; provisional shapes never reach public metadata or Rust output. Run `cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/recursive-callable-results/counter.tsr` to print `2` and `3` from a recursively constructed counter.

The [array removal gate](./docs/milestones/30-array-removal.md) is complete. It adds `array.remove(index)`, returning an owned element while preserving the order and ownership of the remaining array. Scalar, string, record, and nested elements support the existing storage paths, contracts, and source loans. Bounds failures retain source ranges under every optimization profile. Run `cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/array-removal/queue.tsr` to print `Sam`, `saved`, `2`, `Pat`, `Alex`, and `Sam`.

The [record field move gate](./docs/milestones/31-record-field-moves.md) is complete. Direct named string/record/array fields move without cloning while siblings remain usable. Exact field or parent replacement restores availability before whole-record use; joins and loop edges track missing fields. Parameter/callback/capture contracts and conservative root loans remain enforced. Run `cargo run --offline --bin tsrust -- run crates/tsrust-compiler/tests/fixtures/field-moves/profile.tsr` to print `admin`, `7`, `New`, and `Pat`.

## Runtime comparison ledger

SQLite is the canonical analytics store for exact benchmark records and derived measurements. [BENCHMARKS.md](./BENCHMARKS.md) remains a historical snapshot and Markdown exports are optional. The [benchmark harness](./benchmarks/runtime/README.md) defines independent source pairs, pinned tools, checked results, raw samples and normalized allocation profiles. [Checkpoint 91](./benchmarks/runtime/diagnostics/2026-10-03T18-25-41-000Z-checkpoint91-completed-proof.json) accepts 890 tested-revision checks, 68 matched native/compiled-Node comparisons and 204 fully disposed profiles; native is faster on 34/68. Rates include complete workload setup and retained outputs. There are 622 historically measured workloads among 623 registered sources; the separate peer path row remains unmeasured. Accumulated full regressions and measurements run every five gates, with checkpoint 96 accepted and checkpoint 101 next. Compiler-only guarantees and unsupported APIs have explicit [comparison limits](./docs/runtime-comparisons.md).

Strings use the [native Rust UTF-8 contract](./docs/string-semantics.md): byte length, checked byte slices, scalar character access, standard Rust trim/case conversion, and borrowed substring search with byte positions. String-search compiler, runtime comparison, and allocation acceptance pass. Earlier UTF-16 compatibility code is archived outside the crates.

Primitive array search supports `values.includes(value, fromIndex?)` and `values.indexOf(value, fromIndex?)` for integer, boolean, and string elements. Both read live elements and borrow string needles; source-root loans protect receiver and needle during argument evaluation. Literal/cloned/returned arrays own temporary storage. See [milestone 32](./docs/milestones/32-array-search.md) for bounds, equality, evaluation, identity restrictions, and comparison evidence.

Local integer and boolean array literals use ordinary Rust `[i64; N]` and `[bool; N]` storage when every use preserves their length. Indexed writes, independent fixed clones, and proved named calls can retain this storage. Resize, rebind, transfer, return, capture, or an exhausted proof budget retains ordinary growable storage. This private optimization does not change source array types or ownership rules.

### Testing and CLI help

`tsrust test` discovers `.test.tsr` and `.spec.tsr` files in the current directory.
Pass files or directories to select tests. Test files register tests at the top level
and do not declare `main`. Import assertions, matchers, suites and hooks from
`std:testing`; see `examples/testing/basic.test.tsr` and
`docs/plans/testing-library-plan.md` for the current parity and prerequisite ledger.

Use `tsrust --help` or `tsrust test --help` for command and option help.
Color defaults to terminal detection and respects `NO_COLOR` and `TERM=dumb`.
Use `--color always` or `--color never` to override it. JSON results, emitted Rust
and formatted source remain machine-readable.

Building this workspace does not update an executable previously installed on PATH.
To install the current CLI, run `cargo install --path crates/tsrust-compiler --force`.
