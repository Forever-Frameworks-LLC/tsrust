# Checked named API clients

Status: plain clients, native HTTP/SQLite dispatch, the optional TanStack adapter
and the React task app pass their focused milestones. Versioned SDK rollout
support below passes its retained-baseline replica matrix. Electron/mobile and
the complete [prototype goal](../agent-goal.md) remain incomplete.

Declare the operations to expose in a separate `tsrust.api.json`. Keeping this
configuration separate allows backend/client-only adoption without changing an
existing frontend's package manager, lockfile, scripts or directory layout.

```json
{
  "schemaVersion": 1,
  "apiVersion": 1,
  "entry": "app/main.tsr",
  "output": "packages/api",
  "package": { "name": "@example/api", "version": "0.1.0" },
  "operations": [
    { "name": "tasks.list", "module": "app/tasks.tsr", "export": "list", "kind": "query" }
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
tsrust api generate
tsrust api generate --check
tsrust api generate --config ./service/tsrust.api.json --out ./client-package
```

Without `--config`, the CLI finds the nearest ancestor configuration. Configured
entry/module/output paths are relative to that file and exclude `.`/`..` and
backslashes; use portable forward slashes. An explicit output flag is relative to
the current directory and can select a standalone destination. Initial package
versions use ordinary `major.minor.patch`; prerelease versions are a later addition.

The output contains JavaScript ESM, TypeScript declarations, a manifest, package
metadata and the generated runtime's MIT notice. It has no install scripts or
framework dependencies. No backend bodies, source paths, credentials, database
dependencies or source-tree imports are copied into it. The package is private
by default. Review the generated files, package metadata and your own licensing
before enabling publication; generation/check/build never publishes anything.

Existing nonempty application directories are not adopted as generated output.
Only previously generated files whose recorded hashes still match, or already
match the intended replacement, are replaced. Edited generated files produce a
diagnostic and remain untouched. Unrelated files remain user-owned. `--check`
does not write files and fails for stale outputs or markers. A lock excludes
concurrent generators, and files are individually replaced atomically. A failed
partial replacement can be retried using the old/planned hash checks.

## Calls and wire values

The initial package exposes `createClient(transport)`, `ApiError`, a frozen
manifest and an exact contract hash. Named calls follow the selected namespaces:

```ts
const api = createClient(transport);
const tasks = await api.tasks.list({ search: null });
```

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

Supported wire values are strings, booleans, finite numbers, JavaScript-safe
integers, null, checked literals, dense arrays, tuples, plain data records and
unions. Optional fields omit missing/undefined properties; required fields must
exist and null remains distinct. Native integers outside plus/minus
9,007,199,254,740,991 fail validation instead of rounding. Void results use null
on the wire and become undefined to the caller. Object fields are data properties;
accessors and unknown keys reject. Special JSON keys remain data without changing
object prototypes. Recursive/deeper-than-128-level schemas, functions, native
resources and implicit collection/domain serialization reject; project them into
explicit supported data before exporting. Strings must contain well-formed Unicode:
lone UTF-16 surrogates reject before transport because native JSON strings use UTF-8.
JSON encodes negative zero as zero; it is not a distinct wire value.

## HTTP transport

The separate `@example/api/http` export uses the host Fetch API. The plain package
entry still requires no DOM types or Fetch globals. HTTP consumers need their
browser/React Native Fetch type declarations. Pass an absolute endpoint URL:

```ts
import { createClient } from '@example/api';
import { createHttpTransport } from '@example/api/http';

const api = createClient(createHttpTransport({
  url: 'http://127.0.0.1:4318/api',
  headers: async () => ({ authorization: `Bearer ${await currentToken()}` }),
}));
const count = await api.math.count(1, { signal: controller.signal });
```

