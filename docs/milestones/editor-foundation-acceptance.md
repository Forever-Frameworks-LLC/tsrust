# Editor foundation acceptance

October 2, 2026. [Implementation plan and compiler handoffs](../guides/tooling/editor-foundation.md) · [Local build/install instructions](../../editors/vscode/README.md).

## Delivered and locally installed

E1–E4 are implemented. `crates/tsrust-lsp` calls the unchanged compiler `check` API in process. `editors/vscode` provides the `.tsr` language contribution, TextMate grammar, comment/bracket configuration, language client, server path/debounce settings, restart command, output channel and status indicator. The locally built `tsrust-local.tsrust@0.2.0` VSIX is installed in the user's VS Code, and this checkout's ignored `.vscode/settings.json` points to `target/editor/debug/tsrust-lsp`.

Observed in the actual user's VS Code window: `use_after_call.tsr` has colored keywords, primitive types, strings, calls and brackets; its use-after-transfer has a red squiggle, a one-error count, and a running Tsrust status. The native window recognizes Tsrust rather than Plain Text. The existing fixture was opened for inspection without modifying its source.

The VSIX bundles its JavaScript runtime, third-party notices and the language client's process-termination helper. It contains no Rust server binary and was not published externally. Because this repository has no declared source license, the extension remains `UNLICENSED`; packaging emits a missing-project-license warning. Dependency licenses are retained in `THIRD_PARTY_NOTICES.txt`. This warning does not prevent local use.

## Verification

- Seven Rust tests pass: UTF-16 ranges/invalid spans; bounded, fair and versioned scheduling; direct compiler diagnostic equivalence/related locations; blocked-check edits/close/reopen; responsive shutdown during blocked analysis; actual framed stdio Unicode/repair/lifecycle; mandatory UTF-16 fallback and exit status.
- The extension passes TypeScript checking, bundling and real TextMate/Oniguruma tokenization tests, including multiline comments, escapes, numeric literals and ownership syntax.
- VS Code 1.139.1 on macOS arm64 passes the Extension Host suite against the development extension and the exact installed VSIX directory in temporary profiles. It verifies unsaved ownership errors and related transfer information, repair clearing, Unicode-correct type spans, incomplete syntax, independent documents, rapid edits, restart, close/reopen and missing-server behavior. The packaged test installs the VSIX via the official CLI, then directs the test host at those installed files because VS Code requires a development path to execute extension tests.
- Scoped server Clippy with all targets and warnings denied, and server formatting checks, pass. Compiler language implementation was not changed; runtime benchmark tables and the compiler's full feature regression suite were not rerun for editor plumbing.

Reproduction commands live in the extension README. Rust dependencies are locked in workspace `Cargo.lock` and fetched into the normal local Cargo cache so offline workspace commands can resolve the new server member. Node dependencies are locked in `editors/vscode/package-lock.json`. The compiler crate remains dependency-free.

## Measured latency

[Raw records](../measurements/editor-foundation-2026-10-02.json) retain all samples, the toolchain and scope. The local frontend baseline uses the release profile, five warmups and 31 checks. It times `check` only, excluding source construction, result destruction, Rust emission, native builds and editor transport.

The stdio baseline uses the debug server, five warmup edits and 21 measured edits per workload. Every edit changes unsaved text and waits for a diagnostic publication with the matching version. It includes the default 200 ms debounce, full-text JSON/stdio transport, worker scheduling, checking and diagnostic publication; it excludes the VS Code client and UI rendering. These are local observations, not production latency guarantees.

| Workload | Source bytes | Release analysis median | Debug stdio edit median including 200 ms debounce |
| --- | ---: | ---: | ---: |
| Existing read/reuse fixture | 175 | 0.009 ms | 205.4 ms |
| Existing use-after-call fixture | 228 | 0.010 ms | 204.4 ms |
| 1,000 supported independent functions | 55,909 | 1.898 ms | 237.8 ms |

Reproduce from the repository root:

```sh
CARGO_TARGET_DIR=target/editor cargo run --offline --locked --release -p tsrust-lsp --example editor_bench
python3 editors/vscode/scripts/measure-latency.py target/editor/debug/tsrust-lsp
```

## Remaining compiler work and boundaries

C0 is satisfied through the existing API and adapter contract tests. C1 incomplete-source recovery, C2 multiple diagnostics, C3 cooperative cancellation, C4 source-indexed semantic metadata and C5 module/project analysis remain explicitly deferred in the handoff matrix. Completing E1–E4 does not imply any of these capabilities exists.

