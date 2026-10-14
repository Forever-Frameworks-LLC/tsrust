# Object spread compiler continuations

Current continuation: [compiler foundations through S12–S21](compiler-to-roadmap-continuation.md).
Use that entry for current priorities, remaining work, acceptance and resumption.
Historical checkpoint/stop statements below retain their original context.

Object spread source checking and emission use the shared heap work stacks.
This continues the nesting architecture work after `dbd03e89`. Set and Map
constructors remain the next known synchronous source-checking and emission paths.
The overall compiler nesting requirement and S12–S21 remain open.

## Implementation

`analysis/object_spread.rs` owns the pending source parts, checked arguments and
merged field definitions. The driver schedules spread sources in read mode and
explicit fields in consume mode, preserving contextual typing only for fields
that are not overwritten. Each spread source retains the previous builtin-literal
policy until its child finishes; success and error both restore that policy.
Native capability, cloneability, annotation, field order and diagnostic checks
retain their original contracts. Like the former implementation, a record whose
source checking fails has already transferred its fields out of its source slot.

The overwrite proof now uses an explicit worklist. It keeps known record hints
as authoritative and traverses raw spread sources when no record hint exists.
The former depth-32 false-negative fallback is removed. Deep finite overwrite
proofs therefore follow the same final-field rule as shallow records.

`emit/expression_work.rs` schedules the checked source expressions. Assembly
receives their completed fragments and preserves evaluation order, snapshots,
last-write values and first-appearance key order. Record-parameter field snapshots
have a separate pure helper; existing synchronous snapshot consumers retain their
behavior. Borrow-leaf snapshots use a scheduled read before cloning.

## Focused acceptance

Normal test-worker stacks are used with `RUST_MIN_STACK` unset. Cargo uses the
shared `/private/tmp/tsrust-gate100-editor-target` target, `CARGO_INCREMENTAL=0`
and one build job.

- All 50 shared-driver unit tests pass. After restoring compatibility for another
  snapshot consumer, the three object-spread units pass again. These overlap;
  they are not 53 distinct tests.
- The deep unit parses, lowers, checks, emits and disposes 512 object-spread layers.
  Another unit proves overwritten contextual fields through 64 layers. Failure
  witnesses preserve the leaf diagnostic and restore both initial literal policies.
- All 278 compiler integration tests pass across eight affected targets.
  Native witnesses exercise 40 nested source evaluations exactly once and in order,
  duplicate fields, constructor key order, independent copies and annotated
  parameter snapshots. Two original expected-output strings omitted the existing
  quotes and spaces around string-array keys; only those oracles were repaired.
  The original failed run is retained.
- The editor typing matrix, checker agreement and consistency suites pass, together
  with a new nested-spread completion/hover witness: 11 tests in four targets.
- The formatted example executes its assertions. Whole-project formatting changes
  the new example once; the final pass changes zero files and fingerprints.
- Matched native checking/emission, TypeScript checking, native execution and Node
  execution pass at 32 layers. Both executions return the independent oracle `7`.
  Benchmark support retains two warmups and 11 samples; no timing run is claimed.

## Discovery and remaining work

Geometric discovery completes the entire pipeline at 8, 32, 128, 512, 2,048 and
8,192 layers on normal stacks. At 32,768 layers, analysis finishes but emission
exceeds the 30-second operational budget. No observation crashes. Peak RSS is
approximately 227 MB at 8,192 and 826 MB in the timed-out observation. These are
single operational observations made alongside other checks, not formal scaling
measurements or supported ceilings. Emission scaling needs investigation.

The public parser admission policy remains while other consumers still recurse.
Set/Map constructor recursion, other builtin and contextual entry paths, type and
specialization limits, formatter/alias scaling, diagnostic replay, known lifetime
issues, runtime performance work and S12–S21 remain required. No stack size or
source-depth constant is increased.

Canonical evidence uses `tsrust-nesting-object-spread-work-*` in
`benchmarks/records.sqlite`. Acceptance and pending measurements remain separate.