Each call posts `{operation,input}` as JSON to the configured endpoint and sends
`x-tsrust-contract` with a selected checked wire-profile identity. JSON Content-Type
parameters carry the requested API major, current operation fingerprint and SDK
artifact fingerprint. This adds no new CORS header names. The server retains
application authorization responsibility. Headers are resolved
for each call so credential changes take effect; the adapter sets its protocol
headers after application headers. Cookies are omitted and redirects reject.
Use HTTPS for remote credentials. Loopback HTTP is available for the desktop bridge.
URLs containing embedded credentials or fragments reject during construction.

Non-2xx responses can carry the same structured public failure envelope. A success
envelope on a failing HTTP status rejects. HTML, invalid JSON and invalid output
data reject with protocol/HTTP/validation errors. Host network exception details
are replaced with an `api-network` error; cancellation uses `api-cancelled`.
There are no automatic retries, including mutations. Cancellation requests abort
through the host Fetch API; it does not prove rollback or backend cleanup. Supply `fetch` explicitly for a
host-specific adapter or testing. Response text is read before parsing; this client
does not impose a streaming memory limit. Backend request/response bounds and
mobile Fetch behavior remain service/platform acceptance requirements.

Fetch status checking, JSON content types, credentials and cancellation follow
[the Fetch API documentation](https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API/Using_Fetch).

## Versioned SDKs and mixed server replicas

`package.version` identifies an SDK release and retains an immutable wire-contract
reference. `apiVersion` is a positive
API compatibility major, defaulting to 1. `contractId` identifies the exact checked
manifest. `wireContractId` includes the API major, while preserving legacy v1
identities. Consequently an identical DTO schema in API v2 cannot accidentally
pass a pre-versioning API v1 server's hash admission check.

Generated compatibility.json retains deduplicated checked schemas and immutable
SDK release references. Keep it with generated output. Generation reads canonical
history even when exporting with --out, merges destination history, and checks
compatibility before replacing public/private outputs. Changed schemas require a
new SDK version. Within one API major, released inputs remain accepted and old
replica results must satisfy the new SDK. Optional result additions can work;
required new fields must use an explicit API major or remain optional during
rollout. A new API major requires a new SDK major. Business semantics still need
application review; schema containment cannot prove them.

These checks preserve release-to-contract identity, not an npm registry's release
immutability or every generated runtime byte. Regeneration with a newer Tsrust
compiler can update runtime code without changing DTOs. Select a new package
version before publishing any changed package, including runtime-only changes.
Generation does not publish or silently change package versions for the owner.

Known published baselines can be imported with top-level `compatible`, a list of
relative manifest.json or compatibility.json paths. Legacy server.compatible
remains supported. A fresh selected client can omit methods from foreign server
baselines, but subsequent releases cannot silently remove its own published calls.
Existing owned pre-metadata packages bootstrap from their manifest/package version.

Before I/O, the HTTP adapter selects a retained profile whose input codec accepts
the actual supplied arguments and whose output is accepted by the current SDK.
Modern dispatchers recognize retained per-operation fingerprints independently
of unrelated selected methods. Strict response codecs and old-client projections
remain in force. The adapter never strips arguments or retries requests to find
support. Common calls can survive mixed replicas; newly supplied arguments or new
operations can require newer servers. ApiCompatibilityError extends ApiError,
preserves api-incompatible/api-operation codes and reports the requested API major.
Do not infer fleet-wide capabilities from one replica or retry ambiguous writes.
Retain older handlers or migrations for as long as older API majors must work.

The real old/new native server and browser matrix is
`node tools/verify-api-replicas.mjs`. It passes the scoped
[SDK milestone](../milestones/sdk-versioning.md); see
[current evidence](../agent-progress.md) for remaining prototype work.

## Private native dispatch

Optional private source output connects checked operations to the native Router.

```json
{
  "server": {
    "output": "app/generated-api",
    "context": { "module": "app/context.tsr", "export": "TaskContext" },
    "authorize": { "module": "app/access.tsr", "export": "authorize" },
    "compatible": ["contracts/v1.json"],
    "maxRequestBytes": 65536,
    "maxResponseBytes": 65536
  }
}
```

