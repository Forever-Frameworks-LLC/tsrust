# Gate 137: global number names

Status: focused compiler/native, editor, grammar, guide, catalog documentation, current consumer and matched benchmark acceptance complete. Full compiler/LSP regressions and measurements remain deferred.

## Behavior

`parseInt`, `parseFloat`, `isNaN`, `isFinite`, `NaN` and `Infinity` are accepted without the `Number.` prefix. Each is the same function or value as `Number.parseInt`, `Number.parseFloat`, `Number.isNaN`, `Number.isFinite`, `Number.NaN` and `Number.POSITIVE_INFINITY`, with the same argument, arity and type rules.

- A declaration with the same name takes precedence: a top-level function, a variable, a parameter or an imported name.
- `isNaN` and `isFinite` accept a number only. No argument is converted.
- `typeof NaN` and `typeof Infinity` are `"number"`.
- `NaN` and `Infinity` cannot be assigned to (`name`).
- A global function used as a value, for example `texts.map(parseInt)` or `typeof parseInt`, is `unsupported`, with advice to wrap the call in an arrow function.

See [the guide](../global-number-names.md).

## Implementation

`syntax::global_number_member` maps a global name to its `Number` member. During analysis, `Analyzer::global_number_name` rewrites an undeclared global name to the `Number` member expression, at the callee of a call and at a name expression, so the existing `Number` implementation, emission and diagnostics serve both spellings.

Three places that look at names before that rewrite needed the same knowledge:

- The project linker (`project_link.rs`) gave every undeclared name a module symbol. It now leaves the six names alone, so programs that import a module, including `std:testing`, resolve them.
- The generic-default scope check (`generic_default_scope.rs`) accepts the names in parameter defaults.
- Type hints: an undeclared `NaN` or `Infinity` hints `number`, and a call of an undeclared global function hints `number` or `boolean`. Without this, `-Infinity` and `-parseInt(text)` with no expected type were checked as `int`.

`typeof` no longer treats the six names as unresolved.

## Editor

- `builtins.json` documents all six names in the `globals` family: purpose, parameters, result, verified examples, errors, precedence of declarations and a short ownership note. The examples were run with the current compiler when the entries were written, and the catalog tests check them again.
- Hover, completion with documentation, and signature help work for the global names. Binding types are inferred (`number` or `boolean`). A declaration with the same name keeps its own hover.
- Found and fixed: a binding whose initializer starts with a prefix sign, such as `const low = -Infinity` or `const low = -1.5`, had no hover type. `simple_expression_type` now types a signed int or number operand.
- The TextMate grammar highlights the four functions before `(` and the two constants, but not after a member dot.

## Updated expectation

`number_prefix_parsing` expected a bare `parseFloat("1")` to be a `name` error. The row now uses an unknown global name. The previous file is in the gate snapshot.

## Evidence

- Snapshots: `archive/global-number-functions-start` (compiler sources, the changed compiler test, editor sources and catalog, grammar, records), `archive/global-number-functions-benchmark-start`, `archive/global-number-functions-consumers-start` (gate 136 editor and release consumers).
- `global-number-functions-red.log`: the executable fixture and the static contracts fail before the change. `global-number-functions-editor-red.log` and `global-number-functions-grammar-red.log`: the editor and grammar tests fail before their changes.
- `global-number-functions-final-compiler.log`: eighteen suites on the final sources: `global_number_functions` (an executable `std:testing` fixture, a shadowing fixture and twelve static rejections), `higher_order_function_values`, `type_syntax`, `acceptance`, `function_expressions`, `closures`, `optional_parameters`, `formatting`, `generic_default_scope`, `generic_default_builtins`, `number_parse_int`, `number_predicates`, `number_prefix_parsing`, `numbers`, `project`, `number_operators`, `nullable_narrowing` and `data_unions`.
- Both executable fixtures pass in Node through `tools/node-reference/run.mjs`. `global-number-functions-guide-verification.log`: the guide example passes natively and in Node.
- `global-number-functions-final-editor.log`: all editor tests, including `global_number_names_have_hover_help_completion_and_types` and the catalog example and witness tests.
- `global-number-functions-grammar.log`: the grammar test passes.
- Consumers were replaced by atomic rename; hashes are in `global-number-functions-final-debug.log`. The first refresh attempt failed before copying anything because of shell word splitting and was repeated. The workload source was checked with the refreshed release compiler before execution checks.
- `global-number-functions-harness.log`: all 357 runtime harness checks pass in one run.

## Matched support

`global_number_names` retains five cells per iteration: `parseInt` with trailing text plus `parseInt` with radix 16, `parseFloat` with trailing text, `isNaN` of a number read from the input text, an `isFinite` pair over a minimum that starts at `Infinity` and over `-Infinity`, and a value that is `NaN` when the input holds no number. The independent oracle reads digits by hand and detects not-a-number by self-inequality; the registration test compares it with the platform on 2048 cases for six inputs. The source contract counts each global name in the TypeScript syntax tree and rejects `Number.` members. Hashes, guards, registration, runtime and allocation-profile hooks, oracle fingerprints and the report category are in place.

`global_number_names` and `function_expressions` pass untimed on native and Node-after-tsc in `benchmarks/runtime/results/2026-10-05T12-39-44-714Z-verify.json`.

No timing, allocation profiling or complete compiler/LSP regression was run. SQLite measurements are unchanged. Next gate is 138.
