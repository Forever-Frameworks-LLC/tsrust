<!-- tsrust-docs
{"id":"project-structure","group":"getting-started","order":20,"status":"ready","targets":["web","native"]}
-->
# Project structure

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
