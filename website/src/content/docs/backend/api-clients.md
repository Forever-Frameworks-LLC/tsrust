---
title: Generate independent API clients
description: Generate checked TypeScript and async Rust clients without coupling them to server source.
---

The backend is the API source of truth. New applications select their public
operations and authorization in `tsrust.config.json` and import compiler-owned
handlers from `api:server`; checking and running never require generated clients
or dispatcher files. See [backend applications](../applications/).

Generate independent TypeScript or asynchronous Rust SDKs from a live service:

```sh
tsrust api generate --url http://127.0.0.1:4318/api/description --out js-client
tsrust api generate --url http://127.0.0.1:4318/api/description --language rust --out rust-client
```

Use `--description http-api.json` instead of `--url` for a saved description, without
backend source. The same checked description exports OpenAPI.

SDK generation does
not change application source, register operations or start a server.

URL and saved-description generation always require `--out`. For source-based
generation, `api.output` can supply the default destination.
Optional `api.server.output` retains explicit legacy dispatcher export, which new
applications do not use. Keep deployment compatibility baselines application-owned
rather than depending on a generated client directory.

## Generate from source configuration

Use the existing `api` settings in `tsrust.config.json`, or create a separate
`tsrust.api.json` for a client export. The following standalone configuration uses
the greeting function from the [backend starter](../applications/). Save it at
the starter project root and select it explicitly with `--config`:

```json
{
  "schemaVersion": 1,
  "apiVersion": 1,
  "entry": "app/api/operations.tsr",
  "output": "packages/api",
  "package": { "name": "@example/api", "version": "0.1.0" },
  "operations": [
    {
      "name": "greet",
      "module": "app/api/operations.tsr",
      "export": "greet",
      "kind": "query"
    }
  ]
}
```

The entry is checked through the normal module/compiler path and can be a library
module without an executable main function. Selected modules
must belong to that checked graph and selected names must be actual named function
exports. An ordinary export is never automatically an operation. Query/mutation
kind is explicit metadata, not proof that a query is side-effect-free. Application
access policy remains a server responsibility when dispatch is added.

```sh
tsrust api generate --config tsrust.api.json
tsrust api generate --config tsrust.api.json --check
tsrust api generate --config tsrust.api.json --out ./client-package
```

### Paths and configuration selection

