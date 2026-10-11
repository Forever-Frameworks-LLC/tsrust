# Gate 185: project editor (S04)

Date: 2026-10-10. Ledger row: B9 in `docs/plans/compiler-completion-plan.md`; gate S04 in
`docs/standard-library-roadmap.md`.
Acceptance map: `../b9-project-editor-acceptance-map.md`. User guide:
`../project-editor.md`. Earlier increments: `modules-03-workspace-snapshots.md`.

## Result

S04 is accepted. The earlier increments already supplied unsaved dependencies,
invalidation, cross-file diagnostics, assistance, references, versioned fixes and disk
lifecycle handling. This gate closes the items that the increment record still listed
as open and repairs two defects found on the way.

## Added in this gate

- **Quick fixes for closed files.** A fix may now edit a file that is not open. The
  whole project is validated with the edit, as before. Each request compares the file
  on disk with the text that was checked and withdraws the fix when they differ. The
  comparison uses the content, so a change that keeps length and modification time is
  detected. The edit carries no document version.
- **Help in incomplete sources.** `load_project_headers` now accepts an unfinished
  string, template or comment (the rest of that line is ignored), a body without its
  closing brace before a later `export` or `import`, an import statement that is not
  complete, empty import braces and an imported name that the module does not export.
  The other imports and names keep their help. Strict loading and checking are
  unchanged and still report every one of these.
- **Import name completion.** Between the braces of an import, completion lists the
  exports of the module, for local and native modules.

## Defects repaired

- The editor tokenizer let an unfinished quoted string run to the end of the file, so
  every later line lost hover, completion and navigation. A quoted string now ends at
  the end of its line. This also applies to single-file buffers.
- A module buffer with an unfinished string, template or comment was treated as a
  single file, because module detection used the strict lexer. Module detection now
  uses the tolerant lexer and recognizes an `export` or `import` after an unclosed
  body.

## Behavior change to note

Header loading used to fail with `module-private` when an import named a missing
export, which removed all project help for the file. It now drops that one name. Two
older tests asserted the previous behavior and were rewritten:
`current_headers_supply_imported_help_while_bodies_are_incomplete` (editor) and
`editor_header_loading_tolerates_bodies_but_preserves_module_boundaries` (compiler).

## Evidence

- Whole editor crate: 460 passed, 0 failed, 1 ignored (the opt-in cost observation).
  New suites: `project_incomplete.rs` (6 tests) and two closed-file tests in
  `project_actions.rs`. The in-memory protocol test also requests import completion
  with an unfinished string in the buffer.
- Whole compiler crate: 2,726 passed, 0 failed, 3 ignored, no aborted suite.
- Extension compile and TextMate grammar test pass (`npm test`).
- Cost observation at 10, 50 and 200 modules: see the acceptance map.
- Project format pass: no file changed.
- SQLite records: `tsrust-b9-acceptance-whole-compiler.log`,
  `tsrust-b9-acceptance-whole-editor.log`, `tsrust-b9-editor-scaling.log`.
- Clippy was not used as evidence: the compiler crate has older style findings that
  stop a warnings-denied run before it reaches the editor crate.

## Correction to gates 182 and 183: two suites aborted without being counted

The whole-compiler runs recorded for B5 (gate 182) and B7 (gate 183) reported no
failures. That was wrong. `rest_parameters` aborted with a stack overflow in every
whole-crate run since the B5 carrier work, and `math_exact` since the B7 work. An
aborted suite prints no result line, and the summaries only added up result lines. The
stored logs contain the abort messages.

Cause: the frames of the recursive expression checker grew until 100 nested calls or
100 nested unary operators no longer fitted the 2 MB stack of a test thread. The
language limit is 128 levels, and the editor checks on a thread of the same size.

Repair in this gate, in `analysis.rs`, without a behavior change:

- `declared_named_call` keeps only the argument recursion; the work before and after
  the arguments moved to `declared_call_contract` and `finish_declared_call`.
- Unary and binary checks moved out of `expression_inner` into `expression_unary`
  (with one function per kind of unary operator) and `expression_binary`.
- `array_literal` keeps only the element recursion.

Before the repair one nesting level used about 21 KB of stack in a debug build. The
largest frames on that path were `declared_named_call` (6.8 KB, now 1.8 KB),
`expression_inner` (12.6 KB, now 2.9 KB) and `array_literal` (4.4 KB). Both suites pass with a
2 MB stack; the unary case also passes with 1.5 MB. The 24 tests of the two suites are
counted again, which is the difference between 2,702 and 2,726.

Regression summaries now check for aborted targets as well as failed tests.

## Not done

- The packaged-extension (Extension Host) suite was not run for this gate. It starts a
  VS Code window. The additions are covered by library tests and the in-memory protocol
  test. Run `npm run test:integration` in `editors/vscode` after the consumers are
  refreshed to confirm them in the installed extension.
- No runtime workload: the gate changes editor behavior only.
- `textDocument/rename`, default exports, export lists and re-exports are outside S04.
