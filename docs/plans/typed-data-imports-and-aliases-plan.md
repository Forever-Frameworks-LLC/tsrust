# Typed data imports and project aliases

Status: owner approved both additions and their priority on 2026-10-11. This is
required S10 work, ahead of additional JSON runtime APIs and benchmark expansion.
Neither capability is implemented yet. Existing focused JSON evidence is retained.

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
