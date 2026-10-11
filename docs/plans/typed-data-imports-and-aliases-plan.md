# Typed data imports and project aliases

Status: owner approved both additions and their priority on 2026-10-11. This is
required S10 work, ahead of additional JSON runtime APIs and benchmark expansion.
Alias/extension/index resolution has focused compiler, editor and dev acceptance.
Typed data-module integration remains pending. Existing JSON evidence is retained.

## Public behavior

Support default imports from `.json` and `.jsonc` files. Parse and validate them
at build/check time and embed their data in the native program. Execution does
not reopen source data files. Strict JSON remains strict; JSONC permits comments
and trailing commas as an explicit file-format choice. Diagnostics refer to the
original data file and location, not synthetic generated source.

Infer the exported data type from its contents. The compiler and editor consume
the same inferred type: object properties, nested records, arrays, scalar types
and nullability. Provide field completion, hover, checked access and assignment
errors, and navigation to the originating property where practical. Do not expose
`any`, an unchecked generic cast, or a dynamic JsonValue that requires manual
field decoding as a substitute for typed imports. Type inference must handle
empty arrays, heterogeneous arrays, differing object shapes, null values, escaped
property names and numeric boundaries deliberately. Unsupported representability
must produce a source diagnostic rather than silently discard data or widen to an
unsafe type. Preserve ordinary ownership and explicit clone rules; imports must
not introduce hidden shared mutable state or implicit deep copies.

The owner approved the storage contract on 2026-10-11: one immutable value per
canonical data file, accessed read-only across importing modules and functions.
Reads do not copy. `clone(data)` explicitly creates an owned copy, which can be
mutated under ordinary ownership rules. Aliases must share the same stored value.
Reject mutation and ownership transfer through an imported binding; do not hide
per-use allocation, deep copying, shared mutable state or lifetime erasure.

Example target behavior:

```typescript
import settings from '@/data/settings.jsonc';
// settings.jsonc: { "server": { "port": 8080, "host": "localhost" } }
console.log(settings.server.host);
// settings.server.unknown: checked error with valid property suggestions.
```

Use `paths` in `tsrust.config.json` for project aliases:

```json
{
  "schemaVersion": 1,
  "name": "example",
  "build": { "entry": "src/main.tsr", "output": "dist/example" },
  "paths": { "@/*": "./src/*" }
}
```

Resolve mapping targets relative to the configuration file, independent of the
shell working directory. Support exact mappings and one matching wildcard, with
exact matches before wildcard matches and the most specific prefix winning.
Reject malformed or ambiguous mappings with configuration diagnostics. Preserve
`std:` identities and canonical file identity; aliases are alternate paths to the
same module, not duplicate modules or package downloads.

Support explicit file paths, extensionless `.tsr` files and directory `index.tsr`
resolution. JSON and JSONC imports require their extensions so selecting the data
format is explicit. Prefer an existing direct `.tsr` file to a directory index;
missing targets report the requested specifier and attempted paths. Existing
explicit relative imports keep their behavior.

## Implementation order and acceptance

1. Shared project configuration and resolution: CLI, compiler project graph, LSP
   snapshots and dev watcher use one resolution contract. Add aliases and source
   extension/index lookup with focused failing tests first. Verify nested working
   directories, aliases to the same file, missing targets, cycles, config changes,
   unsaved source overlays and default-export imports through aliases.
2. Data-module representation and type inference: validate original JSON/JSONC,
   preserve original source spans, infer safe types, and link typed default data
   exports into checked native emission. Verify root scalars/arrays/objects,
   numeric boundaries, nulls, empty and heterogeneous arrays, missing properties,
   duplicate keys, Unicode, malformed input and size/depth limits. Avoid a
   synthetic executable top-level initializer workaround.
3. Editor and project integration: nested completion/hover, invalid-access
   diagnostics and type updates after data edits. Include unopened dependencies,
   supported unsaved data overlays, rename/delete, alias/config edits, and
   stale-diagnostic removal. `check`, `build`, `run`, `test`, `emit --package` and
   `dev` must agree. Data edits invalidate builds and inferred dependent types;
   failed edits retain the last working dev program under the existing contract.
