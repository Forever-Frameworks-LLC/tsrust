# Generated TanStack Query adapter

Status: focused acceptance passes. Nine rebuilt contract/metadata cases, actual
generated runtime/declarations, plain-client regression and native-service
integration pass. Browser/mobile UI and full prototype acceptance remain pending.
Baseline is accepted task service `9c9d6454`.

The package's separate `/react-query` entry builds callable named operations with
query `key/queryOptions/useQuery` and mutation `mutationOptions/useMutation`
members. Declaration types reference the original Client function types and
upstream TanStack types, preserving the compiler-derived data contract rather
than introducing handwritten DTOs. The plain client, HTTP transport and adapter
share the existing wire validator in a private relative `wire.js` module. Plain
declarations/runtime do not load React or TanStack. Optional peers are React
18/19 and TanStack >=5.105.0 <6; the tested fixture pins React 19.3.1, TanStack
5.105.0 and TypeScript 7.0.2.

## Cache and lifecycle behavior

Scope is a required nonempty backend/principal identity string, not a credential.
Keys add contract identity, operation identity and serialized validated input.
Schema-ordered copies normalize optional undefined fields to absence, preserve
null, capture input before caller mutation, and preserve `__proto__` data through
TanStack's hash. Keys are immutable. Query cancellation forwards TanStack's
AbortSignal through the original client; it does not assert backend rollback or
cleanup completion. Upstream skipToken safely disables unavailable inputs.

Result schemas with a literal `__proto__` field also need a targeted structural
sharing default. The upstream ordinary-object accumulator drops that field from
stored data on refetch, even though fetchQuery returns the intact immediate
response. A new witness inspects the stored data after a changed response and
failed before the repair. Those result schemas now default structuralSharing to
false; explicit true rejects, while a caller-supplied sharing function remains
available. All other schemas keep normal upstream sharing. The repaired staged
witness passes both immediate and stored data checks.

Mutation responses trigger invalidation only for this client scope. Broad
invalidation is deliberate and documented; no parallel cache or dependency
registry is added. Refresh rejection does not convert acknowledged writes into
failures. Tagged domain failures remain data. Query/mutation retry defaults are
false even if global upstream defaults enable retries; explicit per-call options
remain available. Generated keys/functions cannot be overwritten. Direct named
promise calls remain independent of cache behavior.

## Implemented witnesses and admission

`tools/verify-api-query.mjs` uses real QueryClient/QueryObserver/mutation cache,
actual React hooks through React DOM server rendering, and upstream TypeScript
positive/negative checks. It verifies input copies, optional normalization,
principal isolation, special-key cache identity, mutation invalidation/context,
acknowledgement after refresh failure, no hidden retries, abort, skipToken and
inferred selected/cache/mutation types. Browser/mobile UI behavior remains a
later acceptance requirement.
`tools/verify-task-query.mjs` additionally generates a peer-resolvable task client
in the development fixture and runs the existing native task verifier with real
TanStack query/mutation options. Its additional create/update/delete assertions
check refetched native SQLite state through an active QueryObserver. The ordinary
task verifier remains framework-independent unless this mode is explicitly set.
Use `--run-only` to reuse the already accepted native test executable; omit it to
rebuild that executable. Actual run-only integration passes native CRUD,
validation, access/CORS, persistence, orderly drain and active query refreshes.

A disposable renderer compiles the actual `api/react_query.rs` module with a
minimal manifest carrier and uses the existing checked public generator for
fixture input. Staging the actual new JS assets and resulting declarations passes
the upstream witness. This checks source behavior quickly before a costly full
compiler link; it is not proof that the CLI packages the new files correctly.
`target/render-query-declarations.rs/.exe` and the test fixture's
`.tmp-query-staged` directory are disposable and must be removed after actual
generation acceptance replaces them. They were removed after actual acceptance.

The expanded private development fixture adds React DOM 19.3.1, @types/react-dom
19.3.0 and scheduler 0.28.0 (MIT), with install scripts disabled. npm reports zero
known advisories across eleven entries, ten verified signatures and five verified
attestations. Package/lock files remain ordinary editable development files.

## Executed acceptance

The completed generator passes the focused api_contract target (nine tests).
Actual `node tools/verify-api-query.mjs` passes without a staged argument;
`node tools/verify-task-query.mjs --run-only` passes against the accepted native
task executable. Regenerating both native fixture client configurations followed
by `node tools/verify-native-api.mjs --run-only` passes wire validation,
authorization, private Context and rolling-client output projection.
Logs are `target/api-query-acceptance.log`, `target/task-query-acceptance.log`
and `target/api-query-native-regression.log`.
Keep one Cargo target/job, incremental disabled. Preserve passed evidence and
repair only failed checks. Run whole-project formatting before the checkpoint.
Do not claim installed refresh, full-family timing/regression, frontend targets,
package publication or the entire prototype complete from this adapter section.

The first focused rebuild `60088` completed with nine contract/metadata cases
passing (`target/api-query-generator-focused.log`). The subsequent cache refetch
witness exposed the structural-sharing loss above. After its repair and the
queryOptions/hook argument distinction, staged runtime/SSR/types pass again.
The repair rebuild `40498` completed, exit 0, nine cases pass in
`target/api-query-sharing-repair.log`:

```sh
cargo test --offline --locked -p tsrust-compiler -p tsrust-lsp \
  --test api_contract -- --test-threads=1
```

Actual query generation and native run-only checks above replace staged evidence.
Whole-project formatting passed twice with zero changes/fingerprint updates.
The unchanged plain/HTTP SDK and owner-edit verifier passes using the first
rebuilt CLI (`target/api-query-plain-regression.log`); no need to repeat it unless
those inputs change.
