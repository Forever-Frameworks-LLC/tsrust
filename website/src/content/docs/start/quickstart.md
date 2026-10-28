---
title: Your first native program
description: Check and run a small Tsrust program, then create a backend service.
---

You need the `tsrust` CLI on your PATH. Official distributions pair it with
`tsrust-lsp`, the editor language server. Get the matching archive from
[GitHub Releases](https://github.com/Forever-Frameworks-LLC/tsrust/releases), or use
the repository's source-build instructions while no stable release is available.

## Prepare native development

Checking source needs no external Rust compiler. Running, building and testing a
native program require the pinned Rust toolchain, dependency sources and a host
linker. Install Rustup and your platform's C/C++ build tools, then explicitly run (see [native tool setup](../../tools/rust-setup/) for details):

```sh
tsrust setup --toolchain --dependencies --yes
tsrust doctor --native
```

Setup may download the requested tools and sources. Doctor checks the installation
and native host tools without installing anything. Ordinary native builds then
use the prepared sources offline.

## Check and run a program

Save this as `hello.tsr`:

```typescript
function main() {
  const languages = ['TypeScript', 'Rust'];
  console.log(languages.join(' + '));
}
```

```sh
tsrust check hello.tsr
tsrust run hello.tsr
```

The program prints:

```text
TypeScript + Rust
```

Create an optimized executable with `tsrust build hello.tsr --release -o hello`.
On Windows, use `-o hello.exe`. Both development and release programs retain
checked integer arithmetic.

## Create a backend service

```sh
tsrust new hello-api --app
cd hello-api
tsrust dev
```

The starter serves a greeting API on loopback port 4318. Its source, operations and
access policy live under `app/`. Creation refuses an existing destination and does
not install dependencies. Read `app/api/access.tsr` before exposing the service:
the greeting operation is explicitly public.

Use `tsrust check`, `tsrust test` and `tsrust format --check` at the project root.
`dev` rebuilds on source changes and keeps the last working process if a build
fails. Ctrl+C stops the watcher and its owned processes.

For a console project instead, run `tsrust new hello-console` without `--app`.

## Generate an independent client

While the starter is running, use a second terminal:

```sh
tsrust api generate --url http://127.0.0.1:4318/api/description --out js-client
tsrust api generate --url http://127.0.0.1:4318/api/description --language rust --out rust-client
```

Generation writes to the selected client directory without changing the server.
Remote descriptions require HTTPS; HTTP is allowed on loopback. Authentication
and deployment configuration remain application responsibilities.