Add this optional member to the ordinary API config, and add `"context": true`
only to operations whose first parameter is a borrowed native HTTP Context.
That parameter is private; the remaining zero/one plain-data argument is the wire
input. An arbitrary pool/resource or an owned Context cannot be hidden this way.
Context roles do not alter the public contract identity. Application state and
policy stay in backend modules. An authorize hook must return `Task<boolean>`
from the configured Context; it runs before request decoding or operation calls.
Returning true is the application's explicit access decision. Configure Router
body/concurrency/time limits too: the generated decoder's post-read size guard
does not replace HTTP admission limits.

The private output contains `index.tsr`; register its `dispatch` export on the
application Router. It requires POST, JSON MIME, a declared operation and an
accepted contract hash. Schemas derive from the checked functions. Generic
materialization uses ordinary checked specialization, never an assertion of
unknown JSON as a DTO. Invalid arguments/results, policy denial and backend
exceptions produce bounded public error messages. Typed domain-result unions
remain ordinary successful data. Neither failure nor cancellation asserts rollback.

The private output must be separate from the public package. Generation checks
the complete virtual handler before replacing outputs, pins checked input/config
versions and preflights ownership for both destinations. Both destinations have
independent generated hash markers/locks; edits are preserved. Replacements are
individually atomic and retryable, not a filesystem-wide transaction. `--check`
writes nothing. An explicit `--out` changes the client destination while a configured
private server keeps its own destination. Omit `server` for client-only adoption.

Retained baselines check that old inputs fit the new operation and new outputs
can be projected into the old strict shape, recursively. Versioned SDK history
also checks the reverse result direction for old replicas. Unknown operation
shapes reject with api-incompatible before invocation, even when their names
exist. Keep immutable contract versions rather than changing an old policy input
to describe a different client. Neither hash matching nor containment establishes
unchanged application semantics.

The HTTP encoder retains platform JSON escaping and marks unsafe integral JS
numbers as decimal floating-point tokens. Thus finite values such as 1e20 do not
overflow native bare-integer JSON storage. Numeric text in strings/keys stays
unchanged. Safe integer DTOs retain their stricter limit.

Run [the example](../../examples/api-contract/README.md) and
`node tools/verify-api-client.mjs` with the current compiler to check named calls,
codecs, type inference and owner-edit protection. The verification uses existing
repository Node/TypeScript development tools; it installs no template packages.

## Optional framework-independent query adapter

The `/query` export supplies createQueryClient without importing React. Install
the optional @tanstack/query-core peer, supported from 5.105.0 through major 5.
The plain and HTTP entries still import no cache or framework dependency.
Use the application's upstream cache, with generated named options:

```ts
import { QueryClient } from '@tanstack/query-core';
import { createQueryClient } from '@example/api/query';
const cache = new QueryClient();
const api = createQueryClient(transport, { scope: JSON.stringify([backendURL, userId]) });
const tasks = await cache.query(api.tasks.list.queryOptions(input, { staleTime: 30000 }));
```

Query operations expose key and queryOptions; mutations expose mutationOptions.
They work with QueryClient, QueryObserver and MutationObserver without a UI
framework. Direct named calls remain ordinary uncached promises. Upstream owns
cache policy, request deduplication, observer state and resource lifecycle.
Query Core 5 supports stale-while-revalidate through ensureQueryData with generated
options and revalidateIfStale:true; that method is deprecated for the next major,
so a future major integration migration must account for it. Observers can also
return existing data while refetching in the background.

Keys include caller-supplied service/principal scope, versioned contract identity,
operation and validated normalized input. Secrets, cancellation signals and cache
policy do not enter keys. Scope changes must accompany principal/service changes.
Queries/mutations default to no retries. Acknowledged mutations invalidate their
scope, while refresh errors remain query errors. Special JSON data fields retain
the same serialization and structural-sharing protection as the React adapter.
Host AbortSignal/timer typings are required by upstream Query Core: use browser
library types or Node declarations. Plain clients still require no DOM types.

