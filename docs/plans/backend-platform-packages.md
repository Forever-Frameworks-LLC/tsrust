# Backend platform templates and package decisions

Researched: 2026-10-26. Status: selected direction, not installed or security
certification. Resolve exact mutually compatible versions and hashes during the
template implementation section; do not fabricate a tested version matrix from
documentation. The owner explicitly accepts Electron Forge's Vite plugin.

## Selection

Prefer upstream, narrowly composed templates over an opinionated full-stack
boilerplate. Tsrust already owns the backend, so starters that bring their own ORM,
server runtime, RPC protocol, authentication product or deployment vendor add
competing responsibilities. Maintain a small versioned Tsrust integration template.

| Concern | Selected foundation | Rationale and implementation proof |
| --- | --- | --- |
| Web | Official Vite React/TypeScript template with shadcn's documented Vite setup | Familiar frontend build; no extra JS application server required. Pin compatible React, TypeScript, Vite, Tailwind and component dependencies together. |
| Web/desktop components | Upstream shadcn source installed locally | Owner choice; editable application-owned components. Select and record one primitive base/preset instead of mixing incompatible variants. Start with the task-list controls, not every component. |
| Desktop | Electron Forge with its official Vite/TypeScript template/plugin | Electron-maintained packaging route; reuse web UI. Plugin remains documented experimental, explicitly accepted by owner. Pin exact plugin/Electron compatibility and run packaged upgrade checks. |
| Mobile | Official Expo TypeScript/Router foundation plus the React Native Reusables maintained starter/installation | Keep Expo SDK alignment authoritative; use the upstream RNR template as an integration reference. Adopt its NativeWind/primitives dependencies only at compatible pinned versions. |
| Shared server state | TanStack Query and a thin generated Tsrust wrapper | Reuse caching, observers and retries; provide named calls/hooks and typed query/mutation options. |
| Workspaces/package installation | pnpm for the initial tested starter; preserve user-selected tooling | Standard lockfile and store, optional Tsrust orchestration. Users manage packages directly. Verify version-specific settings; no Turbo/Nx requirement. Existing npm/yarn/bun projects retain their tools. |
| Runtime validation | Codecs emitted from the compiler's checked boundary schema | One contract source. Do not require a second hand-maintained Zod model or another RPC server framework. Reevaluate a small codec dependency only when emitted validation materially warrants it. |
| Test tools | Existing compiler/LSP tests; TypeScript checks and Playwright for web/Electron where supported | Prove actual IPC, packaged startup and UI behavior. Mobile needs an actual Android emulator/device witness initially; iOS acceptance requires macOS. Avoid multiplying test frameworks without a need. |

Electron-vite/electron-builder remains an alternative, not an additional layer.
Official Forge integration is the current choice. Next.js/TanStack Start are not
required for a client-rendered prototype served by Tsrust. Add SSR only for an
identified application requirement. Expo web does not replace the selected React
web application. Do not invent a universal component compatibility layer.

## Evidence and limits

