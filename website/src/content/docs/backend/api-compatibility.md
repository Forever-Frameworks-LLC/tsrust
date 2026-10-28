---
title: Version API contracts safely
description: Retain checked contracts for old clients and mixed server replicas.
---

Keep old clients working while deploying new server replicas. Retain the checked
contracts you have released, and let generation reject incompatible changes
before it replaces output files.

## Choose version numbers

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

## Decide whether a change is compatible

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

## Publish a changed SDK

These checks preserve release-to-contract identity, not an npm registry's release
immutability or every generated runtime byte. Regeneration with a newer Tsrust
compiler can update runtime code without changing DTOs. Select a new package
version before publishing any changed package, including runtime-only changes.
Generation does not publish or silently change package versions for the owner.

## Retain application-owned baselines

The server retains its own baseline files under application-owned paths, referenced
by `api.compatible`. It never needs the generated client's `compatibility.json` directory
to start. Copy a reviewed release baseline into the app when retaining it for a
rollout; the served description includes the configured history.

Known published baselines can be imported with `api.compatible`, a list of
relative `manifest.json` or `compatibility.json` paths. Legacy `api.server.compatible`
remains supported. A fresh selected client can omit methods from foreign server
baselines, but subsequent releases cannot silently remove its own published calls.
Existing owned pre-metadata packages bootstrap from their manifest/package version.

## Roll out across mixed replicas

Both generated clients select a retained profile before I/O. Its input codec must
accept the actual supplied arguments, and its output must be accepted by the
current SDK. Modern dispatchers also recognize retained per-operation fingerprints,
independently of unrelated selected methods.

Strict response codecs and old-client projections remain in force. The clients
never strip supplied arguments or retry requests to find support. Common calls
can survive mixed replicas; newly supplied arguments or new operations can require
newer servers.

### Handle a compatibility failure

- **TypeScript/JavaScript:** `ApiCompatibilityError` extends `ApiError` and
  includes the requested `apiVersion`.
- **Rust:** `ApiError` includes `api_version`; `is_compatibility()` identifies
  compatibility failures.

Both retain the `api-incompatible` and `api-operation` codes. Do not infer fleet-wide
capabilities from one replica or retry ambiguous writes. Retain older handlers or
migrations for as long as older API majors must work.
