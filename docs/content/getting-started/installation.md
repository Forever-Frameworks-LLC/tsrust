<!-- tsrust-docs
{"id":"installation","group":"getting-started","order":10,"status":"ready","targets":["web","native"]}
-->
# Installation

The Tsrust CLI checks source, generates native programs and manages offline editable UI components. Native compilation uses the pinned Rust backend supplied by the toolchain.

## Create a project

```text
tsrust new my-app --ui --target native --target web
cd my-app
tsrust dev --target web --port 4317
```

Pass target flags for repeatable scripts. In a terminal, the CLI can ask for missing target choices interactively. Selecting web does not enable native or embedded implicitly.

Use `tsrust dev --target native` for the desktop application. With only one target enabled, `tsrust dev` selects it; projects with both targets default to native. Web development watches source and Markdown, reuses its compiler session and Wasm workspace, and reloads connected browsers after successful builds. Compilation errors keep the previous application available.

Web development requires the `wasm32-unknown-unknown` Rust target and `wasm-bindgen-cli` 0.2.129 installed locally. Dependencies build offline.

Native execution requires a graphical desktop and supported GPU. Custom native titlebars currently require macOS; other desktop platforms use the native titlebar.

## Add local components offline

```text
tsrust ui add button card input --offline
```

The bundled catalog needs no registry login or package-manager download. The first component installation creates missing UI configuration and source directories. Importing a UI component alone does not create project setup: follow the diagnostic's `tsrust ui add` command.

Installation preserves existing project source. A conflicting upgrade gets a versioned filename such as `card-v3.tsrx`; newly installed bundled dependencies use the selected file. Existing application imports remain under your control.

## Check the project

```text
tsrust check src
tsrust format --check
```

Checks and editor analysis remain read-only. See [project structure](project-structure.md) for the files you own, and [theming](../ui/foundations/theming.md) for customization.
