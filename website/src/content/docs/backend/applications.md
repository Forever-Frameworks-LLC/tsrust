---
title: Create a backend application
description: Create a native HTTP service with server-owned operations and explicit access policy.
---

Create a service with a greeting operation, then generate clients independently
for the applications that call it. You need the CLI and
[prepared native build tools](../../tools/rust-setup/).

## Create and run the service

```sh
tsrust new hello --app
cd hello
tsrust dev
```

The starter listens on `127.0.0.1:4318`. Creation refuses an existing destination
and does not install dependencies. `--target backend` is optional; frontend
targets are rejected.

| File | Purpose |
| --- | --- |
| `app/main.tsr` | HTTP routes, server limits and listening address. |
| `app/api/operations.tsr` | Exported operation implementations. |
| `app/api/context.tsr` | Application request context. |
| `app/api/access.tsr` | Authorization policy. |
| `app/lib/greeting.tsr` | Reusable greeting logic. |
| `app/tests/greet.test.tsr` | Native assertions. |
| `tsrust.config.json` | Build entry and selected API operations. |

Use `tsrust new hello` without `--app` for a console project.

## Review access before deployment

The starter authorization function returns `true`: its greeting API is public.
Replace `app/api/access.tsr` before adding protected operations. Authentication
and deployment policy remain your responsibility.

The `api` configuration explicitly selects operations, context and authorization.
Ordinary function exports do not automatically become API operations. The server
imports `dispatch`, `describe` and `openapi` from `api:server`; the compiler checks
and supplies those handlers in memory. Checking and running the service need no
generated client or checked-in dispatcher.

The starter mounts these routes:

| Route | Purpose |
| --- | --- |
| `POST /api` | Dispatch a selected operation. |
| `GET /api/description` | Return the checked description used by SDK generation. |
| `GET /api/openapi.json` | Return the OpenAPI document. |

Description and OpenAPI handlers use the same application access policy as
dispatch. See [authorization](../authorization/) for protected services.

## Check changes

From the project root:

```sh
tsrust check
tsrust test
tsrust format --check
```

For rolling deployments, retain prior checked contracts and list them in
`api.compatible`. Update `api.package.version` for SDK releases; change
`api.apiVersion` for an intentionally breaking API major. Review
[compatibility rules](../api-compatibility/) before changing a public contract.

For release builds and destination checks, see [deployment](../deployment/).

## Generate a client

Keep the service running and use a second terminal:

```sh
tsrust api generate --url http://127.0.0.1:4318/api/description --out js-client
tsrust api generate --url http://127.0.0.1:4318/api/description --language rust --out rust-client
```

Both commands require an explicit output directory. Generation writes client
files without changing the server or installing packages. Backend development
does not require Node or npm. The TypeScript client uses named asynchronous calls;
the Rust client uses awaitable calls and an optional async HTTP transport.

A generated `http-api.json` can be used later without the running service or its
source:

```sh
tsrust api generate --description js-client/http-api.json --out another-client
```

Remote description URLs require HTTPS; loopback HTTP is allowed. Redirects are
rejected and descriptions are limited to 1 MiB. For protected descriptions and
consumer setup, see [independent API clients](../api-clients/).

## Understand OpenAPI scope

OpenAPI describes the service's single `POST /api` RPC endpoint. If you mount the
dispatcher elsewhere, update the deployment specification's path/server URL and
client endpoint. This export does not import arbitrary REST/OpenAPI services.
