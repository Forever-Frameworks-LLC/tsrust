---
title: Understand your project
description: Find backend source, configuration, clients and disposable build outputs.
---

`tsrust new hello --app` creates a backend application with explicit configuration:

```text
app/
  main.tsr
  api/
    operations.tsr
    context.tsr
    access.tsr
  lib/
    greeting.tsr
  tests/
    greet.test.tsr
tsrust.config.json
.gitignore
AGENTS.md
README.md
```

The generated `.gitignore` excludes `/target/` and `.env.local`.

## Entrypoint and targets

`build.entry` names `app/main.tsr`; `build.output` names the native executable.
Paths are relative to the configuration directory. The explicit `api` section
selects operations, context and access policy. Ordinary exports stay private
unless selected. The starter greeting is explicitly public; review its access
policy before adding sensitive operations.

The server imports dispatch, description and OpenAPI handlers from `api:server`.
The compiler validates and supplies them in memory. Running the backend does not
require client generation or a checked-in generated dispatcher.

## Independent clients

Generate a TypeScript or asynchronous Rust SDK from the running service's HTTP
description, or a saved description, into a caller-selected directory. Examples
are `js-client/`, `rust-client/` or an external consumer project. These are outputs,
not required backend source directories. Generation does not modify the server
or install frontend packages. Bring your own UI and call the HTTP API.

See the [backend application guide](../../backend/applications/)
for commands, access policy, retained compatibility contracts and OpenAPI scope.

## Console projects

`tsrust new hello` creates `src/main.tsr`, a test under `tests/`, project
configuration and guidance. The explicit executable entry remains authoritative;
there is no implicit component or function-name entry discovery.

## Build and cache ownership

`target/` holds disposable build outputs. Ordinary native compilation uses shared
Cargo dependency sources and locked, offline generated workspaces. Source/setup
commands are explicit. Binding configuration, exact locks and Rust adapters remain
application-owned. Portable exports contain independently verified sources, notices
and offline Cargo configuration and can be relocated for rebuilding.
