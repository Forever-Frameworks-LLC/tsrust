# Checked named API clients

Status: first compiler/client foundation passes focused acceptance. This does not yet
provide the finished HTTP/SQLite service, Electron adapter, TanStack wrapper or
frontend templates from the [prototype goal](../agent-goal.md).

Declare the operations to expose in a separate `tsrust.api.json`. Keeping this
configuration separate allows backend/client-only adoption without changing an
existing frontend's package manager, lockfile, scripts or directory layout.

```json
{
  "schemaVersion": 1,
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

The test transport invokes no remote code; the next section supplies a normal
HTTP transport/dispatcher. A transport implements
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
explicit supported data before exporting.

The contract hash describes exact wire shape. It is not a compatibility policy:
do not require exact hash equality for additive rolling upgrades. Hosted transport
limits, application authorization, public error projection, compatibility checks
and query adapters remain separate prototype acceptance requirements.

Run [the example](../../examples/api-contract/README.md) and
`node tools/verify-api-client.mjs` with the current compiler to check named calls,
codecs, type inference and owner-edit protection. The verification uses existing
repository Node/TypeScript development tools; it installs no template packages.