The server checks whole single files and publishes the first compiler error. An incomplete construct can obscure later errors. It suppresses obsolete results and keeps at most one latest pending snapshot per open document; it cannot interrupt a running compiler check. The protocol loop remains responsive and shutdown/exit does not wait for the analysis worker. Worker failures report an internal failure instead of clearing errors as if validation succeeded.

Version 0.2 adds source-assisted hover, completion and navigation as described below. Compiler-proven semantic indexing, rename, semantic highlighting, formatting and debugging remain separate. Other operating systems, remote-host installations and representative multi-file project latency remain unverified. Compiler gates must continue recording grammar, diagnostic/span, analysis-cost and metadata impact as specified in the plan.

## Version 0.2 source assistance acceptance

The user's expanded request adds file icons, documentation/signature hover and autocomplete without waiting for complete compiler metadata. E5 now also delivers same-file definitions/references/highlights, outline, folding and snippets. Function hover includes declared args/result and adjacent `/** ... */` or `//` comments; signature help includes `@param` descriptions and correct active-argument counting through nested calls and string literals. Source-declared callback parameters also expose their argument signatures.

Eight source-intelligence tests add source doc/argument/navigation/outline assertions, lexical shadowing and comment isolation, incomplete record/array member suggestions, function completion with comments and forward declarations, Unicode/nested argument counting, callback signatures/folding, loop element types/scope isolation, and an accepted compiler witness for every built-in catalog entry. Unknown compound types do not produce guessed suggestions. Earlier diagnostic/scheduler/transport tests are retained.

The expanded installed-VSIX Extension Host suite passed on VS Code 1.139.1 macOS arm64: actual hover provider returned function docs; signature help returned two declared args and selected the second; completion returned functions and record fields on incomplete unsaved buffers; definition/reference and symbol providers resolved the declared function; the manifest packaged the file icon. The original diagnostics tests also passed. Rust Clippy with warnings denied and formatting passed.

The compatibility script verified current compiler sources while milestone 67 was in progress (gate 66 was the last accepted gate). It stores a compiler/editor fingerprint and successful contract results in `target/editor/compiler-compatibility.json`; it skips unchanged passing state, supports `--force`, and identifies new implemented built-in inventory entries missing from the catalog. It never runs the compiler's full feature regression or benchmark tables for editor work.

Hover/navigation/completion results are source assistance, not a claim that an incomplete file type-checks or that the index implements all compiler name/type/ownership semantics. Fully inferred types/contracts, complex callable origins, semantic rename and modules remain compiler handoffs. The user explicitly approved hourly read-only monitoring, and the automation `check-tsrust-compiler-and-editor-compatibility` is active. It reports compatibility failures or newly available editor capabilities and stays quiet on unchanged passing state. Native VS Code visual verification after reload confirmed the orange Tsrust icon in Explorer, tabs and breadcrumbs.

## Built-in hover documentation refresh — October 3, 2026

The 114-entry built-in catalog now explains observable behavior, argument meaning/defaults, return values, mutation, relevant native differences and ownership. Each entry includes an executable Tsrust example and expected stdout. Existing generic search/predicate descriptions were corrected; string search signatures expose the supported optional position. Optional/rest parameter labels now resolve their parameter descriptions in signature help.

`documented_builtin_examples_match_native_output` checks every example with the current compiler, combines the examples into a generated Rust program, compiles it with the pinned rustc and compares complete stdout to the expected values. Hover/signature tests verify that examples, rounding differences and optional argument documentation reach protocol responses. These examples validate the documented happy paths; edge-case descriptions also follow existing compiler milestone contracts and acceptance fixtures.

Future catalog entries must include meaningful behavior, argument/default/error information, at least one verified example and relevant native differences. Keep ownership notes after the user-facing behavior. The catalog supplies hover, completion documentation and signature help; no extension reinstall is needed after rebuilding the configured LSP binary, but a running server must restart to load it.

### Built-in roots and type hover

Hover now also covers `console`, `Math`, `Number`, `clone`, and primitive type names (`int`, `number`, `string`, `boolean`, `void`). Namespace documentation lists the current supported members directly from the catalog. Number conversion and clone include argument help, observable behavior, ownership and examples. Regression checks cover root hover, callable signature help, source declaration precedence, record fields and literal exclusion. Existing member hover and catalog examples remain verified by the focused suite.

