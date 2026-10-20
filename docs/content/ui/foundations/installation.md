<!-- tsrust-docs
{"id":"reference-ui-foundations-installation","group":"foundations","order":20,"status":"partial","targets":["web","native"]}
-->
# Offline UI installation

```text
tsrust ui add button card input --offline --target web --target native
```

The first `ui add` creates missing project setup. An ordinary import does not silently create directories or configuration. Interactive target selection is available when explicit flags are omitted.

Source installs under the configured local components directory, usually `src/components/ui`. Project tokens and explicit targets live in `tsrust.config.json`. Dependencies and file provenance are recorded in the lock file; notices are deduplicated under `attributions/`.

Conflicting upgrades preserve existing files and application imports. Newly installed bundled dependencies refer to the selected versioned file. Read [project structure](/getting-started/project-structure) before moving installed files.
