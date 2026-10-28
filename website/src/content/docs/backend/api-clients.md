---
title: Generate independent API clients
description: Generate checked TypeScript and async Rust clients without coupling them to server source.
---

The backend is the API source of truth. New applications select their public
operations and authorization in tsrust.config.json and import compiler-owned
handlers from `api:server`; checking and running never require generated clients
or dispatcher files. See [backend applications](../applications/).

Generate independent TypeScript or asynchronous Rust SDKs from a live service:

```sh
tsrust api generate --url http://127.0.0.1:4318/api/description --out js-client
tsrust api generate --url http://127.0.0.1:4318/api/description --language rust --out rust-client
```

Use `--description http-api.json` instead of --url for a saved description, without
backend source. The same checked description exports OpenAPI. SDK generation does
not change application source, register operations or start a server. Optional
api.output configures a client export destination; otherwise --out is required.
Optional server.output retains explicit legacy dispatcher export, which new
applications do not use. Keep deployment compatibility baselines application-owned
rather than depending on a generated client directory.

For source-based SDK export without HTTP, the retained separate configuration also
works:

Declare the operations to expose in a separate `tsrust.api.json`. Keeping this
configuration separate allows backend/client-only adoption without changing an
existing frontend's package manager, lockfile, scripts or directory layout.

```json
{
  "schemaVersion": 1,
  "apiVersion": 1,
  "entry": "app/main.tsr",
  "output": "packages/api",
  "package": { "name": "@example/api", "version": "0.1.0" },
  "operations": [
    { "name": "tasks.list", "module": "app/tasks.tsr", "export": "list", "kind": "query" }
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
tsrust api generate
tsrust api generate --check
tsrust api generate --config ./service/tsrust.api.json --out ./client-package
```

Without `--config`, the CLI finds the nearest ancestor configuration. Configured
entry/module/output paths are relative to that file and exclude `.`/`..` and
backslashes; use portable forward slashes. An explicit output flag is relative to
the current directory and can select a standalone destination. Initial package
versions use ordinary `major.minor.patch`; prerelease versions are a later addition.

The output contains JavaScript ESM, TypeScript declarations, a manifest, package
metadata and the generated runtime's MIT notice. It has no install scripts or
framework dependencies. No backend bodies, source paths, credentials, database
dependencies or source-tree imports are copied into it. The package is private
by default. Review the generated files, package metadata and your own licensing
before enabling publication; generation/check/build never publishes anything.

Existing nonempty application directories are not adopted as generated output.
Only previously generated files whose recorded hashes still match, or already
match the intended replacement, are replaced. Edited generated files produce a
diagnostic and remain untouched. Unrelated files remain user-owned. `--check`
does not write files and fails for stale outputs or markers. A lock excludes
concurrent generators, and files are individually replaced atomically. A failed
partial replacement can be retried using the old/planned hash checks.

## Calls and wire values

The initial package exposes `createClient(transport)`, `ApiError`, a frozen
manifest and an exact contract hash. Named calls follow the selected namespaces:

```ts
const api = createClient(transport);
const tasks = await api.tasks.list({ search: null });
```

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

Supported wire values are strings, booleans, finite numbers, JavaScript-safe
integers, null, checked literals, dense arrays, tuples, plain data records and
unions. Optional fields omit missing/undefined properties; required fields must
exist and null remains distinct. Native integers outside plus/minus
9,007,199,254,740,991 fail validation instead of rounding. Void results use null
on the wire and become undefined to the caller. Object fields are data properties;
accessors and unknown keys reject. Special JSON keys remain data without changing
object prototypes. Recursive/deeper-than-128-level schemas, functions, native
resources and implicit collection/domain serialization reject; project them into
explicit supported data before exporting. Strings must contain well-formed Unicode:
lone UTF-16 surrogates reject before transport because native JSON strings use UTF-8.
JSON encodes negative zero as zero; it is not a distinct wire value.

## HTTP transport

The separate `@example/api/http` export uses the host Fetch API. The plain package
entry still requires no DOM types or Fetch globals. HTTP consumers need their
browser/React Native Fetch type declarations. Pass an absolute endpoint URL:

```ts
import { createClient } from '@example/api';
import { createHttpTransport } from '@example/api/http';

const api = createClient(createHttpTransport({
  url: 'http://127.0.0.1:4318/api',
  headers: async () => ({ authorization: `Bearer ${await currentToken()}` }),
}));
const count = await api.math.count(1, { signal: controller.signal });
```

Each call posts `{operation,input}` as JSON to the configured endpoint and sends
`x-tsrust-contract` with a selected checked wire-profile identity. JSON Content-Type
parameters carry the requested API major, current operation fingerprint and SDK
artifact fingerprint. This adds no new CORS header names. The server retains
application authorization responsibility. Headers are resolved
for each call so credential changes take effect; the adapter sets its protocol
headers after application headers. Cookies are omitted and redirects reject.
Use HTTPS for remote credentials. Loopback HTTP is available for the desktop bridge.
URLs containing embedded credentials or fragments reject during construction.

Non-2xx responses can carry the same structured public failure envelope. A success
envelope on a failing HTTP status rejects. HTML, invalid JSON and invalid output
data reject with protocol/HTTP/validation errors. Host network exception details
are replaced with an `api-network` error; cancellation uses `api-cancelled`.
There are no automatic retries, including mutations. Cancellation requests abort
through the host Fetch API; it does not prove rollback or backend cleanup. Supply `fetch` explicitly for a
host-specific adapter or testing. Response text is read before parsing; this client
does not impose a streaming memory limit. Backend request/response bounds and
mobile Fetch behavior remain service/platform acceptance requirements.

Fetch status checking, JSON content types, credentials and cancellation follow
[the Fetch API documentation](https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API/Using_Fetch).

## Ordinary local package adoption

Generate into an explicit folder, then use the application's package manager:

```sh
tsrust api generate --config tsrust.api.json --out ./dist/client
cd dist/client
npm pack --ignore-scripts
cd /path/to/existing-app
npm install /path/to/client/example-query-api-0.1.0.tgz --ignore-scripts
```

The tarball filename follows the configured npm package name/version. Inspect
`npm pack --json --ignore-scripts` before sharing. Its allowlist includes runtime,
declarations, contract, README and license; generated ownership markers and
unrelated files are excluded. Generated packages are private by default; local
packing works without publication. Consumers need no compiler, backend source or
generated-directory symlink. Generated plain SDKs have no framework or cache peers.

For an npm workspace, include the generated package directory in the root's
workspaces and declare its configured name/version in the consuming application's
normal dependencies. Users own their package manager, scripts and dependencies.
Other package managers use their ordinary package/workspace workflows; SDK
export never installs packages or rewrites the consuming application's files.


## Next steps

- [Configure authorization and dispatch](../authorization/) on the server.
- [Retain compatible SDK contracts](../api-compatibility/) for rolling deployments.
