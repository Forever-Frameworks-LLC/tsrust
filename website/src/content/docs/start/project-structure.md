---
title: Understand your project
description: Find backend source, configuration, clients and disposable build outputs.
---

`tsrust new hello --app` creates a backend service with source-defined routes:

```text
app/
  main.tsr
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
Paths are relative to the configuration directory. Define routes with `app.get`,
`app.post` and the other Router methods in `app/main.tsr`. Register middleware and
authorization with `app.use`. Shared state belongs in `c.env`; ordinary exports do
not expose HTTP endpoints. The starter greeting is public; protect sensitive
routes before deploying them.

## Independent clients

Generate a TypeScript or asynchronous Rust client from the checked service source:

```sh
tsrust api generate --out js-client
tsrust api generate --language rust --out rust-client
```

An explicit entry file works without a project config. API generation requires no
endpoint list or middleware settings in JSON and does not need a running server.
Each client includes `http-api.json` and `openapi.json`; a saved contract can be
shared with another consumer instead of backend source. Generated directories are
outputs, not required backend source. Generation never changes routes or installs
frontend packages. Bring your own UI and call HTTP APIs.

See [the service guide](/tsrust/learn/first-service/#generate-api-clients) for client
calls, regeneration and deployment. Existing `api:server` applications retain
[configured RPC compatibility](/tsrust/tools/configured-rpc/).

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
