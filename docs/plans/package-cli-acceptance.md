# Package CLI acceptance

Implemented commands: add, update, features, packages and remove. Public usage and
transaction limits are documented in [Rust packages](../guides/rust-packages.md).
Package operations create/update configuration; no handwritten JSON or Cargo.toml
is required. The regex example is migrated by the actual CLI and gains two executable
acceptance tests.

## Focused source checks

- 95 compiler/CLI checks pass: 18 CLI/dev/publication unit checks, seven package CLI
  integrations, six Rust bridge integrations, five retained project-session checks,
  three offline package-export checks and 56 testing-library checks.
- 188 editor checks pass across Rust bindings/hover, real-server completion matrix,
  checker agreement, consistency and project intelligence. The affected matrix row
  passes all four typing modes. Managed embedded-lock edits invalidate retained
  project analyses; readable Rust type hovers contain no empty inline comments.
- Package integrations exercise add without a version, exact pins, repeat add,
  feature replacement/enable/disable/defaults, explicit version updates, unused
  removal, used-crate removal rejection, failed resolution, failed Rust adapter
  compilation, rejected callers, failed project tests and successful previews.
- Concurrent commands retain both feature changes. Publication unit checks retain
  the old generation with a partial pending file and continuously read complete,
  matching configuration/lock generations across atomic replacements. Candidate
  source reads remain frozen and changed inputs abort publication.
- Preview/failure removes newly created unpublished vendor generations. Existing
  published cache closures and custom adapter source remain unchanged.
- The real regex example's package command runs two passing tests before publishing
  the managed generation. Its native source is preserved. The existing adjacent
  lockfile remains unchanged as a legacy input.
- Managed locked sources restore offline without creating a sidecar lockfile.
  Package inspection works with an empty Cargo home using the vendored closure.
  Relocated debug/release exports build offline and produce `Found 314 at byte 6`
  after deleting the original source and cache.

Validation exposed an existing bridge defect: test registration lifts functions,
and native adapter recognition previously missed their local identities, executing
source stubs. Native recognition now retains both the lifted name and original
source declaration identity. The real regex registered test guards this behavior.

Closed original logs and the managed-export acceptance script are retained byte-for-
byte in benchmarks/records.sqlite under tsrust-packages-* paths. Failed attempts are
preserved alongside their repairs. Release consumer refresh, installed hover checks
and a representative untimed Rust-data workload check remain pending at this source
checkpoint. Full unrelated regressions and fresh performance measurements are not
claimed by these focused checks.

## Boundaries

Compilation validates Rust APIs and Tsrust ownership/types. Tests validate exercised
behavior; untested behavior and platforms remain unknown. Tests run in their normal
project directory; their own I/O is not rolled back. One program uses one managed
dependency configuration. Compiler-owned dependency pins are not configurable.
Curated regex is a starter adapter, not an automatic binding of every crate API.
