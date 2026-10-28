# Retained original UI tooling

The original Tsrust UI source, documentation app and explicit UI commands remain
available for existing projects. They are outside the active backend product;
new applications bring their own UI and call the native HTTP API. These are retained
setup instructions, not a new UI completeness or platform-validation claim.

First follow [CLI installation](../../../website/src/content/docs/start/installation.md).
Original UI builds use the optional full source pack; prepare it explicitly:

```sh
tsrust setup --portable --yes
```

Run `tsrust` in a terminal for the interactive launcher. For UI, select targets
explicitly:

```sh
tsrust new my-app --ui --target native --target web
cd my-app
tsrust dev --target native
```

Pass target flags for repeatable scripts. In a terminal, the CLI can ask for
missing target choices interactively. Selecting web does not enable native or
embedded implicitly. Native execution requires a graphical desktop and supported
GPU. Custom native titlebars currently require macOS; other desktop platforms
use the native titlebar.

## Browser prerequisites

Web UI uses WebAssembly and the same checked component source. Install these
additional tools explicitly:

```sh
rustup target add wasm32-unknown-unknown --toolchain 1.96.1
cargo +1.96.1 install wasm-bindgen-cli --version 0.2.129 --locked
tsrust dev --target web --port 4317
```

Web development watches source and Markdown, reuses the compiler session and Wasm
workspace, and reloads browsers after successful builds. Failed builds keep the
previous application available. Dependencies compile offline after setup.

## Add components offline

```sh
tsrust ui add button card input --offline
```

The bundled catalog needs no registry or package-manager download. The first
installation creates missing UI configuration and source directories. Importing
a component alone does not create setup: follow the diagnostic's `tsrust ui add`
command. The catalog is still being completed; inspect `tsrust ui list --bundled`
for the actual available components.

Installation preserves existing source. A conflicting upgrade gets a versioned
filename such as `card-v3.tsrx`; newly installed bundled dependencies use the
selected file. Existing application imports stay under your control.


## Original UI project layout

A UI project has an explicit bootstrap, application source and editable local components. Configuration lives in `tsrust.config.json`.

```text
src/
  main.tsr
  app/
    page.tsrx
  components/
    ui/
      button.tsrx
tsrust.config.json
ui-components.lock.json
attributions/
```

## Entrypoint and targets

`build.entry` names the bootstrap, usually `src/main.tsr`. That bootstrap creates the root page and calls `run`. The root page convention is `src/app/page.tsrx`; it does not replace the explicit entrypoint.

`ui.target.web`, `ui.target.native` and `ui.target.embedded` are independent booleans. Embedded rendering is not implemented by selecting the flag. The current checked paired slice supports web and native.

## Editable components

`ui.components` normally points at `src/components/ui`. Installed source is yours to change. The lock records installation provenance; it does not make component files immutable.

Shared third-party notices live under `attributions/`. Short references in component files point at those notices. Preserve the notices when distributing adapted source.

## Documentation content

This documentation project keeps authored Markdown under `content/` and actual examples under `src/examples/`. A checked content build produces ignored files under `target/docs-content/`.

```text
tsrust docs content
```

The generated typed module owns a checked content snapshot. It does not read Markdown, execute fences or fetch packages in the browser. Edit the original Markdown/example/component files and rebuild; generated files are not another authoring source.