The pinned implementation follows upstream's
[QueryClient API](https://tanstack.com/query/latest/docs/framework/react/reference/classes/QueryClient)
and the installed 5.105.0 package sources. It introduces no parallel cache engine.
Focused acceptance passes; see the [query-core milestone](../milestones/query-core-adapter.md).

## Optional named TanStack adapter

The generator supplies a separate `/react-query` entry. Actual generated
runtime/declarations and native task integration pass the upstream witnesses
described in [adapter evidence](../milestones/generated-query-adapter.md).
The plain and HTTP entries share the same wire checks without importing React.
React/TanStack peers are optional in package metadata and must be resolvable in
applications that use this entry.

```ts
const api = createReactClient(transport, { scope: JSON.stringify([backendURL, userId]) });
const tasks = api.tasks.list.useQuery({ after: 0, limit: 100 });
const createTask = api.tasks.create.useMutation();
await createTask.mutateAsync({ title: 'Ship prototype' });
```

The task backend returns tagged domain results; inspect `tasks.data.kind` and
mutation result kinds separately from protocol/network errors. `queryOptions`
and `mutationOptions` also work with ordinary TanStack APIs. Select results,
mutation variables/context and cache data retain generated types. Use upstream
skipToken when query input is unavailable. Generated hooks return conservative
possibly-undefined data; callers can pass options to upstream useQuery for its
initialData overloads.

The application supplies its QueryClient/Provider and a stable transport/client.
Scope identifies backend and principal without containing credentials. Contract
identity and normalized copied input are added to keys automatically. Mutations
invalidate only that client scope after an acknowledged response; refresh failure
does not change write acknowledgement. Default retries are false, with ordinary
explicit retry options available. Direct promise calls preserve plain-client
behavior and do not invalidate/cache by themselves. Query functions/keys/hash
functions and mutation functions/keys remain generated ownership boundaries.
Result schemas containing `__proto__` default to structuralSharing:false so
refetches preserve those fields in stored data. Explicit true rejects for those
schemas; other schemas retain ordinary upstream sharing, and custom sharing
functions remain an explicit application choice.

## Ordinary local package adoption

Generate into an explicit folder, then use the application's package manager:

```sh
tsrust api generate --config tsrust.api.json --out ./dist/client
cd dist/client
npm pack --ignore-scripts
cd /path/to/existing-app
npm install /path/to/client/example-query-api-0.1.0.tgz --ignore-scripts
```

The tarball filename follows the configured npm package name/version. Inspect
`npm pack --json --ignore-scripts` before sharing. Its allowlist includes runtime,
declarations, contract, README and license; generated ownership markers and
unrelated files are excluded. Generated packages are private by default; local
packing works without publication. Consumers need no compiler, backend source or
generated-directory symlink. Plain users need no React/TanStack installation.
Applications using `/react-query` install its peers normally in their own project.

For an npm workspace, include the generated package directory in the root's
`workspaces`, declare its configured name/version in the consuming app's normal
dependencies and run `npm install`. Hoisted application peers resolve from the
linked generated package. A raw sibling symlink outside a workspace may resolve
peers from the source directory instead; use a tarball or npm's `--install-links`
for an installed copy. Other package managers need their own workspace peer
acceptance; do not copy npm-specific settings into their projects automatically.

`node tools/verify-api-package.mjs` checks isolated external plain/React consumers,
TypeScript errors/inference, npm workspace resolution and an owner-installed
dependency without rewriting existing scripts/source/configuration. It uses only
cached pinned upstream packages and never publishes. This proves ordinary npm
adoption; dedicated Tsrust export/pack/setup convenience commands remain pending.
