# Gate 116: String.normalize

Status: focused implementation, editor support, documentation and benchmark support accepted. Full regressions and timing remain deferred.

## Behavior

`text.normalize(form?)` returns independent Unicode-normalized text, without changing its receiver or form argument. Both expressions evaluate once, receiver first.

- Omitted form or undefined selects NFC. The form can also be a string-or-undefined variable or optional parameter.
- NFC performs canonical decomposition, canonical combining-class ordering and composition. NFD retains the canonical decomposed form.
- NFKC and NFKD additionally apply compatibility decomposition; for example ligatures and circled digits become ordinary letters and digits.
- Combining marks with equal classes retain their order. Composition observes blocking and full composition exclusions. Hangul syllables compose/decompose algorithmically.
- Empty text remains empty. Normalization can combine, split or reorder characters and change byte length. It does not fold case or remove accents.
- Unknown form names stop native execution with `normalize form must be NFC, NFD, NFKC or NFKD` and the call's source bytes. This happens even for empty or ASCII text. Nonstrings other than undefined produce a type diagnostic; more than one argument produces an arity diagnostic.
- Input strings contain valid Unicode scalars; lone UTF-16 surrogate halves cannot be represented.

## Data and implementation

The complete Unicode 16.0.0 source tables and license are pinned in `tools/unicode-normalization/data`, with exact hashes in `sources.json`. This matches Node v24.11.1 / ICU 77.1 / Unicode 16.0. The [offline generator](../../tools/unicode-normalization/README.md) produces combining-class, decomposition and composition tables, including full composition exclusions; regeneration is byte-identical.

The algorithm follows [Unicode normalization](https://www.unicode.org/reports/tr15/), with decomposition, stable mark ordering and blocked/excluded composition. Generated Rust includes the tables and complete Unicode license notice only when normalization is used. No Cargo dependency or build-time network access is added.

The checker handles arity, string/undefined form types and receiver loans in a separate helper. Native lowering stages temporary optional-form owners before borrowing their union views; optional ReadOnly forms, owned unions and temporary unions are covered. The emitter evaluates both operands once and retains their borrows until normalization finishes.

The `string.normalize` catalog entry supplies completion, hover, optional-form signature help and two executable examples. Editor inference shows a string result.

## Evidence

- Initial three compiler tests fail before implementation (`target/editor/string-normalize-red.log`). Focused checks cover all forms, defaults, character ordering, Hangul, receiver/form evaluation, source reuse, diagnostics and invalid runtime names.
- The optional-form fixture initially exposed a native union-view lowering error, repaired with union tag selection. Temporary optional forms then exposed an owner lifetime error; the regression and repair are retained in `string-normalize-temporary-red.log` and `string-normalize-temporary-green.log`.
- Official Unicode conformance: 19,965 rows, 399,300 relationships and 1,101,784 assigned-character identity checks pass natively. The same 399,300 row relationships pass independently on Node; see `string-normalize-conformance-supplement.log`, `string-normalize-compiler.log` and `string-normalize-node-conformance.log`.
- `string-normalize-compiler.log`: six normalization tests, 21 optional-parameter tests and nine string tests pass. The seventh normalization test in `string-normalize-emission-supplement.log` proves table/license inclusion when used and omission in plain string programs. All earlier passing evidence is retained.
- `string-normalize-editor.log`: focused protocol checks and both catalog examples execute successfully. `string-normalize-generation.log` proves pinned source integrity and byte-identical table regeneration. `string-normalize-node-fixtures.log` proves the basic compiler fixture agrees with Node.
- Debug compiler/LSP and release compiler rebuilt; workspace/editor consumers refreshed. The matched normalization scenario retains seven results covering all forms, both default paths and a dynamically selected form over canonical, compatibility, Hangul and combining-order inputs. Its written-form oracle does not call normalize. Fingerprint/AST guards, ABI/driver registration and profile hooks are ready.

- `string-normalize-workload-verify.log` and `benchmarks/runtime/results/2026-10-04T19-46-24-799Z-verify.json`: five untimed cases pass on native and Node after tsc. A first preparation rejected an owned binding for an indexed form string; the matched sources now borrow the selected form directly in the call. The original failed preparation log remains retained.
- `string-normalize-harness.log`: all 314 benchmark harness tests pass on current consumer binaries. Seven normalization tests have passing evidence, including the later emission/license supplement; adjacent optional-parameter and string suites retain their completed passes.

## Pending measurements

No full compiler/LSP regression, timing or allocation profile is run. Decomposition uses temporary scalar buffers; composition and output construction allocate further storage, and table lookups are binary searches. These costs are unmeasured. SQLite measurement records remain unchanged. Exact prior source and binary snapshots remain in `archive/string-normalize-start`.
