# Native deployment

[Documentation index](../../README.md) · [Proposed roadmap](../../roadmap.md) · [Source section map](../../plans/section-map.md)

## Build and run a backend

From a configured backend project:

```sh
tsrust check
tsrust test
tsrust build --release -o target/server
# Windows output: target/server.exe
```

Build on the destination OS/architecture with the pinned native tools and prepared
sources described in [machine setup](../../../website/src/content/docs/tools/rust-setup.md). These commands do not
provide a cross-compilation switch. Copy the produced executable and any
application-owned configuration or assets to the deployment environment. Native
execution needs no Tsrust compiler, Cargo, Node or JavaScript runtime.

A native executable is not necessarily statically linked. Its OS libraries,
external native dependencies, TLS trust material and application files still need
to be present. Validate the actual artifact in the destination image or machine;
`doctor --native` checks the build host, not deployment compatibility. Keep secrets
in deployment configuration rather than source or generated client packages.

The [backend starter](../../../website/src/content/docs/backend/applications.md) listens on loopback by default.
Choose production binding, authentication, TLS termination and process supervision
for your deployment. API description and OpenAPI endpoints use application policy;
review that policy before exposing them publicly. Generate independent clients
from the checked contract and retain compatibility baselines for rolling updates.

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="core-33"></a>

## 33. Startup and Deployment

Target:
lang build --release
producing one deployable native executable where practical.
Investigate:
* static binaries
* dynamic linking
* CA certificates
* timezone database
* native dependencies
* database drivers
* TLS implementations
Goal:
COPY app /app
ENTRYPOINT ["/app"]
with no language runtime installation required.
