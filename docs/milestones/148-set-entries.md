# Gate 148: Set entry snapshots

Status: accepted with focused compiler/editor checks, runtime harness and matched
untimed verification. No full regression or timing run was performed.

## Behavior

Set.entries() takes no arguments and returns an independent [Key, Key][] array
in insertion order. Each tuple contains the key twice. Owned keys are explicitly
copied into both fields; the receiver stays usable. Empty Sets return empty arrays.
Adding an existing key preserves its position; deletion and reinsertion moves it
last. Number keys preserve SameValueZero, including NaN and normalized zero.
Later source mutation or disposal does not change the snapshot.

Enum and union key types, generic functions and temporary receivers retain typed
tuples. Wrong argument counts or incompatible result annotations are static
errors. This operation returns an array rather than a live iterator. Access and
ordinary bindings still do not implicitly clone. See [the guide](../sets.md).

## Implementation and editor

A separate inline-never helper materializes the checked tuple type. Native
emission copies keys while walking the existing insertion slots. Storage and
equality policy are unchanged. Catalog documentation includes behavior, errors,
snapshot semantics, an executable example and ownership notes.

The failing editor witness caught a catalog placeholder mismatch: Set help
substitutes Element, whereas Map help substitutes Key/Value. The signature now
uses the correct family placeholder. The original failure is retained.

## Evidence

Logs are retained under target/editor/set-entries-*.log.

- Original compiler red run and check reject unsupported Set.entries. Independent
  pinned Node reference passes with explicit Array.from iterator materialization;
  adapted reference sources are preserved with the evidence.
- set-entries-final-native.log: new compiler suite 2/2, covering strings,
  insertion order, source clearing, empty results, NaN/zero, enums/unions,
  generics, temporary receivers and static contracts.
- set-entries-native.log: adjacent Set 6/6 and Map snapshots 4/4 pass. The first
  Set.entries witness passes there too; the later generic/static additions are
  checked in the focused final suite rather than repeating adjacent suites.
- set-entries-final-editor.log: new editor check 1/1. Adjacent Set editor 4/4
  passes in set-entries-editor.log, which retains the original placeholder failure.
- Nine catalog examples match native and adapted Node output in
  set-entries-documented-native.log and set-entries-documented-node.log.
  Catalog/fixture synchronization passes in the adjacent editor suite.
- set-entries-harness.log: runtime harness 374/374 on refreshed consumers.
- set_entries verifies five cases per engine, native and node-tsc, in
  results/2026-10-06T06-51-39-122Z-verify.json. verifiedOnly is true and timing
  arrays are empty. The workload remains unmeasured. Final acceptance flags change
  afterward; final registration is checked separately and the original manifest
  is preserved in archive/set-entries-verification-start.
- Workload check passes before refresh. Debug/release builds pass. Five consumers
  refresh atomically; hashes are in set-entries-consumers.log, with previous bytes
  preserved in archive/set-entries-consumers-start.

Matched sources, independent oracle, source contracts, registration, runtime and
profiling hooks, report category, fingerprints, coverage and roadmap are ready.
Original compiler, editor, fixtures and metadata remain in archive/set-entries-*-start.
Collection callbacks, borrowed Map iteration, live iterators and collection-family
measurements remain later work. No Git.
