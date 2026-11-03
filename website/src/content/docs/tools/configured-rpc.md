---
title: "Configured RPC compatibility"
description: "Maintain existing api:server applications and versioned RPC clients."
---

This page covers existing `api:server` RPC applications. New services use
[source-defined Router routes and middleware](/tsrust/learn/first-service/) and
generate clients from checked source, without API configuration. Existing RPC
configuration, versioned contracts and clients remain supported. These settings
apply to that RPC protocol; they are not required for ordinary HTTP services.

## Authorize requests

Select the operations clients may call and supply an application-owned access
policy. Exporting a function alone does not expose it over HTTP. Use these settings only when maintaining a configured RPC application.

### Configure the server

In `tsrust.config.json`, the `api.server` object identifies the context type and
authorization function. This fragment uses example RPC paths; merge it into
its existing `api` settings rather than replacing the whole configuration:

```json
{
  "server": {
    "context": {
      "module": "app/api/context.tsr",
      "export": "AppContext"
    },
    "authorize": {
      "module": "app/api/access.tsr",
      "export": "authorize"
    },
    "maxRequestBytes": 65536,
    "maxResponseBytes": 65536
  }
}
```

Paths are relative to the configuration directory. Use forward slashes, without
`.` or `..` path segments. Both byte limits default to 65536 and accept integers
from 1 through 16777216.

Import `dispatch`, `describe` and `openapi` from `api:server` and mount only the
handlers you need on your Router. Earlier RPC starters mounted all three. They share the
same authorization hook, including requests for the public API description.
Compilation checks these handlers in memory; it needs no generated dispatcher
file or client output directory.

### Make an access decision

The configured authorization function receives the native HTTP context and
returns `Task<boolean>`. It runs before dispatch reads or decodes the request
body or calls an operation:

- Return `true` to allow the request.
- Return `false` to send HTTP 403 with `api-denied`.
- Throwing sends HTTP 500 with `api-internal`, without exposing the exception.

Earlier RPC starters returned `true`, so its greeting API is public. Replace that policy
before exposing protected operations. Application credentials, state and policy
stay in backend modules; the generated SDK does not implement authentication
for you. See the repository's
[application-owned bearer policy](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/examples/backend-tasks/access.tsr)
for a larger example with origin checks and explicit CORS preflight handling.

### Pass private context to an operation

Add `"context": true` to an entry in `api.operations` when its first parameter
is `ReadOnly<Context<...>>` or `Mutable<Context<...>>`, including a named alias.
The compiler supplies that parameter on the server. It is absent from the wire
input and does not change the public contract identity.

The remaining input is zero or one supported plain-data argument. An arbitrary
pool/resource or an owned `Context` cannot be hidden as the context parameter.
Keep resources in application state and pass only supported data to clients.

### Bound requests and responses