4. Acceptance: native execution from embedded data after relocating the exported
   package and removing original inputs; executable documentation, project
   scaffolding examples, focused editor protocol checks and installed-consumer
   smoke. Complete matched workload/oracle and allocation support where the new
   behavior introduces runtime costs. Record implementation acceptance separately
   from the later S10 family measurement milestone.

Follow with remaining S10 dynamic operations, typed streaming, borrowed adapters,
public writers and the full correctness/performance matrix. Do not lose those
requirements when prioritizing imports.

## Shared resolver foundation

`project_resolution.rs` defines filesystem-independent alias expansion and ordered
file candidates. Focused checks cover configuration-relative targets, exact-match
precedence, longest wildcard-prefix selection, wildcard suffixes, ambiguity,
malformed mappings, explicit data extensions and `.tsr` before `index.tsr`.
A more specific wildcard wins even if less specific mappings would be ambiguous.
Providers remain responsible for canonical identities and snapshot ownership.

The initial foundation checkpoint did not connect source providers. The compiler
file-loader integration below now consumes it; editor snapshot and dev
invalidation acceptance remain pending. Focused evidence is retained
as `tsrust-import-resolution-*` in SQLite.


## Compiler file-loader integration

The file provider discovers the nearest `tsrust.config.json` from each importing
file, validates its paths mapping, expands aliases and resolves shared candidates.
Canonical identities keep explicit and aliased paths to one file identical.
Missing candidates report attempted paths. Unreadable or malformed configuration
fails instead of silently searching farther up. CLI project configuration accepts
and validates `paths`. Source headers now defer alias existence and extensionless
resolution to the provider; unsupported data imports remain rejected until their
checked representation is implemented.

Two focused integration tests cover graph checking, canonical identity,
file-before-index precedence, missing-path diagnostics, malformed configuration,
unmapped package rejection and native CLI execution through project defaults.
The executable fixtures also pass formatter idempotence. The existing malformed
import diagnostic test passes with resolution errors for unresolved specifiers.
Evidence: `tsrust-alias-loader-*` in SQLite. Installed consumers have not been
refreshed; compiler/editor parity and invalidation must land before user-facing
feature acceptance. No benchmark timing or complete family regression was run.

## Editor snapshot resolution

Workspace snapshots now use the shared alias discovery and candidate rules.
Configuration reads have a separate immutable cache and do not appear as Tsrust
source modules. Lexical/canonical identities and missing candidates are observed
for dependency invalidation. Snapshot merging rejects conflicting configuration
contents, and disk creation/rewrites invalidate reuse without waiting for watcher
notifications. Unsaved configuration and source buffers work at the snapshot API.

Four focused snapshot tests cover alias checking, unsaved configuration changes,
file-before-index changes, absent/new disk configuration, disk rewrites and the
existing combined-snapshot consistency witness. Existing workspace and workspace
scan suites pass. Evidence is retained as `tsrust-alias-editor-*` in SQLite.

This is not complete editor acceptance: the protocol currently accepts only
Tsrust document buffers and the VS Code extension watches only `.tsr` files.
Wire configuration notifications and supported config overlays, verify actual
hover/completion/navigation and dev invalidation, then refresh installed consumers.
The lower-level snapshot tests do not prove protocol delivery or UI behavior.

## Configuration protocol delivery

The LSP now accepts `tsrust.config.json` buffers as configuration inputs without
scheduling them as Tsrust source documents. Open/change/close updates flow through
project dependency invalidation, retaining buffer versions and restoring disk
fallback on close. The VS Code client selects these files and watches their
creation, changes and deletion alongside source files.

