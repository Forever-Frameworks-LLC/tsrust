# Framework-independent generated query adapter

Focused acceptance passes at `67cb812c`, from `47bb2375`, 2026-10-27. The full application
prototype remains incomplete; see docs/agent-goal.md.

## Behavior and boundary

Generated packages add an optional `/query` entry with createQueryClient, named
queryOptions/key methods and mutationOptions. The shared runtime owns checked
input normalization, scoped/versioned keys, conservative retry defaults and
acknowledged-mutation invalidation. Query Core owns caching, deduplication,
observers, stale policy, cancellation and lifecycle. Direct named calls remain
uncached promises. There is no second cache or hand-written hashing engine.

The plain and HTTP exports still import no cache or framework package. The
optional cache entry imports @tanstack/query-core, not React. Its declarations
derive from the same compiler-checked functions and preserve input, result,
selection and tagged cache-key inference. Upstream host AbortSignal/timer types
are required, from browser libraries or Node declarations; the plain SDK remains
usable with ES2022 alone.

React adds hooks over the common runtime. Its adapter translates unique skip
tokens between its own Query Core dependency and the SDK's peer, instead of
assuming the package manager deduplicates module instances. Existing prototype
field protection, immutable input copies, client scope isolation, no implicit
retry, cancellation and acknowledged-write semantics remain in the shared path.

The explicit optional peer range is >=5.105.0 <6; exact tested Query Core and
React Query versions are both 5.105.0. The SDK fixture and initial web project
declare the direct peer normally. Users can still manage their dependencies,
scripts and locks. The web integration metadata advances to 0.2.0; automated
upgrade/migration convenience commands remain a separate required goal section.

## Upstream evidence and dependency review

The installed pinned Query Core source implements cache execution and
ensureQueryData/revalidateIfStale; React's queryOptions runtime returns its input.
The generated entry therefore returns typed upstream options instead of importing
React helpers. Query Core 5 marks fetchQuery/ensureQueryData deprecated for its
next major; new imperative examples use queryClient.query, while SWR examples
document the current supported ensureQueryData path and future migration need.
See [upstream QueryClient](https://tanstack.com/query/latest/docs/framework/react/reference/classes/QueryClient)
and the source distributed inside the pinned installed packages.

No new resolved package version was admitted. Query Core was already an installed
React Query dependency; this section adds explicit dependency/optional-peer
declarations. The SDK fixture lock retains the existing package integrity:
`sha512-7p56vRmT/z4UJ6sctGIdl5WRUQyUv53IUpUwEKi2g97zMbO7zh8ZsCLffGAvCPBR4xD72e3VXsDyLwXHhoNTMQ==`.
Its package license is MIT. Existing package-admission evidence remains applicable
to that unchanged graph; this is not a new blanket advisory-free claim.

Pinned pnpm 12.3.4 supply-chain checking passed 438 entries after fetching missing
registry metadata normally. Offline checking first failed for absent full cached
metadata; no policy was disabled. Only intended direct Query Core importer lines
changed in the starter lock. An initial inspection incorrectly treated the first
YAML document as the entire lock: pnpm 12 stores its environment and application
graphs in separate documents. The full application graph was already present.
See [pnpm's lock format](https://github.com/pnpm/pnpm.io/blob/main/docs/lockfile.md).

## Passed acceptance

- Offline/locked single-job api_contract Rust batch: all 10 tests pass, including
  export/dependency isolation and retained SDK compatibility checks.
  `target/api-core-contract.log`.
- Final source-current CLI build passes. `target/api-core-cli-build.log`.
- Plain SDK, HTTP, TypeScript, strict codecs, owner-edit/version guards, no retries,
  redirects and abort checks pass. `target/api-core-client.log`.
- Existing generated React/TanStack cache, hooks, types, mutation context,
  invalidation, cancellation, prototype keys and major isolation pass.
  `target/api-core-query.log`.
- Exact local tarball allowlist/integrity, detached consumers and owner dependency
  additions/workspaces pass. A core-only consumer installs no React/React Query;
  it proves cache reuse, request deduplication, SWR returning stale data before
  background completion, observer reads, mutation invalidation and typed result/
  selection/cache inference. A React consumer loads a real second Query Core
  package instance and proves skip-token adaptation. `target/api-core-package.log`.
- Updated web dependency integration and production build pass; installed SDK
  runtime hashes match generated source. `target/api-core-web-build.log`.
- External source copy, regenerated SDK, frozen offline install, TypeScript/
  production build, unchanged owner scripts/source and a normal dependency
  addition pass. `target/api-core-web-clean.log`.
- Actual native-served production React CRUD and existing static isolation,
  traversal, streaming and drain checks pass, using the retained native fixture.
  `target/api-core-native-web.log`. No backend code changed in this section.

The first expanded package attempt passed the core-only checks, then failed the
new separate-module skip-token witness because the earlier compiler asset lacked
the already-written late source repair. A source-current CLI rebuild and affected
package rerun passed. No input validation or test assertion was weakened.

Whole-project formatting passes twice with zero file/fingerprint changes; the
protected owner docs source hash is unchanged. Logs target/api-core-format-first.log
and target/api-core-format-second.log. git diff --check passes. The installed CLI
is hash-matched to the source-current debug build and its task API stale check
passes, target/api-core-installed.log. SHA-256:
`2f93465f31973a3d4e2dc4941634ac49dc569a6ecab321d7ac0667df33c28589`.
Full backend-family regression/measurement, Electron/mobile,
convenience commands/migrations and cancellation cleanup remain required.
