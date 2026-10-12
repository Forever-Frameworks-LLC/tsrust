# Program-owned module state

Status: focused globals implementation checks pass. Source checkpoint and consumer
refresh are next. S11 remains in progress.

## Approved direction

Support module-level let and const, exports/imports, ordinary parameter passing,
inferred borrowing and explicit cloning. The program owns each canonical module's
storage once. Const prevents rebinding; it does not freeze owned objects. Let permits
rebinding. Neither keyword adds implicit cloning, locks or reference counting.
Exported bindings are live; importers cannot rebind them. Mutation through an imported
object remains subject to ordinary exclusive-access checking.

Initialize dependencies before dependents and bindings in declaration order. Keep
import-cycle rejection. Detect reads before initialization, including indirect calls.
Initialization failure prevents the entry body and cleans up initialized owners.
Main remains optional for checking; define executable module initialization without
requiring a dummy main. Preserve existing main execution for current applications.

A live borrow constrains all conflicting access, including access through functions,
callbacks, aliases and global reassignment. Consuming program-owned storage must not
leave a globally reachable uninitialized binding. Support normal borrowing and
explicit clone; an explicit replacement/take contract needs separate consideration.
Do not use static lifetime substitution, unsafe mutable statics or hidden shared owners.

## Implementation and acceptance

1. Preserve top-level declarations and source locations through parsing, module
   exports, canonical linking, source walks and formatter round trips.
2. Build a program-owned initialization scope with checked global access summaries.
   Evaluate reuse of existing lexical capture/readiness/effect analysis, but do not
   assume it covers reentrant access or returning borrowed global fields.
3. Emit one owner per module with ordinary native references. Preserve single
   initialization, source order, failure cleanup and checked function access.
4. Cover module-level schemas, scalar live bindings, records/arrays, calls during
   initialization, cross-module aliases, diamond imports, shadowing, recursion,
   callback escape, mutation conflicts and rejected consumption.
5. Complete editor hover, definitions, references/rename, import completion,
   inferred schema members, unsaved dependency updates and workspace diagnostics.
6. Add matched workload/oracle/ABI support, batch focused compiler/native/editor
   checks when integrated, format the whole project, preserve evidence in SQLite,
   checkpoint and refresh installed/configured CLI/LSP consumers.

Then resume S11 diagnostic options and remaining schema requirements. After S11
acceptance, complete the required cleanup before S12. Thread-pool design discussion
is deferred until after async implementation; mandatory global pools and the proposed
batch/state APIs are not approved.

## Focused implementation evidence

The implementation lowers canonical module declarations into an ordinary program
owner before named-capture resolution. Initializer order, lexical shadowing,
transitive access and real reference lifetimes are preserved. Validated JSON data
imports keep their existing dedicated ownership path. Global object initializers
infer structural record fields; that inference is limited to their source ranges.
Editor facts retain declaration identity and checked types across import aliases,
including inferred schema output types and unsaved dependency updates.

The callback audit found and repaired scalar snapshotting: module captures now
retain ordinary shared/exclusive references even for Copy values. Owned callback
contracts reject these borrowed environments; borrowing callbacks use
`Mutable<Function<...>>`. No hidden copy, shared owner or lifetime erasure is used.

Focused checks completed:

- 10 compiler/native module tests, including reusable schemas, direct and indirect
  mutation, aliases/default/diamond imports, recursive calls, callbacks, lexical
  shadowing, readiness, rejected consumption, no-main startup and startup failure.
- 161 project IntelliSense tests, including imported values/schema hover,
  completion, definitions and unsaved changes.
- Existing `closure_arguments` and `callable_captures` integration suites.
- Two benchmark source/ABI contracts and ten native/Node-after-tsc parity cells.
  Each cell uses three warmups and zero timing samples.
- Whole-project formatting, token preservation and idempotence; final pass changed
  zero files. Native module tests also execute formatter-round-tripped fixtures.

SQLite retains successful and failed records under `tsrust-module-state-*`.
`native_module_state` has matched sources, an independent oracle, source hashes,
ABI guards, timing/allocation adapters, report registration and an untimed checker.
The driver retains one scalar owner across invocations. Initialization is outside
measurement on both sides. No performance acceptance is claimed.

This is focused globals acceptance, not S11 or release acceptance. Diagnostic-option
drafts at `8d47c02b` remain unfinished. Installed-consumer refresh follows the source
checkpoint; then resume S11. Full family regressions and measurement remain due at
the S11 milestone. See [module variables](../guides/module-state.md) for usage.
