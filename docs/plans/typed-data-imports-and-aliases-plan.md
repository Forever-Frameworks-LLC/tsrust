# Typed data imports and project aliases

Status: owner approved both additions and their priority on 2026-10-11. This is
required S10 work, ahead of additional JSON runtime APIs and benchmark expansion.
Compiler file loading now has focused alias/extension/index acceptance; editor, dev
invalidation and typed data-module integration remain pending. Existing JSON evidence is retained.

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
