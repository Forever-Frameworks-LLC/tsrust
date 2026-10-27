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
Fetching descriptions directly by URL and Rust output remain pending; this file
does not claim those options exist.

`tsrust new hello` retains the console scaffold. Original Tsrust UI source and
explicit legacy commands remain historical compatibility pending dependency
isolation; they are outside the active backend product. See the backend-only reset
for cleanup, distribution and self-update acceptance still pending.