A live protocol witness opens an unsaved configuration and aliased dependency,
checks the importing file and its checked hover, changes the mapping to a missing
target, restores it, and closes the unsaved configuration. Import diagnostics
appear and clear accordingly. The existing live unsaved-dependency lifecycle test
also passes. Extension compilation passes. Evidence: `tsrust-alias-protocol-*`
and `tsrust-alias-extension-build.log` in SQLite. These are protocol/build checks;
installed consumers have not yet been refreshed.

Remaining dev correctness issue: its source provider currently delegates alias
resolution to live filesystem configuration, while source content comes from a
captured build snapshot. Resolve against captured configuration and candidate
identities before accepting dev behavior; include configuration edits during a
build and nested project configuration discovery. Existing watchers recognize
configuration filenames, but that alone does not prove snapshot consistency.

## Dev snapshot alias resolution

Dev now selects captured source candidates and reads captured project aliases
within its watched root. Creating a higher-priority source file, changing an
alias or deleting its previous target after capture no longer changes emission
for that request. Single-file identity also uses the captured provider. Nested
configuration files join source files in the watcher input map; configuration
paths are canonicalized before deduplication, including temporary-directory aliases
on macOS. Ordinary source discovery used by checking/formatting still returns
source files only.

Focused witnesses preserve emission after those disk edits and verify that a
fresh snapshot observes the changed mapping. Existing dev watcher/event tests are
included in the bounded batch. Records: `tsrust-dev-alias-*` in SQLite.

Remaining acceptance includes end-to-end dev restart/last-working-program behavior,
installed CLI/editor refresh and alias completion/navigation. Review symlink and
outside-root dependency behavior before final acceptance: the existing discovery
policy excludes symlinks and generated directories, and this captured resolver
must not claim immutable identities for files outside its captured map. In-root
uncaptured source candidates are rejected rather than read live. Broader watched
dependency coverage is not proved by this snapshot slice.

## Live dev and checked editor witnesses

A real `tsrust dev` process now passes an alias-edit lifecycle test: an initial
program remains alive after a mapping produces a reported build failure; restoring
a valid mapping builds the new program, stops the previous process, and starts the
replacement. Interrupting dev reaps the replacement. The test uses isolated
projects and formatted source, bounded waits, and actual process-liveness checks.

A checked editor witness imports a default function through `@/components`, resolves
`src/components/index.tsr`, preserves source documentation and the string signature
in hover, offers the imported local name in completion, and navigates to the real
dependency URI. This proves symbol completion through an alias, not path-string
suggestions. Evidence: `tsrust-dev-alias-e2e*`, `tsrust-alias-intelligence*` and
`tsrust-alias-acceptance-format.log` in SQLite.

Installed-consumer refresh and dependency-boundary review remain outstanding;
these focused witnesses do not close the complete import feature or S10.

## Captured dependency boundaries

Dev snapshots now capture the configured entry's header dependency graph in
addition to discovered project files. Capture retains canonical import results,
dependency bytes and read failures, including external files and symlink targets.
A captured request does not follow a later symlink retarget or substitute repaired
bytes for a captured read error. Changes during capture reject that snapshot for
retry; project source stamps are revalidated after dependency capture.

Native events continue to cover ordinary project files. Only dependency paths
outside that tree (including imported files under excluded directories) receive
metadata reconciliation on supervisor polls. Missing attempted external paths
and configuration discovery paths are tracked as well. Capturing headers adds
frontend work when inputs change; no performance improvement or timing claim is
made for that work, and the S10 milestone must retain its measurement obligation.

Twelve bounded dev checks pass, including external edits, symlink retargeting,
frozen invalid UTF-8 failures and watcher recovery. A live dev test also edits an
outside-root imported file and observes a replacement process, then verifies bad
alias recovery and shutdown cleanup. Evidence: `tsrust-dev-boundaries-*` in SQLite.
Typed data imports and the mandatory API audit remain unfinished S10 requirements.


## Installed alias consumers

CLI and LSP release binaries and the compiled VS Code extension bundle were
refreshed from `5ebaf46e`. An isolated installed-CLI project executes a default
import through `@/components` and its directory index. The installed LSP reports
no diagnostics and returns the imported string signature in hover.

