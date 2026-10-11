# Tsrust commands, formatting and clone fixes

`tsrust run` and the editor play button keep generated Rust, the executable and default cache in a private system temporary directory, removed when the command finishes (including ordinary errors). They leave no build artifacts beside the source. Force-killing the compiler or shutting down the system can interrupt cleanup. `build -o` preserves the requested executable and caches successful artifacts in `target/tsrust-native` under the working directory. Set `TSRUST_CACHE_DIR` to explicitly retain a reusable cache for either command; that directory is not automatically removed. Checks still run before reuse, and the pinned backend is verified. Source graph, generated Rust, native features, backend identity and optimization options invalidate old artifacts. Missing or corrupt cache entries rebuild; cache write failure does not invalidate a successful build. Remove the cache directory to clear it. This is a local artifact cache, not a package download mechanism.

Workspace diagnostics scan `.tsr` files recursively, including unopened files.
They skip `.git`, `node_modules`, `target` and symlinks during discovery. Open
buffers override disk contents, including unsaved new files. File watcher changes
and workspace folder changes schedule a fresh scan; deleted files and removed
folders have their old diagnostics cleared. Opening a file outside the workspace
still checks that file. Syntax recovery reports multiple statement errors. Semantic checking also reports
independent type and ownership errors, suppressing follow-on name errors from an
invalid declaration. Recovery currently stops at invalid returns and control jumps.


## Using the editor

Run terminals retain output after the process exits; no acknowledgement is needed to finish the run. Command echo and the terminal-reuse prompt remain hidden. Set `tsrust.run.closeOnSuccess` to true only if automatic terminal closure is desired. Lint uses the Output panel and formatting runs through the editor; neither needs a terminal acknowledgement. Reload the VS Code window after installing the updated extension. The terminal presentation change passed extension type checking and packaging; interactive terminal behavior has not been manually verified in the running window.

Open a saved .tsr file in a trusted workspace. The editor title's play button and **Tsrust: Run File** save the current file and execute it in a dedicated terminal. Stop it with VS Code's task controls. **Tsrust: Check File** saves and runs syntax/type/ownership checking, showing results in the Tsrust output channel. Live diagnostics continue while typing. **Tsrust: Format Document**, or VS Code's standard Format Document, uses the same formatter as the CLI.

`tsrust.compiler.path` optionally selects the compiler executable. Otherwise it uses tsrust beside the configured language server. `tsrust.run.release` selects optimized execution. `tsrust.formatOnSave` formats before saving; `tsrust.checkOnSave` checks after saving. Both are enabled for this repository workspace. Other workspaces can opt in. Do not also configure a second format-on-save provider for the same document.

Run uses argument-based ProcessExecution, without shell interpolation, so spaces and shell characters in filenames remain data. The working directory is the source file's directory. Command execution requires workspace trust. Cancelling a save prevents execution. Run requires the existing configured Rust backend; check/format do not build an executable or download tools.

## CLI

The local CLI is installed in the user's Cargo bin directory:

```sh
tsrust run path/to/file.tsr
tsrust run path/to/file.tsr --release
tsrust check                            # recursively check .tsr files under cwd
tsrust check src                        # recursively check a directory
tsrust check path/to/file.tsr
tsrust check path/to/file.tsr --json
tsrust format                           # recursively update .tsr files under cwd
tsrust format src                       # recursively update .tsr files under src
tsrust format --check                   # report files needing changes; do not write
tsrust format path/to/file.tsr          # formatted text on stdout
tsrust format path/to/file.tsr --write  # update source in place
tsrust format path/to/file.tsr --check  # exit 1 if formatting differs
```

Recursive formatting skips `.git`, `node_modules`, `target` and symbolic links. It sorts and deduplicates discovered files, rewrites only changed files, and prints one short write summary. `--check` lists changed files and returns 1 when changes are needed. Lexical or filesystem failures return 1; an invalid source does not prevent other discovered files from being formatted. Explicit single-file printing, `--write` and `--check` retain their existing behavior.

Check uses compiler syntax, type and ownership checks. A single-file check retains detailed contract output; `check --json` retains structured results. With no paths it scans the current directory recursively; explicit directories and multiple paths are also supported. A manual editor check shows compiler results; successful save-time checks stay silent and never reveal the Output panel. Files with errors return a nonzero exit code. Run preserves the program's exit code and terminal streams. Named project/package scripts remain separate future work.

