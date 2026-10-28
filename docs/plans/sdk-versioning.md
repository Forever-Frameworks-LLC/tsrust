# Versioned SDK compatibility implementation

Status: focused acceptance passes from `03f20f0`, following the owner's
mixed-replica concern. See [the milestone evidence](../milestones/sdk-versioning.md).
The design checklist below is implemented; guarantees apply to retained checked
baselines, not every unknown server or application semantic change.

## Guarantees and boundaries

Keep three identities separate: SDK package version, explicit API compatibility
major version and exact generated artifact fingerprint. Preserve the current wire
schema IR instead of adding a second DTO language. A checked upgrade within one
API major must preserve old callers and let the new SDK decode older replicas'
results. Required new result fields therefore need an explicit API major or must
be optional during a mixed rollout. Input additions can require a newer operation
capability when supplied; arguments must never be silently discarded.

Compatibility checks cover known retained wire baselines, not arbitrary business
semantics or every external deployment. Fresh client-only projects need a known
baseline to claim upgrade compatibility. New features cannot run on replicas that
do not implement them. Keep typed failures and deployment/routing policy explicit.
Never retry an ambiguous write while discovering a version.

## Implemented policy foundation

`api/versioning.rs` defines serde-validated ApiHistory/ApiRelease records with API
major, SDK version and checked manifest. advance returns an immutable candidate,
checks SDK monotonicity, prevents same-version contract replacement, requires an
SDK major for an API-major change, and checks both rollout directions against all
same-major baselines. Unchanged wire shapes keep one baseline with their latest
SDK version rather than accumulating identical schema copies.

`Operation.accepts_previous_result` reuses the existing strict schema-containment
algorithm. It intentionally differs from supports_previous: the latter permits
projecting new output fields away for old clients, while the former catches new
required fields absent on an old replica. Regression witnesses cover that exact
distinction, compatible optional additions, unrelated operations, removals, SDK
version misuse and serialization. These witnesses pass in the ten-test api_contract batch.

## Implemented integration checklist

1. Add explicit API version to generator configuration independently of package
   version. Read/validate generated compatibility metadata from the canonical
   configured output before replacements, including export to a different folder.
   Bootstrap pre-metadata output from its checked manifest/package version. Preserve
   the existing pinned-input, marker, symlink and owner-edit checks. Stale checks
   must remain read-only and repeated generation deterministic.
2. Carry retained semantic baselines as generated metadata, without server code,
   credentials, command logs or source snapshots. Normal setup/export should not
   make users hand-maintain a list of every SDK artifact hash. Explicit published
   baselines and existing server.compatible manifests remain migration inputs.
3. Generate safe per-operation wire-profile selection. For a common operation,
   choose an older known profile only if its input validator accepts the actual
   supplied arguments and its output is statically accepted by the current SDK.
   Do this before sending one request; do not implement speculative network retries
   or silently strip new arguments. Preserve strict response validation.
4. Add per-operation schema identity and explicit API-version metadata alongside
   artifact diagnostics. A modern dispatcher can recognize the caller's exact
   operation shape independently of the total selected client manifest. Known
   current shapes receive current results; retained shapes get their checked
   projections. A selected older profile remains usable on pre-change servers,
   whose existing contract header handling must still be exercised.
5. API majors must affect wire/cache identity even when the raw schema is identical;
   otherwise an old server could accidentally accept new-major semantics merely
   because a shape hash matches. Preserve legacy v1 identities for migration, but
   ensure new majors cannot accidentally pass pre-version server admission.
6. Keep auth and server input/output checks authoritative. Support/version errors
   must occur before invocation. Carry metadata in JSON Content-Type parameters,
   which existing dispatchers already accept; do not add a CORS header requirement
   during rollout. Preserve old SDK strict projection and private failure sanitization.

## Required acceptance matrix

Use real native old/new dispatchers and an alternating-replica endpoint. Include
the retained pre-change native executable or equivalent immutable legacy source
evidence separately from two contracts implemented by a new protocol.

- Old SDK calls retain their original output shape on the new server.
- A new client subset or unrelated added operation does not break unchanged calls
  across either replica.
- Optional new arguments omitted can use an older safe profile; supplying a new
  argument cannot be silently downgraded or execute an unsupported write.
- Optional response additions work without requiring their presence on an older
  replica; required additions fail the same-major generation gate.
- Explicit API-major changes remain distinguishable even with identical schemas.
- Unknown operations/capabilities return bounded typed failures, with no repeated
  writes or fleet-wide capability inference from one replica.
- Generation rejects accidental breaks before either output changes; owner edits,
  deterministic stale checking, detached packages and plain/framework adapters
  remain intact.

Run the completed focused generation/runtime/TypeScript/native matrix once, repair
affected failures only, format the whole project and checkpoint locally. Full
family timing/regression remains a later milestone, not a trigger for each policy
edit. Preserve the recorded scope and limits when describing rollout acceptance.

## Implemented protocol and executable acceptance

Generated compatibility.json retains checked wire baselines and immutable SDK
version references. Canonical and export destinations are read through pinned
source identities and ownership markers before either public/private output is
replaced. Pre-metadata output bootstraps from its owned manifest and package.
An explicit compatible list can import known manifests or generated histories.
Client-only subsets can omit foreign baseline methods without claiming those
methods as their own released SDK surface.

The dependency-free HTTP adapter chooses a known input-compatible profile before
I/O. Content-Type carries tsrust-api, tsrust-operation and tsrust-client parameters;
x-tsrust-contract carries the selected legacy-safe wire profile. API v1 preserves
legacy identities; other API majors affect both wire identity and cache identity.
The native dispatcher admits known per-operation shapes independently of the
selected whole-client fingerprint and preserves checked legacy projections.
ApiCompatibilityError preserves api-operation/api-incompatible error codes and
reports the requested API major. No network retries or argument removal occur.

tools/verify-api-replicas.mjs is the native/browser rollout witness. It retains
the accepted pre-change task-http-witness executable, reconstructs the old SDK
runtime from immutable checkpoint 03f20f0 in a disposable consumer, builds the
new dispatcher separately, and shares a disposable database between replicas.
An alternating proxy counts calls; real browser preflights go to the old replica.
The witness checks a selected client subset, unrelated new method, common writes,
typed unsupported-operation failures, identical-schema major isolation and drain.
It restores the canonical generated dispatcher and removes temporary consumers.
This witness passes, including optional new request/result fields and strict
old-SDK projection. The milestone records exact commands, executable identities,
retained failures/repair and remaining prototype scope.