After reloading the repository editor window, Tsrust checking is enabled and the
observed status shows no problems; the previous JSON workload error badge clears.
The separate owner project window also shows no problems, but its reload could
not be confirmed through the UI. No owner source files were edited. Evidence:
`tsrust-alias-consumer-*` and `tsrust-alias-installed-smoke.py` in SQLite.


## Data source representation foundation

The internal data-module parser validates through the existing bounded JSON
backend, then retains original byte spans for values and decoded property names.
JSONC accepts comments and trailing commas; strict JSON retains its existing
syntax rules. Duplicate properties retain the last value and its source location.
Number lexemes remain exact for subsequent native representability checks instead
of being rounded before type inference. Limits match the current parser defaults:
16 MiB, 64 levels, and one million entries per collection and total value nodes.

Focused parser witnesses cover original Unicode offsets, escaped names and strings,
scalar roots, nested arrays/objects, duplicate keys, numeric boundaries, malformed
input and depth limits. Evidence: `tsrust-data-module-*` in SQLite. This is an
internal representation checkpoint only: data imports, inferred types, native
ownership/linking, editor integration and end-to-end acceptance remain pending.


## Structural data inference foundation

An internal shape pass now infers scalar, record, array, nullable and heterogeneous
data-union shapes. Compatible record elements merge their fields; absent fields
become optional. Nested empty arrays can obtain their element type from nonempty
siblings. An empty array with no such context receives a source-span error rather
than an unsafe element type. Mixed data values use the compiler's existing data
union domain; they are not rejected merely for being heterogeneous.

Integer spellings must fit native signed 64-bit `int`. Decimal/exponent spellings
and negative zero infer `number`. When a mixed numeric array widens integers to
`number`, every integer must remain exactly representable; errors retain the
original numeric span. Native type construction and value lowering must preserve
these checks, optional fields and union tags in the subsequent integration.

Focused inference witnesses and their failing precursors are retained as
`tsrust-data-shape-*` in SQLite. This remains internal groundwork, not public
data-import acceptance or a performance measurement.


## Checked data declaration and value projection

The internal lowering pass constructs ordinary record declarations, structured
array/union annotations and value AST nodes directly from validated data. It
preserves property and value spans, adds `undefined` to optional fields, selects
numeric expressions after precision validation, and uses compiler-owned record
identities. It does not generate executable source text or a top-level initializer.

Focused witnesses run the resulting AST through ordinary analysis and native
emission, check missing-field rejection, and observe nested records, optional
fields, mixed unions, nullable values and nested arrays. The fixture source is
formatted and checked for idempotence before AST insertion. Evidence is retained
as `tsrust-data-lowering-*` in SQLite. Import storage, module graph integration,
editor integration and public acceptance remain pending.


## Data module graph loading

Module loading now recognizes explicit JSON/JSONC dependencies, validates syntax
and inferred representability at original source locations, and publishes only a
default data export. Cached headers compare original data bytes; invalid edits
fail and corrected edits recover. Data modules retain their parsed values apart
from executable declaration source.

Focused graph witnesses cover JSONC input, strict JSON rejection, UTF-8 error
locations, changed cached data, numeric representability and named-export
rejection. Evidence: `tsrust-data-project-*` in SQLite. The initial graph checkpoint reported `data-import-storage` during executable
checking. The storage integration below replaces that temporary diagnostic.


## Immutable data storage integration

Data modules now link a compiler-owned initializer and one native immutable
`LazyLock` owner per canonical source. Function-local bindings borrow that owner;
reads do not reconstruct the value or reopen its source. Ordinary checked record,
array and union layouts remain in use. The initializer is direct compiler AST,
not generated Tsrust source or a user-visible executable top-level declaration.

