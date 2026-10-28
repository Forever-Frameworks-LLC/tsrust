# Create a backend application

`tsrust new hello --app` creates a native greeting HTTP API. The optional `--target backend` is
accepted; frontend targets are rejected. Bring your own UI and call the API.
Creation never overwrites an existing destination or installs dependencies.

```text
hello/
  tsrust.config.json       Build entry and explicit API operation settings
  app/
    main.tsr               Native HTTP server entry
    api/                   Operations, context and access policy
    lib/                   Reusable backend logic
    tests/                 Native assertions
```

Run `tsrust dev` at the project root. The starter listens on loopback port 4318.
Read app/api/access.tsr before adding sensitive operations: the greeting is
explicitly public. Authentication remains application policy.

The server owns its API in app/. Its explicit api settings select operations,
context, access policy and retained compatibility baselines. app/main.tsr imports
`dispatch`, `describe` and `openapi` from `api:server`. The compiler checks those
handlers and supplies their source in memory; it writes no dispatcher directory.
Client generation is never required to check, build or run the service.

Run `tsrust check`, `tsrust test` and `tsrust format --check` for backend source.
After changing a contract, review its description and retain prior checked contract
baselines under app/ as needed for rolling deployments. List those paths in
api.compatible, bump package.version for releases, and change apiVersion for an
intentionally breaking API major. Ordinary exports remain private unless selected.

Generate an independent TypeScript or Rust SDK from the running service, or from a
saved description. Choose its location explicitly: js-client/, rust-client/, an
external project or a directory for later publication. No SDK output is created by
the backend scaffold, and generating an SDK never modifies the server.

The TypeScript package exposes plain named calls and a configurable HTTP transport,
without React, TanStack or runtime package dependencies. Its package.json is ESM
metadata; backend development does not require npm or Node installation.
The starter explicitly mounts GET /api/description and GET /api/openapi.json,
using the same application access policy as dispatch. Existing applications opt
in with api settings and mount the api:server describe/openapi handlers themselves. The OpenAPI
document describes the actual single POST endpoint; configure its server URL and
application authentication policy for deployment.

SDK generation also emits http-api.json and openapi.json. A separate directory
can regenerate the TypeScript SDK without backend source:

```text
tsrust api generate --description http-api.json --out sdk
```

Descriptions are limited to 1 MiB and validated before output changes. Existing
generated output retains ownership guards and compatibility history protection.
Fetch a description directly, or generate an asynchronous Rust client:

```text
tsrust api generate --url http://127.0.0.1:4318/api/description --out js-client
tsrust api generate --url http://127.0.0.1:4318/api/description --language rust --out rust-client
```

Remote URLs require HTTPS; HTTP is allowed on loopback. Redirects are rejected and
descriptions are limited to 1 MiB. For a protected description, add
`--header-env Authorization=API_AUTHORIZATION`; the environment variable contains
the complete header value. Credentials are resolved for this explicit request and
are not embedded in the generated SDK. Existing outputs retain ownership guards.

Rust named calls are awaitable, for example `client.api().greet(name).await`.
The default optional HTTP transport uses async reqwest on the caller's Tokio
runtime. Custom transports return futures; header providers may await credential
refresh. There is no blocking HTTP adapter. The Rust generator supports tuples
up to 16 elements and rejects larger tuples. Client generation installs nothing;
Rust dependency resolution remains an explicit Cargo operation in the consumer.

OpenAPI describes the starter's POST /api endpoint. When mounting the dispatcher
elsewhere, adapt the deployment specification's path/server URL and client URL.
This is checked Tsrust RPC export, not an arbitrary REST/OpenAPI importer.

`tsrust new hello` retains the console scaffold. Original Tsrust UI source and
explicit legacy commands remain historical compatibility pending dependency
isolation; they are outside the active backend product. See the backend-only reset
for cleanup, distribution and self-update acceptance still pending.