`maxRequestBytes` limits the UTF-8 request text's byte length after it has
been read. `maxResponseBytes` bounds serialization of operation results and their
success envelopes. These guards do not replace the Router's admission limits.
Configure its body limit, concurrency and timeouts too; see
[listener limits](/tsrust/learn/first-service/#listener-limits).

Dispatch requires POST and `application/json`. It admits the selected operation
only when its contract identity or versioned per-operation fingerprint matches a
supported profile. Use a generated client to supply this protocol metadata.

| Condition | HTTP status | API error code |
| --- | --- | --- |
| Policy denies access | 403 | `api-denied` |
| Method is not POST | 405 | `api-method` |
| Missing or unsupported content type | 415 | `api-media-type` |
| Body exceeds the API request limit | 413 | `api-request-limit` |
| Invalid request or operation arguments | 400 | `api-validation` |
| Operation is undeclared | 404 | `api-operation` |
| Incompatible operation profile | 409 | `api-incompatible` |
| Result fails its checked schema | 500 | `api-result` |
| Policy, operation or result serialization throws | 500 | `api-internal` |

Router admission failures can occur before these handlers run. The table covers
ordinary dispatcher failures, not every malformed protocol-metadata case.
Public errors use bounded messages rather than exposing backend exception text.
Typed domain-result unions remain successful response data. Failure or
cancellation does not imply rollback: an operation may already have changed state.

### Retain older contracts

List reviewed baseline files in `api.compatible`; `api.server.compatible` remains
supported for older configurations. Keep each baseline immutable.

Compatibility checks require old inputs to fit the current operation and current
results to project into the old client's strict shape. SDK history also checks
whether old replica results satisfy a new client. Unknown or incompatible shapes
are rejected before invocation. Schema compatibility cannot prove unchanged
business behavior; see [versioning and rollout](/tsrust/tools/configured-rpc/#evolve-the-api).

### Export a dispatcher for an existing project

Normal `api:server` compilation needs no physical dispatcher. For an existing
project that requires one, set `api.server.output` to a private directory such
as `app/generated-api`, then explicitly run `tsrust api generate`.

Keep that private directory separate from the public client package. Generation
checks the handler and output ownership before replacing files. Generated hash
markers and locks protect both destinations; edited files are preserved.
Replacements are individually atomic and retryable, not one transaction across
the filesystem.

`--check` writes nothing. `--out` overrides the client destination while the
configured private server keeps its own destination. Omit `server` for
client-only generation from source.

## Generate API clients

The backend is the API source of truth. Existing RPC applications select their public
operations and authorization in `tsrust.config.json` and import compiler-owned
handlers from `api:server`; checking and running never require generated clients
or dispatcher files. See [backend applications](/tsrust/learn/first-service/#create-the-service).

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

### Generate from source configuration

Use the existing `api` settings in `tsrust.config.json`, or create a separate
`tsrust.api.json` for a client export. The following standalone configuration uses
an existing checked RPC function. Save it at
the RPC project root and select it explicitly with `--config`:

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

#### Paths and configuration selection

Without `--config`, the CLI searches upward for API configuration. If the same
directory has both inline `api` settings and `tsrust.api.json`, it requires an
explicit selection. Configured
entry/module/output paths are relative to that file and exclude `.`/`..` and
backslashes; use portable forward slashes. An explicit output flag is relative to
the current directory and can select a standalone destination. Package
versions require `major.minor.patch`; prerelease labels and build metadata are
not supported. See [version rules](/tsrust/tools/configured-rpc/#choose-version-numbers).

### Understand generated output

The TypeScript output contains JavaScript ESM, TypeScript declarations, a manifest, package
metadata and the generated runtime's MIT notice. It has no install scripts or
framework dependencies. No backend bodies, source paths, credentials, database
dependencies or source-tree imports are copied into it. The package is private
by default. Review the generated files, package metadata and your own licensing
before enabling publication; generation/check/build never publishes anything.

#### Regenerate safely

Existing nonempty application directories are not adopted as generated output.
Only previously generated files whose recorded hashes still match, or already
match the intended replacement, are replaced. Edited generated files produce a
diagnostic and remain untouched. Unrelated files remain user-owned. `--check`
does not write files and fails for stale outputs or markers. A lock excludes
concurrent generators, and files are individually replaced atomically. A failed
partial replacement can be retried using the old/planned hash checks.

### Calls and wire values

The TypeScript package exposes `createClient(transport)`, `ApiError`, a frozen
manifest and a contract identity. Named calls follow your selected operation
names: `greet` becomes `api.greet(...)`, while `tasks.list` would become
`api.tasks.list(...)`.

To try the generated RPC client without a running server, save this as
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

#### Supported data

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

### Call the service over HTTP

The separate `@example/api/http` export uses the host Fetch API. The plain package
entry still requires no DOM types or Fetch globals. TypeScript HTTP consumers need their
host's Fetch type declarations, whether supplied by browser, Node or React Native
tooling. Pass an absolute endpoint URL:

With the RPC service listening on port 4318, save this as `try-http.mjs` beside
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

#### Request headers and credentials

Each call posts `{operation,input}` as JSON to the configured endpoint and sends
`x-tsrust-contract` with a selected checked wire-profile identity. JSON Content-Type
parameters carry the requested API major, current operation fingerprint and current checked manifest identity. This adds no new CORS header names. The server retains
application authorization responsibility. Headers are resolved
for each call so credential changes take effect; the adapter sets its protocol
headers after application headers. Cookies are omitted and redirects reject.
Use HTTPS for remote credentials. Loopback HTTP is available for the desktop bridge.
URLs containing embedded credentials or fragments reject during construction.

#### Errors and cancellation

Non-2xx responses can carry the same structured public failure envelope. A success
envelope on a failing HTTP status rejects. HTML, invalid JSON and invalid output
data reject with protocol/HTTP/validation errors. Host network exception details
are replaced with an `api-network` error; cancellation uses `api-cancelled`.
There are no automatic retries, including mutations. Cancellation requests abort
through the host Fetch API; it does not prove rollback or backend cleanup.

Supply `fetch` explicitly for a host-specific adapter or testing.

#### Response size

Response text is read before parsing; this client
does not impose a streaming memory limit. Backend request/response bounds and
mobile Fetch behavior remain service/platform acceptance requirements.

For host Fetch usage, see
[the Fetch API documentation](https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API/Using_Fetch).

### Ordinary local package adoption

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

#### Workspace consumers

For an npm workspace, include the generated package directory in the root's
workspaces and declare its configured name/version in the consuming application's
normal dependencies. Users own their package manager, scripts and dependencies.
Other package managers use their ordinary package/workspace workflows; SDK
export never installs packages or rewrites the consuming application's files.

### Use the Rust client

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

### Next steps

- [Configure authorization and dispatch](/tsrust/tools/configured-rpc/#authorize-requests) on the server.
- [Retain compatible SDK contracts](/tsrust/tools/configured-rpc/#evolve-the-api) for rolling deployments.

## Evolve the API

Keep old clients working while deploying new server replicas. Retain the checked
contracts you have released, and let generation reject incompatible changes
before it replaces output files.

### Choose version numbers

In the project's `api` settings:

| Setting | Meaning |
| --- | --- |
| `package.version` | SDK version in `major.minor.patch` form, such as `0.2.0`. |
| `apiVersion` | Positive API compatibility major; defaults to `1`. |

SDK version components must be nonnegative integers without leading zeroes.
Prerelease labels and build metadata are not accepted. Versions cannot move
backwards. Raising the API major requires raising the SDK major too.

The generated `contractId` identifies the checked manifest. `wireContractId`
also includes the API major, while preserving legacy v1 identities. Identical
schemas in different API majors therefore have different wire identities.

### Decide whether a change is compatible

Generated `compatibility.json` retains deduplicated checked schemas and immutable
SDK release references. Keep it with generated output.

For source-based generation, the compiler reads canonical history even when using
`--out` and merges destination history before replacing output files. Generation
from an HTTP description uses the history carried by that description. It rejects
a description that omits history already retained by the destination SDK.

| Change | Requirement |
| --- | --- |
| Change a checked schema | Choose a new SDK version. |
| Keep the same API major | The server must still accept released inputs, and older replica results must satisfy the new SDK. |
| Add an optional result field | Can remain compatible when older replicas omit it. |
| Require a new result field | Use a new API major, or keep the field optional during rollout. |
| Raise the API major | Raise the SDK major as well. |

Business semantics still need application review; schema containment cannot prove
them.

### Publish a changed SDK

These checks preserve release-to-contract identity, not an npm registry's release
immutability or every generated runtime byte. Regeneration with a newer Tsrust
compiler can update runtime code without changing DTOs. Select a new package
version before publishing any changed package, including runtime-only changes.
Generation does not publish or silently change package versions for the owner.

### Retain application-owned baselines

The server retains its own baseline files under application-owned paths, referenced
by `api.compatible`. It never needs the generated client's `compatibility.json` directory
to start. Copy a reviewed release baseline into the app when retaining it for a
rollout; the served description includes the configured history.

Known published baselines can be imported with `api.compatible`, a list of
relative `manifest.json` or `compatibility.json` paths. Legacy `api.server.compatible`
remains supported. A fresh selected client can omit methods from foreign server
baselines, but subsequent releases cannot silently remove its own published calls.
Existing owned pre-metadata packages bootstrap from their manifest/package version.

### Roll out across mixed replicas

Both generated clients select a retained profile before I/O. Its input codec must
accept the actual supplied arguments, and its output must be accepted by the
current SDK. Modern dispatchers also recognize retained per-operation fingerprints,
independently of unrelated selected methods.

Strict response codecs and old-client projections remain in force. The clients
never strip supplied arguments or retry requests to find support. Common calls
can survive mixed replicas; newly supplied arguments or new operations can require
newer servers.

#### Handle a compatibility failure

- **TypeScript/JavaScript:** `ApiCompatibilityError` extends `ApiError` and
  includes the requested `apiVersion`.
- **Rust:** `ApiError` includes `api_version`; `is_compatibility()` identifies
  compatibility failures.

Both retain the `api-incompatible` and `api-operation` codes. Do not infer fleet-wide
capabilities from one replica or retry ambiguous writes. Retain older handlers or
migrations for as long as older API majors must work.