The checker rejects mutation and non-copy ownership transfer through data imports
with `data-readonly` and directs users to explicit `clone`. Scalar reads retain
ordinary copy semantics. Focused native witnesses read through multiple modules
and callbacks, mutate an explicit clone without changing the source, and reject
whole-value, field and callback ownership transfers. CLI acceptance also checks
canonical alias sharing and execution after removing the original JSONC file.

Evidence: `tsrust-data-storage-*` in SQLite. Broader ownership/closure boundaries,
all entry modes, editor integration, documentation/examples and matched benchmark
support remain required before public feature acceptance. Installed consumers
have not been refreshed for data imports. No new timing claim is made.


## Editor data schema and live buffers

Editor indexes now consume compiler-derived data schemas, including original
property spans. Imported nested fields provide completion, hover and navigation
to their JSON/JSONC declarations. Checked record facts use stable data declaration
identities rather than displaying the entire JSON object as a type name. Import
hover explains embedded immutable storage and explicit cloning.

The extension watches JSON/JSONC files and synchronizes their open buffers. The
language server treats these as auxiliary project inputs, so they invalidate
dependents without being checked as Tsrust source files. Focused protocol tests
cover unsaved data type changes, diagnostic recovery and closing an unsaved input;
existing config and Tsrust dependency lifecycle checks still pass. The nested
field witness also checks updated hover types after editing data. The extension
compiles successfully. Evidence: `tsrust-data-editor-*` in SQLite.

Incomplete-source assistance, additional data shapes and ownership boundaries,
all CLI entry modes, dev/package acceptance, public documentation and matched
benchmark support remain pending. Installed consumers still need a verified
feature refresh. These focused checks do not close S10.


## Dev and package workflow acceptance

A live dev witness exposed a missing retained-cache entry for data modules. Data
initializers now enter the same generation cache as source declarations, so
importers invalidate correctly. Native filesystem subscriptions now include JSON
and JSONC files. The live witness confirms start, invalid-data retention, recovery,
replacement of the old process and shutdown cleanup; the existing alias dev
witness still passes.

An exported data-import package builds offline after relocation and deletion of
original inputs, with an empty Cargo home and the shared Cargo target. It needs
no vendored JSON parser. Additional native execution covers scalar roots, mixed
arrays, Unicode and escaped property names. Literal string record indexing now
uses ordinary checked field access. The module guide documents aliases, data
ownership, inference and bounds. Evidence: `tsrust-data-workflows-*` in SQLite.

Further ownership and editor boundaries, test-command integration, matched
benchmark support and installed-consumer verification remain pending.


## Additional ownership and editor boundaries

Focused acceptance now covers `tsrust test` callbacks, an escaping callback that
reads immutable imported data, explicit cloning of owned fields, and mutation of
quoted properties on an explicit clone. Imported quoted properties, array
assignments, aliased array mutation, callback mutation and returning an owned
field without cloning all produce `data-readonly`. These checks passed without
additional implementation changes.

Completion for nested data fields remains available while the importing function
body is incomplete. Evidence: `tsrust-data-boundary-*` in SQLite (three focused
tests passed; whole-project formatting changed no files or fingerprints).
Matched benchmark support, additional editor shape coverage and installed-consumer
verification remain pending. No timing or S10 completion is claimed.


## Matched imported-data benchmark support

`native_json_import` now has matched Tsrust and TypeScript sources, a pinned JSON
input, source and input guards, an independent arithmetic oracle, ordinary scalar
ABI verification, report registration and harness support for typed JSON imports.
Each iteration reads one imported scalar and scans a three-element imported
array. Loading occurs before timed calls; compiler hoisting remains allowed. This
workload does not claim to isolate memory latency, parsing or clone costs.

The reusable `benchmarks/runtime/typed-data-imports-smoke.mjs` accepts `TSRUST`
and validates five native/Node-after-tsc cases with zero timing samples. Both
registration tests pass. Evidence: `tsrust-data-benchmark-*` in SQLite. Timing
remains deferred to the JSON family milestone; installed-consumer refresh and
additional editor shape coverage remain open.
