# Gate 132: declaration lists and loop update lists

Status: focused compiler/native, editor, guide, current consumer and matched benchmark acceptance complete. Full compiler/LSP regressions and measurements remain deferred.

## Behavior

A `const` or `let` declaration accepts several declarators separated by commas: `let a = 1, b = a + 1, text: string = "t";`. Initializers run in source order and a later one can read an earlier name. A declarator can be a destructuring pattern. Every declarator needs an initializer. The keyword applies to every name.

The initializer of a classic `for` loop accepts a declaration list, and its update clause accepts several assignments or expressions separated by commas: `for (let i = 0, j = 10; i < j; i++, j--)`. Updates run in source order on every continuing edge, including after `continue`.

Each declarator keeps the ownership rules it has as its own declaration. The comma is a separator in these two positions only; a general comma expression is still unavailable.

See [the syntax guide](../guides/language/declaration-lists.md).

## Implementation

`Parser::declaration_list` in `syntax.rs` lowers each later declarator to an ordinary declaration placed after the first through the existing pending-statement queue, so blocks and classic-for initializers receive them in order. `pattern_declarator` is the pattern path without its keyword. `for_update` collects the update statements. All three are non-inlined helpers outside the recursive statement frame. No analysis or emission pass changed.

The editor records the commas that separate declarators and declares every later name with the declaration's keyword. A comma inside a generic type argument list is not a separator.

## Repairs

- Formatter: a record pattern in a parameter list was split across lines (`function f({` / `x, y` / `}`); it now stays on one line, for functions and arrows. A pattern in a later declarator stays on one line.
- Formatter: a parenthesis after an operator lost its space (`a -(b)`, `const g =(z: int)`, `(1 + 2) *(3 - 4)`, `return(a)`). Operators and keywords now keep the space; a sign keeps `-(a)`.
- Two editor catalog tests had failed since matcher and complete-program examples entered the catalog: `documented_builtin_examples_match_native_output` and `every_catalog_entry_has_a_current_compiler_witness`. Baseline sources from gate 131 show the same two failures. The tests now run any example with its own `main` as a complete program, treat the `Expectation` and `NegatedExpectation` families as witnessed by their examples, and accept standard-library source modules that have no native binding. Catalog entries were not changed.
- `archive/declaration-lists-start/lsp` first lost the editor source snapshot because two files shared one name. `src-intelligence.reconstructed.rs` is the current source with this gate's editor edits reversed; it is labeled as a reconstruction.

## Evidence

- Snapshots: `archive/declaration-lists-start` (sources), `archive/declaration-lists-benchmark-start` (harness and tracking files), `archive/declaration-lists-consumers-start` (gate 131 consumers).
- `target/editor/declaration-lists-red.log` keeps the initial failing run. `declaration-lists-final-compiler.log`: eleven suites pass on the final sources: `declaration_lists`, `for_pattern_initializers`, `formatting`, `destructuring`, `destructuring_defaults`, `destructuring_rest`, `loops`, `control_flow`, `control_flow_native`, `higher_order_function_values` and `type_syntax`. The log lists each count.
- The executable fixture uses `std:testing` assertions and passes in Node through `tools/node-reference/run.mjs`.
- `declaration-lists-guide-verification.log`: all three guide examples pass natively and in Node.
- `declaration-lists-editor-baseline.log`: gate 131 sources, 125 passed and the same 2 catalog tests failed. `declaration-lists-final-editor.log`: all 211 editor tests pass, including `declaration_lists_declare_every_name_with_its_type`.
- Consumers were rebuilt and replaced by atomic rename before execution checks; hashes are in `declaration-lists-final-debug.log`.
- `declaration-lists-harness.log`: all 347 runtime harness checks pass in one run.

## Matched support

`declaration_lists` retains five cells per iteration: a six-name `const` list with an array pattern, a converging two-variable loop with two updates, and a loop whose update list appends text after `continue`. The independent oracle is closed-form; the registration test also compares it with the platform on 2048 cases. Hashes, guards, the whole-scenario denominator, registration, runtime and allocation-profile hooks, oracle fingerprints and the report category are in place.

Both registration checks pass in `declaration-lists-registration.log`. `declaration_lists`, `for_pattern_initializers` and `classic_for_control` pass untimed on native and Node-after-tsc in `benchmarks/runtime/results/2026-10-05T11-40-59-998Z-verify.json`.

No timing, allocation profiling or complete compiler/LSP regression was run. SQLite measurements are unchanged. Next gate is 133.
