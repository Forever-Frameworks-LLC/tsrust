# Checked API client foundation

Date: 2026-10-26. Baseline checkpoint: `e3974b2f`. Status: focused acceptance passes;
the working application prototype remains incomplete.

## Implemented boundary

`tsrust api generate` reads an explicit `tsrust.api.json` allowlist, checks its
entry graph and resolves selected named function exports against the checked
program. It uses the linker's canonical symbol helper and concrete checked type
identities, not display-string parsing or separately authored DTO interfaces.
Library-only entries work without an executable main function. Unselected native
resource exports do not become operations or enter the generated package.

The manifest has deterministic operation ordering and a wire-contract hash that
excludes absolute paths/package version. Generated JavaScript ESM and declarations
provide named calls, strict argument/result codecs and structured error handling.
They have no framework dependencies, install scripts or backend source. Runtime
support carries the repository's MIT notice; package publication is private by
default and no publication occurs during generation.

Supported values include safe integers, finite numbers, booleans, strings, null,
literals, dense arrays, tuples, optional record fields and discriminated data
unions. Native handles, recursive schemas, unsupported callable/collection values
and unsafe integer literals reject. Special JSON keys remain data properties;
manifest JSON is parsed as data rather than embedded as object-literal syntax.
Literal placeholder-like strings remain byte-exact.

Outputs use a generated hash marker, an exclusive generator lock, preflight
owner-edit checks and atomic file replacement. Existing application directories
are preserved. `--check` detects stale output without writes. A partially written
generation can accept either its prior or intended file hashes on retry. Output
symlinks/junctions reject, while ordinary linked-package consumption is supported.

## Validation

- Whole-project formatting passes; new persistent fixtures were formatted and
  exercised in formatted form. Temporary acceptance fixtures also require a stable
  second formatting pass before compiler checks. No benchmark fingerprints changed.
- `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --offline --locked -p tsrust-compiler --test api_contract -- --test-threads=1`
  passes five tests: checked nullable/optional/literal contracts and deterministic
  output, explicit export selection/name conflicts, unsupported wire rejection,
  async eventual results, and portable package metadata. Build observation:
  11m15s with one job; this is not a benchmark sample.
- `node tools/verify-api-client.mjs` passes against the current normal CLI. It
  exercises all advertised codec families, safe-number boundaries, no transport
  invocation for invalid inputs, non-mutating validation, malformed/public errors,
  literal fidelity, special keys/accessors, sparse arrays, exact TypeScript
  inference and negative type witnesses. Node and TypeScript resolve an ordinary
  linked package from an independent user-owned project without rewriting its
  package manifest/scripts.
- The same witness checks schema changes/staleness, preservation of prior output
  on backend errors, edited generated files, existing non-generated directories,
  native-resource/recursive/unsafe-literal rejection and unselected resource
  exports. Its temporary junction requires normal filesystem permission on this
  host; the sandbox EPERM failure is retained. The unchanged witness passes with
  that permission and remains entirely headless.
- The final formatted native fixture builds and runs through the normal CLI:
  `tsrust build examples/api-contract/src/main.tsr -o target/api-contract-native.exe`,
  then the executable prints `API contract native witness passed`. Assertions cover
  task data, literal strings, async results, special keys, tuple fields, finite
  numbers and booleans. The fixture uses supported named union declarations and
  an explicitly typed expected literal for strictEqual's same-concrete-type contract.

The initial `--lib` filtered test attempt could not compile the existing Unix-only
`std:fs` runtime unit module on Windows. That full-library platform gap is not
passed or suppressed. API witnesses live in a focused integration target and test
the public API. An initial new SHA-256 LowerHex assumption was repaired using the
repository's existing per-byte hex formatting pattern.

Current verified normal CLI SHA-256:
`74394e67c9bbfc7453d0a18aa022e03d3de7b6c00027aa304e08c69a976e401d`.
Local diagnostic logs are under `target/api-*.log`; no logs/source snapshots were
imported into analytics. No performance workload was timed, no broad matrix/CI
was restarted, and no dependency package or release was published.

## Remaining prototype work

This foundation generates contracts and client calls. HTTP transport/dispatch,
server-side wire validation, request policy/context, SQLite task persistence,
TanStack named hooks/options, React/shadcn web/Electron, Expo/RNR, mounts, packaging,
cancellation, scoped migrations and the full backend-family review remain open.
The manifest hash is not a rolling-compatibility policy. See
[the guide](../../website/src/content/docs/backend/api-clients.md) and [prototype goal](../agent-goal.md).

Accepted source checkpoint: `0075eb3e`. The normal installed compiler was atomically
refreshed from the verified executable; installed `api generate` and `--check`
both pass for the six-operation example. Existing owner `docs/src/main.tsr`
changes and `.agents/` remain intact.
Desktop control is released; continue with file/terminal and headless tools.
