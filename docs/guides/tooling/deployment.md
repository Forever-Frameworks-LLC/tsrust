# Deployment research and original vision

Current user instructions: [Deploy a native backend](../../../website/src/content/docs/backend/deployment.md).

The material below preserves the original deployment proposals; it is not a claim
that static linking or every destination environment is supported.

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
