# Create a backend application

`tsrust new hello --app` creates a native greeting HTTP API and a checked,
framework-independent TypeScript client. The optional `--target backend` is
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
  generated/
    client/                SDK, codecs and compatibility history
    server/                Native dispatcher
    .tsrust-integration.json
```

Run `tsrust dev` at the project root. The starter listens on loopback port 4318.
Read app/api/access.tsr before adding sensitive operations: the greeting is
explicitly public. Authentication remains application policy.

After changing a contract, run `tsrust api generate` and inspect the generated
SDK, dispatcher and compatibility history. Generated ownership guards preserve
custom edits; ordinary app files remain yours. API regeneration is explicit.
Run `tsrust check`, `tsrust test` and `tsrust format --check` for backend source.
Keep compiler and editor diagnostics visible.

The TypeScript package exposes plain named calls and a configurable HTTP transport,
without React, TanStack or runtime package dependencies. Its package.json is ESM
metadata; backend development does not require npm or Node installation.
The starter explicitly mounts GET /api/description and GET /api/openapi.json,
using the same application access policy as dispatch. Existing applications opt
in by mounting the generated describe/openapi handlers themselves. The OpenAPI
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
tsrust api generate --url http://127.0.0.1:4318/api/description --out generated/external-client
tsrust api generate --url http://127.0.0.1:4318/api/description --language rust --out generated/rust-client
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