Without `--config`, the CLI searches upward for API configuration. If the same
directory has both inline `api` settings and `tsrust.api.json`, it requires an
explicit selection. Configured
entry/module/output paths are relative to that file and exclude `.`/`..` and
backslashes; use portable forward slashes. An explicit output flag is relative to
the current directory and can select a standalone destination. Package
versions require `major.minor.patch`; prerelease labels and build metadata are
not supported. See [version rules](../api-compatibility/#choose-version-numbers).

## Understand generated output

The TypeScript output contains JavaScript ESM, TypeScript declarations, a manifest, package
metadata and the generated runtime's MIT notice. It has no install scripts or
framework dependencies. No backend bodies, source paths, credentials, database
dependencies or source-tree imports are copied into it. The package is private
by default. Review the generated files, package metadata and your own licensing
before enabling publication; generation/check/build never publishes anything.

### Regenerate safely

Existing nonempty application directories are not adopted as generated output.
Only previously generated files whose recorded hashes still match, or already
match the intended replacement, are replaced. Edited generated files produce a
diagnostic and remain untouched. Unrelated files remain user-owned. `--check`
does not write files and fails for stale outputs or markers. A lock excludes
concurrent generators, and files are individually replaced atomically. A failed
partial replacement can be retried using the old/planned hash checks.

## Calls and wire values

The TypeScript package exposes `createClient(transport)`, `ApiError`, a frozen
manifest and a contract identity. Named calls follow your selected operation
names: `greet` becomes `api.greet(...)`, while `tasks.list` would become
`api.tasks.list(...)`.

To try the generated starter client without a running server, save this as
`try-client.mjs` beside `js-client`, then run `node try-client.mjs`:

```js
import { createClient } from './js-client/index.js';

const api = createClient({
  async request(operation, input) {
    if (operation !== 'greet') throw new Error('Unexpected operation');
    return { ok: true, value: `Hello, ${input}!` };
  },
});
console.log(await api.greet('Ada')); // Hello, Ada!
```

This transport is a local demonstration. Use the HTTP adapter to call the service.

A transport implements
`request(operation, checkedInput, options)` and returns either
`{ok:true,value}` or `{ok:false,error:{code,message}}`. Input is validated and copied
before transport. Result validation rejects malformed values and unexpected
fields. Calls return promises; they add no caching, retries or implicit mutation
invalidation. Optional cancellation signals are forwarded to the transport and do
not imply rollback or backend cleanup completion.

Operations currently accept no arguments or one required data argument. Put
multiple inputs in a request record. Rest/default/optional top-level parameters
and unresolved generic functions require concrete wrapper functions. Async
operation signatures describe eventual data, not transportable tasks.

### Supported data

| Value | Contract |
| --- | --- |
| Strings | Well-formed Unicode; lone UTF-16 surrogates reject before transport. |
| Booleans and null | Preserved; null is distinct from a missing field. |
| Numbers | Finite values only. JSON encodes negative zero as zero. |
| Native integers | Must fit ±9,007,199,254,740,991; larger values reject instead of rounding. |
| Arrays and tuples | Dense arrays; tuple length and each element are checked. |
| Plain records | Declared data properties only; accessors and unknown keys reject. |
| Optional fields | Missing or undefined properties are omitted. Required fields must exist. |
| Literals and unions | Values must satisfy a declared alternative. |
| Void results | Encoded as null, returned as undefined by the TypeScript client. |

Special JSON keys remain data without changing object prototypes. Recursive or
deeper-than-128-level schemas, functions and native resources are rejected.
Collections and domain objects need explicit conversion to supported data;
there is no implicit serialization for them.

## Call the service over HTTP

The separate `@example/api/http` export uses the host Fetch API. The plain package
entry still requires no DOM types or Fetch globals. TypeScript HTTP consumers need their
host's Fetch type declarations, whether supplied by browser, Node or React Native
tooling. Pass an absolute endpoint URL:

With the starter listening on port 4318, save this as `try-http.mjs` beside
`js-client` and run it with a Node version that provides Fetch:

```js
import { createClient } from './js-client/index.js';
import { createHttpTransport } from './js-client/http.js';

const api = createClient(createHttpTransport({
  url: 'http://127.0.0.1:4318/api',
}));
console.log(await api.greet('Ada')); // Hello, Ada!
```

For an installed package, use its configured name, such as `@example/api` and
`@example/api/http`, instead of relative file imports. Protected services can
supply `headers` as an object or an async function returning headers. For example,
`headers: async () => ({ authorization: 'Bearer ' + await currentToken() })`
requires your own `currentToken` function. Pass `{ signal: controller.signal }`
as the second call argument when using an application-owned `AbortController`.

### Request headers and credentials

Each call posts `{operation,input}` as JSON to the configured endpoint and sends
`x-tsrust-contract` with a selected checked wire-profile identity. JSON Content-Type
parameters carry the requested API major, current operation fingerprint and current checked manifest identity. This adds no new CORS header names. The server retains
application authorization responsibility. Headers are resolved
for each call so credential changes take effect; the adapter sets its protocol
headers after application headers. Cookies are omitted and redirects reject.
Use HTTPS for remote credentials. Loopback HTTP is available for the desktop bridge.
URLs containing embedded credentials or fragments reject during construction.

### Errors and cancellation

Non-2xx responses can carry the same structured public failure envelope. A success
envelope on a failing HTTP status rejects. HTML, invalid JSON and invalid output
data reject with protocol/HTTP/validation errors. Host network exception details
are replaced with an `api-network` error; cancellation uses `api-cancelled`.
There are no automatic retries, including mutations. Cancellation requests abort
through the host Fetch API; it does not prove rollback or backend cleanup.

Supply `fetch` explicitly for a host-specific adapter or testing.

### Response size

Response text is read before parsing; this client
does not impose a streaming memory limit. Backend request/response bounds and
mobile Fetch behavior remain service/platform acceptance requirements.

For host Fetch usage, see
[the Fetch API documentation](https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API/Using_Fetch).

## Ordinary local package adoption

Generate into an explicit folder, then use the application's package manager:

```sh
tsrust api generate --config tsrust.api.json --out ./dist/client
cd dist/client
npm pack --ignore-scripts
cd /path/to/existing-app
npm install /path/to/project/dist/client/example-api-0.1.0.tgz --ignore-scripts
```

Replace the absolute paths above with your project and consumer paths.
The tarball filename follows the configured npm package name/version. Inspect
`npm pack --json --ignore-scripts` before sharing. Its allowlist includes runtime,
declarations, contract, README and license; generated ownership markers and
unrelated files are excluded. Generated packages are private by default; local
packing works without publication. Consumers need no compiler, backend source or
generated-directory symlink. Generated plain SDKs have no framework or cache peers.

### Workspace consumers

For an npm workspace, include the generated package directory in the root's
workspaces and declare its configured name/version in the consuming application's
normal dependencies. Users own their package manager, scripts and dependencies.
Other package managers use their ordinary package/workspace workflows; SDK
export never installs packages or rewrites the consuming application's files.

## Use the Rust client

The Rust export contains `Cargo.toml`, owned DTO types and asynchronous named
calls. Add it as a normal Cargo path dependency in your consumer. Its default
`http` feature supplies `HttpTransport`; disable default features when providing
a custom transport. Run HTTP calls inside your application's Tokio runtime.
Generation does not fetch Cargo dependencies.

Read the generated `README.md` and `src/lib.rs` for the package name and exact
operation types. Optional fields use `Option`; nullable unions preserve explicit
null separately. The Rust generator currently rejects tuples longer than 16
elements. Do not assume the TypeScript SDK's Fetch behavior describes the Rust
transport's limits.

## Next steps

- [Configure authorization and dispatch](../authorization/) on the server.
- [Retain compatible SDK contracts](../api-compatibility/) for rolling deployments.
