---
title: Configure API authorization and dispatch
description: Keep access policy on the server and expose only checked operations.
---

Select the operations clients may call and supply an application-owned access
policy. Exporting a function alone does not expose it over HTTP. Start with a
[backend application](../applications/) for a complete working configuration.

## Configure the server

In `tsrust.config.json`, the `api.server` object identifies the context type and
authorization function. This fragment shows the starter's paths; merge it into
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
handlers you need on your Router. The starter mounts all three. They share the
same authorization hook, including requests for the public API description.
Compilation checks these handlers in memory; it needs no generated dispatcher
file or client output directory.

## Make an access decision

The configured authorization function receives the native HTTP context and
returns `Task<boolean>`. It runs before dispatch reads or decodes the request
body or calls an operation:

- Return `true` to allow the request.
- Return `false` to send HTTP 403 with `api-denied`.
- Throwing sends HTTP 500 with `api-internal`, without exposing the exception.

The starter returns `true`, so its greeting API is public. Replace that policy
before exposing protected operations. Application credentials, state and policy
stay in backend modules; the generated SDK does not implement authentication
for you. See the repository's
[application-owned bearer policy](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/examples/backend-tasks/access.tsr)
for a larger example with origin checks and explicit CORS preflight handling.

## Pass private context to an operation

Add `"context": true` to an entry in `api.operations` when its first parameter
is `ReadOnly<Context<...>>` or `Mutable<Context<...>>`, including a named alias.
The compiler supplies that parameter on the server. It is absent from the wire
input and does not change the public contract identity.

The remaining input is zero or one supported plain-data argument. An arbitrary
pool/resource or an owned `Context` cannot be hidden as the context parameter.
Keep resources in application state and pass only supported data to clients.

## Bound requests and responses

`maxRequestBytes` limits the UTF-8 request text's byte length after it has
been read. `maxResponseBytes` bounds serialization of operation results and their
success envelopes. These guards do not replace the Router's admission limits.
Configure its body limit, concurrency and timeouts too; see
[listener limits](../http-transport/#listener-limits).

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

## Retain older contracts

List reviewed baseline files in `api.compatible`; `api.server.compatible` remains
supported for older configurations. Keep each baseline immutable.

Compatibility checks require old inputs to fit the current operation and current
results to project into the old client's strict shape. SDK history also checks
whether old replica results satisfy a new client. Unknown or incompatible shapes
are rejected before invocation. Schema compatibility cannot prove unchanged
business behavior; see [versioning and rollout](../api-compatibility/).

## Export a dispatcher for an existing project

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