### Console parity expansion — October 3, 2026

Compiler and editor now support variadic log/info/debug/warn/error, borrowed structured values and table with optional columns. Console catalog examples verify stdout and stderr separately; namespace docs, completion, hover and variadic argument help match compiler behavior. Focused native tests pass at O0/O3, including temporary lifetimes, loan rejection and disposal against the warmed stdio baseline. The 35 existing fixed-storage/nested-record groups and 30 current editor tests pass. Compiler/LSP debug and release builds pass. Changed compiler targets and all editor targets pass Clippy; a broader invocation encountered an unrelated quoted-string fixture warning during concurrent gate work.

The compatibility checker now distinguishes primitive Number/Boolean instance methods from static NumberConstructor members and fingerprints its own implementation. It supports milestone numbers beyond two digits. Current catalog-expansion candidates are empty. Detailed contract, remaining host gaps and five-gate checkpoint obligations are in [console parity](console-parity.md).

## Node console reference expansion

The console reference is now the pinned Node.js v24.11.1. The earlier TSV/literal-placeholder console contract is superseded by [console-parity.md](console-parity.md). Inline console object literals, boxed table output, placeholders, inspection, assertions, groups, counters, timers, dir/dirxml, clearing, native tracing and inspector no-ops are implemented and documented across all 22 global methods. Hover/signature help/completion use the same catalog. This is native compatibility within supported values, not a claim of complete Node host equivalence; the milestone explicitly tracks value, wrapping, width, advanced inspection, timer, warning and native-stack differences.

Focused acceptance includes 11 compiler console tests (common-value stdout/stderr compared against Node at O0/O3, plus native ownership/storage/host contracts), 21 fixed-storage tests, 14 nested-record tests and 31 LSP tests (24 intelligence, 3 library, 2 main, 2 stdio). Executable documentation examples include separate stderr outputs; nondeterministic timer/trace output is checked separately. Compiler changed-target and LSP all-target Clippy pass. Full regression/performance remain at the planned five-gate checkpoint. No new console throughput measurement is claimed.

## Singleton values and static namespaces — October 3, 2026

Null/undefined literals and type hover, typeof/void inference, String/Boolean conversion signatures and Array.isArray discovery are implemented. Root/member documentation includes observable behavior, defaults, examples and native limits. Array constructor and instance catalog families remain separate; local undefined declarations take precedence over global hover/inference. All 35 LSP tests pass, including compiler witnesses and native examples. TypeScript/real TextMate checks and installed-VSIX E2E provider/lifecycle tests pass using a temporary profile outside the sandbox. Debug/release builds and focused Clippy/format checks pass; the updated VSIX is installed locally.

Compiler proof, exact limits, checkpoint-pending conversion workloads and the next union/identity/property prerequisites are in [the value milestone](value-types-and-namespaces.md) and [full plan](../plans/javascript-parity-plan.md). The hourly monitor remains read-only and does not reinstall extensions.

## Compiler-backed punctuation quick fixes

Public SuggestedFix metadata, quickfix capability, current-snapshot validation and versioned documentChanges are implemented. The packaged VS Code provider applies a missing semicolon to unsaved Unicode source without losing its comment and clears the diagnostic. Focused compiler, adapter, transport, extension build/grammar and Clippy checks pass. Existing nullable-union fixtures in the broader editor suite remain failing against the current compiler parser; see [precise evidence and limits](../guides/tooling/editor-quick-fixes.md).

## Clone suggestions and single-file commands

Verified clone-at-transfer quick fixes, shared CLI/LSP whitespace formatting, lint alias, play-button Run File, Check/Lint File, Format Document and automatic save actions are implemented. Installed-VSIX E2E applies clone edits, formats unsaved source, runs compiler checks and a terminal execution task, and verifies format-on-save. The updated extension and local CLI are installed. Scope, settings, formatter invariants and precise acceptance limits are in [command documentation](../guides/tooling/editor-commands.md).

## Automatic entry-point documentation

Top-level function main receives explanatory hover/completion/signature documentation covering automatic execution through the play button or tsrust run, its zero-parameter/void contract and a runnable Hello example. Adjacent user documentation is preserved; local callable variables and record fields named main receive no entry-point note. All 32 intelligence tests pass, the documented Hello example executes natively, and debug/release LSP builds pass. The test file was restored from the gate84 accepted-source snapshot after an accidental overwrite; its entire prior source matches that snapshot before the new test.
