# Declared Rust crate bindings and owned data

Source acceptance covers project dependency manifests, locked offline native builds,
relocatable exports, owned struct/enum adapters, checked editor facts and readable
regex bindings. Public Rust declarations define the bindings; JSON selects Cargo
inputs only. Signatures are not duplicated in a separate binding language.

## Accepted checks

- 102 affected compiler/CLI checks: 57 project-filter unit checks, 15 CLI/dev checks,
  16 project integration checks, six Rust bridge checks, two dependency-bundle
  checks, three shared runtime-build checks and three offline package-export checks. The dependency parser's stricter
  exact-version validation has a supplemental focused unit check.
- 196 affected editor checks across checker agreement, completion matrix,
  consistency, project intelligence, incomplete source, sessions and Rust bindings.
  The full matrix's other rows passed; the new binding row initially exposed a
  synthetic-source location defect. After repairing compiler facts and editor
  locations, that row passes all four real-server typing modes. Imported function
  results expose record fields without requiring a separate type import.
- Four VS Code extension tests and extension compilation pass.
- Struct and enum round trips preserve heap-buffer addresses, including pool-task and dedicated-thread returns. Nested records,
  primitive buffers, Result error records, unit/tuple/named variants, three distinct
  empty variants and move rejection pass. Custom Drop, borrowed records, private
  fields, recursive data and generic layouts fail closed. Existing worker panic,
  cooperative cancellation and five guard disposals remain accepted.
- The real regex example returns `Found 314 at byte 6`. Both relocated debug and
  release exports build offline with an empty Cargo home after deleting the source
  files and original vendor cache. Default Unicode support versus defaultFeatures
  false/features std produces true versus false for a Greek character class.
  macOS leaks reports zero leaks and zero leaked bytes for the optimized example.
- The new matched Rust-data recurrence workload passes the independent BigInt
  checksum oracle (4791373295) in native and Node-after-tsc execution. Its timing
  protocol remains three processes, eight warmups and 31 samples per variant.
  Timing and installed-consumer refresh are pending at this source checkpoint.

Closed command logs and benchmark records are imported byte-for-byte into
benchmarks/records.sqlite under tsrust-rust-bindings-* and benchmarks/threads/
record paths. Failed attempts remain alongside repair evidence. Full unrelated
compiler/editor regressions are not repeated for this focused bridge extension.

## Scope and follow-up

The adapter file explicitly exports selected functions and concrete data. Crate
internals remain private Rust. Cargo owns resolution and vendoring; native builds
use --offline --locked and include the exact dependency configuration in cache
identity. Binding manifests within one program must share versions/features/lock.
Registry dependencies are supported; git/path dependencies, returned borrows,
opaque resource handles and vectors of records are outside this pass.

The CLI currently provides bindings resolve (including --offline). A normal
package-add command should create/update dependency configuration and resolve it;
users should not need to hand-edit generated Cargo files. Curated packages should
supply intentional public adapters, with custom adapters available for other crates.
Migrating existing stdlib implementations to the shared binding machinery is a
separate change, especially where opaque resources require lifecycle contracts.
