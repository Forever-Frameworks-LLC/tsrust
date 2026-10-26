# Backend dependency isolation: first section

Date: 2026-10-26. Baseline implementation: `89656526`; planning checkpoint:
`c3813d68`. Status: focused validation in progress; this is not completion of the
backend application prototype or complete removal of legacy UI coupling.

## Changes

The owner approved the native-backend pivot and clarified that frontend projects
remain user-owned. Active goals, roadmap, root instructions and old UI plan banners
now identify the new scope. Preserve old renderer/component source and historical
evidence in place. No archives, dependency installations or public actions are
part of this section. Package research and ownership boundaries are recorded in
[the package decision record](../plans/backend-platform-packages.md).

The default workspace members are now compiler and LSP. The UI crate remains an
explicit workspace member; `--workspace` still intentionally includes it. A new
UI `runtime` feature preserves the previous default runtime behavior and is
implied by `native` and `ui-web`. Compiler dependencies disable UI defaults and
retain style/theme validation without node/state/event/navigation/vector runtime
compilation. URL, Kurbo and SVG parsing dependencies are conditional on runtime.
The static example explicitly requires runtime.

The embedded UI source emitter removes library-only runtime selection guards and
maps remaining runtime test guards to its existing generated `ui` feature. This
preserves generated legacy UI source rather than making old UI programs silently
emit an empty runtime. Only node-dependent tests require runtime; pure authoring
tests remain usable without it. A focused public authoring witness exercises theme
colors, invalid class rejection, documentation and completion without runtime.

## Validation

- `cargo tree --offline --locked -p tsrust-compiler -e normal --depth 2 --prefix none --format '{p} features={f}'`
  shows `tsrust-ui` with no enabled features and only color/serde/serde_json below
  that dependency. URL/vector/runtime dependencies are absent from that subtree.
  This does not assert that unrelated HTTP dependencies never use URL handling.
- `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo check --offline --locked -p tsrust-compiler -p tsrust-lsp --lib --bins`
  passes in 4m24s. This is build/type-check evidence, not an editor protocol pass.
- Initial no-runtime tests passed 7 theme tests and one authoring integration test.
  After retaining additional pure authoring tests, the final no-runtime lib run
  passes 19 tests plus the authoring integration witness. The initial integration
  compile correctly rejected use of private CSS helpers; the witness now uses
  the public compiler-facing authoring API instead.
- The preserved default runtime lib suite passes 104 tests plus the authoring
  integration witness. GPU/window, browser interaction and Table acceptance were
  not resumed or claimed.
- Generated static/native source and generated HTML execution checks are pending.
- The initial whole-project formatter reported zero changed files/fingerprints.
  Final whole-project formatting passes twice with zero changed files/fingerprints;
  local checkpoint follows. Generated compatibility checks remain pending.

Commands use the shared target, one Cargo job and disabled incremental compilation.
The compiler check's sampled CLI rustc private memory reached 1,125 MiB; this is an
observation, not a maximum or benchmark. A redundant automatic editor
`cargo check --workspace --all-targets` was cancelled to avoid an unsolicited
parallel broad build; the editor host was preserved. No full regression/performance
milestone ran and no existing benchmark sample changed.

## Remaining coupling and next section

This is feature isolation, not a separate compiler distribution. Compiler
`ui_project` and JSX validation still depend on the frozen style/theme module.
The generated UI emitter still embeds source from `crates/tsrust-ui/src`. The CLI
still carries legacy UI commands/catalog/docs integration. `build.rs` still vendors
the complete locked `native-dependencies` source closure; the CLI embeds those
bytes for offline builds even when a generated backend does not activate UI
features. Therefore no compiler binary-size/build-time saving is claimed.

Next trace the offline bundle identities, source selection and cache consumers
before introducing an optional legacy-UI distribution boundary. Preserve offline
integrity, deterministic identities and legacy exports; do not delete vendored
sources or silently weaken compatibility. This deeper split can be scheduled
alongside the first checked operation-manifest/client proof when a dependency
requires it. The next product section is the SQLite task service and generated
HTTP client, not another renderer gate.

Current concise logs are under `target/backend-isolation-*.log`; they are disposable
current-gate diagnostics, not analytics imports. Preserve unrelated
`docs/src/main.tsr` edits and untracked `.agents/`.