- [shadcn Vite setup](https://ui.shadcn.com/docs/installation/vite) documents
  React/TypeScript setup, Tailwind integration and workspace-aware component paths.
  [shadcn monorepos](https://ui.shadcn.com/docs/monorepo) is the workspace reference.
- [Electron packaging guidance](https://www.electronjs.org/docs/latest/tutorial/forge-overview)
  uses Forge. [Forge's Vite plugin](https://www.electronforge.io/config/plugins/vite)
  documents its experimental status and separate renderer/main/preload builds.
  The owner's follow-up accepts this tradeoff. Its build concurrency can be bounded.
- [Electron security](https://www.electronjs.org/docs/latest/tutorial/security)
  makes renderer isolation, sandboxing, restrictive CSP, sender validation,
  navigation controls and framework updates ongoing responsibilities.
- [Expo project creation](https://docs.expo.dev/get-started/create-a-project/),
  [Expo monorepos](https://docs.expo.dev/guides/monorepos/) and
  [RNR maintained templates](https://github.com/founded-labs/react-native-reusables-templates)
  support an upstream-based mobile starting point. Use the
  [RNR installation source](https://github.com/founded-labs/react-native-reusables/blob/main/apps/docs/content/docs/installation/manual.mdx)
  to verify NativeWind/Metro and primitive dependencies against the chosen SDK.
- [TanStack query options](https://tanstack.com/query/latest/docs/framework/react/guides/query-options)
  and [mutation options](https://tanstack.com/query/latest/docs/framework/react/reference/functions/mutationOptions)
  provide the primitives for generated wrappers without replacing the cache.
- [pnpm settings](https://pnpm.io/settings) and its
  [build-script settings source](https://github.com/pnpm/pnpm/blob/main/pnpm/docs/settings/build.md)
  show version-sensitive supply-chain controls. Exact policy must be tested against
  the pinned package-manager version, including Electron's binary-install step.

These primary sources establish integration paths and maintenance responsibilities.
They do not establish that arbitrary current versions are mutually compatible or
free of known vulnerabilities. Before admitting a template, inspect exact release
history, maintainer ownership, licenses, package tarball provenance where available,
lockfile advisory results and required install scripts. Record actionable findings
and validate upgrades. Official ownership and popularity are useful signals, not a
substitute for the resolved-dependency review. No packages were installed for this
research, and no blanket security claim is made.

## User ownership and Tsrust integration

### Initial generated-wrapper development fixture

`tools/api-client-tests` pins TanStack React Query/query-core 5.105.0, React
19.3.1, @types/react 19.3.0 and TypeScript 7.0.2 in a private npm fixture. Its
lockfile records exact transitive/platform integrity. Admission on Windows x64
with Node 24.19.0/npm 12.0.2 used disabled install scripts, zero known npm advisory
findings, seven verified registry signatures and three verified attestations.
The TypeScript CLI runs and the existing plain generated SDK passes its positive
and negative declaration consumer using ES2022 without DOM types. See the fixture
README for license/source observations and repeatable commands. These packages
are development-test inputs, not an accepted web/Electron/Expo template matrix;
generated wrapper runtime/hooks remain pending.

The installed query-core default object-key hash sorts fields through an ordinary
object accumulator. The wire contract permits a literal `__proto__` data field,
so generated query keys must preserve it. Prefer the existing validated,
schema-ordered wire input serialized as a key string (with explicit backend/user
scope and contract identity) over passing raw input objects to that accumulator.
Verify cache isolation, optional/null normalization, special keys, cancellation,
invalidation and actual upstream inference before accepting the adapter.

Owner clarification, 2026-10-26: generated and imported frontend applications are
ordinary user-owned projects. Users may cd into any frontend directory, install
or upgrade packages, edit manifests/locks/scripts, change bundlers, or manage the
application without Tsrust wrappers. This is supported use, not an escape hatch or
configuration drift requiring permission. Preserve their chosen package manager.

Tsrust owns initial setup/import, generated API packages, integration adapters and
optional orchestration (for example starting Vite or serving built assets).
Convenience commands can cover setup/dev/build without requiring users to learn
all upstream tools, but must delegate to inspectable ordinary project scripts.
There is no closed dependency environment or mandatory all-target version matrix.

Template releases record tested upstream versions, licenses and provenance.
Initial imports have lockfiles and normal integrity checks. Users can then add
packages normally. On Tsrust upgrades, offer migrations only for Tsrust-owned
integration code/configuration and dependency changes needed by that integration.
Preview edits, explain compatibility requirements, preserve user changes and let
users decline or apply migrations manually. Do not overwrite their entire lockfile,
force unrelated dependency upgrades, reject harmless customizations, or require
Tsrust for routine frontend development. Framework upgrades remain the user's
choice; publish tested integration compatibility ranges and useful diagnostics.

A managed Node/package-manager download may be offered for convenience, but it
must not replace a suitable user-selected toolchain or change global settings.
Downloaded artifacts need provenance, integrity checks, platform identity and
cache recovery. Respect chosen lockfiles and package-manager script policies;
Tsrust must not silently weaken those policies to get an installation to pass.
Inspect resolved dependencies and required scripts when admitting/upgrading the
Tsrust template/integration; do not audit or rewrite a user's whole project on
every build. Ordinary builds must not fetch unpinned integration generators.

Use a target/mount configuration rather than hardcoding one frontend. A backend
may serve several independently built apps under explicit URL prefixes or hosts,
with separate asset roots and navigation fallbacks. Users may also change Tsrust
server code and register these mounts themselves. Keep API routes and mounts
conflict-checked, isolate static paths, and apply fallback only inside the relevant
mount. External hosting and backend-only projects remain valid. No universal
frontend router or automatic conversion of application code is needed.

Tsrust can detect prerequisites and orchestrate builds, but Apple signing/Xcode,
Android SDK/emulators and registry identities remain explicit prerequisites.
Installed desktop end users need neither Rust nor Node. No paid cloud service is
required for the prototype.

Forge's [CLI packaging notes](https://www.electronforge.io/cli) call out its physical
dependency-layout requirement and pnpm's hoisted linker. Resolve this in a tested
starter configuration using the pinned pnpm major's supported setting; do not
silently change an imported user's workspace layout. Provide a compatibility
message and optional scoped migration if their packaging setup needs adjustment.

## Existing projects, export and publication

Both generated starters and existing projects are first-class consumers. Configure
the backend root, exported operation set, package identity/version, output directory
and enabled adapters in Tsrust configuration. UI scaffolding and frontend dependency
management are optional. Proposed workflows, not implemented commands:

```text
tsrust init --backend-only
tsrust api generate --out packages/api
tsrust api export --out ./dist/client
tsrust api pack
tsrust api publish --registry <explicit-registry>
```

Generation produces an importable workspace package. Export produces a standalone
package without absolute paths, backend source, source-tree-relative imports or
hidden runtime downloads. Pack produces a local npm-compatible tarball with an
explicit files allowlist, package exports, JavaScript, declarations, licenses and
API compatibility metadata. Support the prototype's ESM consumers first; add CJS
only if a real supported consumer needs it. The framework-independent entry point
must not import React, Electron, Expo or TanStack. React/TanStack adapters are
separate optional entry points or packages with compatible peer dependency ranges;
they must use the application's React/QueryClient instance.

Users can put an existing project under the Tsrust workspace and import the
generated package using their normal workspace mechanism. Configure its actual
path rather than moving or rewriting it automatically. An external application
can consume an exported folder/tarball or a published version without installing
the compiler. Do not assume a top-level `app/`, `web/` or `mobile/` tree is required.
Detect and preserve npm/yarn/pnpm/bun locks; reject ambiguous adoption with a useful
diagnostic. Backend build must not rewrite a consumer's package manifest.

Publication is a distinct explicit action with selected registry, package name,
version, visibility and credentials. Preview the exact tarball contents before
publishing, use the registry's supported authentication/provenance mechanism and
never embed credentials in generated source. Client packing/generation, tests and
backend builds never publish. Follow [pnpm pack](https://pnpm.io/cli/pack) and
[npm publish](https://docs.npmjs.com/cli/v11/commands/npm-publish/) instead of
implementing a registry protocol.

Acceptance requires two independent consumers: one Tsrust-generated starter and
one ordinary existing TypeScript project importing an exported/packed client with
its own lockfile. Also verify local workspace imports, configurable output paths,
package file allowlisting, stale/breaking API diagnostics and that no server-only
code enters either consumer. Keep publication support in scope, but exercise it
with an isolated local test registry or dry run until public publication is
explicitly authorized.

## Named calls and hooks

Proposed generated surface, to settle with executable TypeScript witnesses:

```ts
const tasks = await api.tasks.list({ completed: false });
await api.tasks.create({ title: 'Ship prototype' });

const tasksQuery = queries.tasks.list.useQuery({ completed: false });
const createTask = queries.tasks.create.useMutation();
createTask.mutate({ title: 'Ship prototype' });

await queryClient.prefetchQuery(
  queries.tasks.list.queryOptions({ completed: false })
);
```

The plain client is framework-independent and returns promises without silently
adding caching/retries. React hooks obey normal hook rules and use the provider's
QueryClient. Generated options retain type inference and documented override
points. No string command names, manual fetch wrappers, handwritten query keys or
duplicated DTOs in normal application code. Keep mutation invalidation explicit
and typed; do not guess which reads every mutation affects. Cache identity includes
the service/transport and authentication scope so local and hosted data cannot mix.

The named surface selects a transport at client construction. HTTP and Electron
preload carry the same checked operations, codecs, public failures and policy.
Capabilities remain allowlisted; exposing a named method does not grant raw IPC,
shell access or unchecked arbitrary function invocation. No Node-only dependency
may enter the plain client or Expo bundle.