## Formatting contract

The formatter normalizes spacing, expands braces and top-level semicolon boundaries, and uses two-space indentation. Intentional blank lines between statements are preserved, including leading and trailing blank lines. It inserts blank lines between functions and after completed blocks where separation is missing. It preserves the exact spelling/content of string/template tokens and retains comment trivia; comment placement can limit surrounding layout normalization. The entire output must tokenize to the same token kinds and values as the input. Unsupported token-changing formatting and invalid lexical input fail before --write changes the file. `as` and `satisfies` stay attached to the preceding expression; callback types have a space after those keywords. Generic type applications and explicit generic calls keep tight brackets, while comparisons and shifts retain operator spacing. Nested array types and chained indexes stay attached. Blank lines inside these expression continuations are normalized. Formatting is idempotent on accepted output. It does not fix syntax or ownership errors, impose stylistic lint warnings, or change program values. Layout options and sophisticated comment reflow remain future work.

## Clone suggestions

On a transferred string, array or record, Cmd+. can offer **Clone before transferring (copies data; may allocate)**. It wraps the earlier transfer in clone(...), preserving the original value for later use. This is an explicit independent copy, not aliasing. Only a current-version edit that passes the complete compiler check is offered. Callable consumption, unsupported partial-field moves and files with unrelated errors do not receive this fix.

## Acceptance

Focused compiler tests cover formatter idempotence, strings, Unicode, comments, continuations, templates, loops, negative values, CLI format/check/write/lint/run with space-containing paths, and native mutation independence after applying the clone edit. Code-action tests check clone metadata, the exact earlier transfer and validation. Framed transport and earlier punctuation guards remain passing. The installed-VSIX E2E requests/applies clone fixes, invokes formatting and compiler checking, runs a file through a terminal task and verifies zero process exit, and verifies formatting before save. Existing editor lifecycle/navigation/diagnostic checks remain in that E2E run.

Formatter and editor commands have no new generated runtime operation requiring throughput measurement. Full compiler regression and performance timing follow meaningful feature-family milestones in the development workflow. Concurrent nullable-union compiler code currently triggers two unrelated Clippy warning classes (collapsible_if and nonminimal_bool); the changed editor code is checked with only these classes suppressed until that work settles. Do not describe that as a clean unrestricted workspace Clippy pass.

## Terminal diagnostics

CLI source diagnostics use aligned line-number gutters, highlighted primary and related locations, and paths relative to the working directory when possible. Colors are automatic on a terminal; redirected streams stay plain. `--color auto|always|never` overrides selection. NO_COLOR and TERM=dumb disable automatic coloring. JSON stays uncolored regardless of flags. Source tabs expand for caret alignment and control characters are escaped in excerpts. VS Code run tasks hide the raw command echo and terminal-reuse message; failure exit status remains visible. Focused terminal rendering/CLI policy tests, extension build/grammar/packaging and workspace format checks pass. The user's failing ownership fixture was inspected with the new renderer.

Multiple successive supported transfers can be repaired by one clearly labelled combined clone action, verified against the entire edited document and limited to eight transfer sites.

## Module execution foundation

The development CLI now checks, emits, builds and runs entry files with explicit named relative imports, for example `import { value } from "./dependency.tsr";` and `export function value(): int { return 7; }`. Only the selected entry's `main()` executes automatically. Dependencies retain private names and defining-module record identity. Aliases are supported; import cycles, private/missing exports and source overwrite targets produce errors. Bundled `std:` modules are supported; bare package imports remain future work. Compile/runtime failures identify the original imported file. Check reports diagnostics without building an executable.

This compiler increment does not supply project-aware live IntelliSense or unsaved dependency checking yet; those are S04 in the standard-library roadmap. No VSIX installation or installed global CLI refresh was performed for this increment; use the development binary in target/editor/debug after building.

## Missing native linker

Build/run reports a missing native linker as an environment setup failure and includes the backend error, rather than calling it a compiler defect. Finding rustup under the Cargo home is independent of finding the platform linker on PATH. See [missing linker diagnostics](missing-linker-diagnostic.md) for setup and focused acceptance evidence.


Contributor checkpoints run `node tools/format-project.mjs` to format the entire
repository and refresh verified formatting fingerprints. Source and execution
checks then consume formatted fixtures. The formatter is shared by CLI commands
and the language server, so its output must preserve the program's behavior.
