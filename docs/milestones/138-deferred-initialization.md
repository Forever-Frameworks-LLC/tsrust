# Gate 138: `let` without an initializer

Status: focused compiler/native, editor, formatter, guide, current consumer and matched benchmark acceptance complete. Full compiler/LSP regressions and measurements remain deferred.

## Behavior

`let name: Type;` declares a variable that is assigned later. It can stand alone, be one declarator of a declaration list, or be the initializer clause of a `for` statement.

- The annotation is required (`annotation-required`). `const` still needs an initializer (`syntax`), and so does a destructuring pattern.
- Every use must follow an assignment on every path; otherwise the use is an `unassigned` error.
- A type that includes `undefined` starts as `undefined` and can be read at once.
- A function-typed variable needs an initializer (`unsupported`).

See [the guide](../guides/language/deferred-initialization.md) for the accepted assignment forms.

## Implementation

- Parser: `Parser::declarator_value` returns a placeholder initializer (`Expr::deferred`, an `undefined` node with an empty span) when a `let` declarator is followed by `;` or `,`. `StmtKind::Let` is unchanged.
- Analysis: `Analyzer::deferred_let` binds the name with its annotated type and no value. For a type that admits `undefined`, it turns the placeholder into an ordinary `undefined` initializer instead.
- `definite_assignment.rs` runs after analysis on every checked function and function-value body. It walks the statements after each deferred declaration. A plain assignment assigns. An `if` assigns when each branch assigns or leaves. Loop bodies are checked for early uses but never assign for the code after the loop. A function value that captures the variable counts as a use; captures are found through the checked closure table, because checked function values keep their bodies outside the expression. The accepted programs are a subset of what the generated Rust accepts.
- Emission: a deferred declaration is `let mut valueN: Type;`. The first assignment initializes it.

No editor source change was needed: the editor reads the annotation. The new editor test passed on its first run and guards hover, definition and member completion. The formatter also needed no change; a formatting test fixes the layout.

## Updated expectations

`declaration_lists` expected `let a = 1, b;` to be a `syntax` error; it is now `annotation-required`, and a `const` declarator without an initializer keeps the `syntax` row. `mutable_bindings` expected `let n: int;` to be a `syntax` error; the rows now check that reading it first is `unassigned` and that `const n: int;` is still `syntax`. The previous files are in the gate snapshot.

## Evidence

- Snapshots: `archive/uninitialized-let-start`, `archive/uninitialized-let-benchmark-start`, `archive/uninitialized-let-consumers-start` (gate 137 editor and release consumers).
- `uninitialized-let-red.log`: both compiler tests fail before the change.
- `uninitialized-let-final-compiler.log`: eighteen suites on the final sources: `uninitialized_let` (an executable `std:testing` fixture and sixteen static rejections), `declaration_lists`, `destructuring`, `destructuring_assignment`, `for_pattern_initializers`, `braceless_bodies`, `switch_statements`, `loops`, `control_flow`, `closures`, `higher_order_function_values`, `type_syntax`, `formatting`, `acceptance`, `global_number_functions`, `function_expressions`, `project` and `mutable_bindings`.
- The executable fixture passes in Node through `tools/node-reference/run.mjs`. A second probe with loops, a `do...while`, a function value, unions, arrays, records and a moved string printed the same thirteen lines natively and in Node.
- `uninitialized-let-guide-verification.log`: the guide example passes natively and in Node, and each rule stated in the guide gives the stated result.
- `uninitialized-let-final-editor.log`: all editor tests, including `deferred_let_declarations_keep_types_definitions_and_members`.
- Consumers were replaced by atomic rename; hashes are in `uninitialized-let-final-debug.log`. The workload source was checked with the refreshed release compiler before execution checks.
- `uninitialized-let-harness.log`: the runtime harness checks in one run.

## Matched support

`deferred_initialization` retains five cells per iteration from five declarations without initializers: one assigned through an `if` chain, one in a declaration list, one through a `switch` with a `default` clause, one in a nested block, and one whose type includes `undefined`. The independent oracle writes each cell as a direct expression; the registration test compares it with the platform on 2048 cases for three inputs. The source contract finds exactly those five declarations without initializers in the TypeScript syntax tree. Hashes, guards, registration, runtime and allocation-profile hooks, oracle fingerprints and the report category are in place.

`deferred_initialization` and `global_number_names` pass untimed on native and Node-after-tsc; the result file is named in `uninitialized-let-verify.log`.

No timing, allocation profiling or complete compiler/LSP regression was run. SQLite measurements are unchanged. Next gate is 139.
