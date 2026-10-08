# Precise Map entry capture release

## Required behavior

Deleting a known entry releases only its callback captures. Remaining entries,
borrowed lookup results and moved payloads retain their own loans. Replacement
releases the replaced payload after evaluating its key and replacement value.
Dynamic keys and uncertain branch outcomes must retain every potentially live
capture. No implicit clone, shared cell, unsafe cast or object identity is needed.

The first red witness is deleting_one_map_entry_releases_only_its_capture in
scoped_record_composition.rs. It stores callbacks capturing two independent
arrays, deletes first, mutates first, calls second, clears and mutates second.
Source checking currently reports capture-conflict on the released first array.
The paired negative witness must still reject mutation of second. Native O0/O3
Drop observers must show the first closure dropping during delete and the second
during clear. The independent Node observable reference passes.

## Two coordinated changes

Current capture_loan_owners::Place identifies a container and record field path,
not its entries. Map.set attaches every incoming callback to that same owner.
Map.delete has no ownership effect. Separately, scoped_map_carriers joins every
insertion into one native callback environment family for the entire function.
Merely deleting a source loan would allow programs that still fail generated
Rust because the surviving Map type retains the removed capture lifetime.
Implement both layers before accepting the witness.

1. Add an isolated entry-state module. Track canonical stable scalar keys under
   each Map storage path, their actual payload types, and capture owner paths.
   Unknown keys use a conservative bucket. Use exact typed key semantics;
   never use expression spelling as identity. Preserve the current float policy.
   Known replacement stages the RHS before discarding its previous slot.
   Known deletion discards only the selected slot. Unknown writes and deletes
   retain all possible aliases. Moves transfer this metadata, clear removes it,
   branches join reaching states, and loops retain safe fixed-point behavior.
2. Keep point-sensitive storage epochs separate from the existing global
   insertion-family join. Record checked deletion/replacement boundaries and
   the remaining concrete environment types. A surviving owner must acquire a
   native type that no longer mentions a provably removed capture. Reuse
   tsr_owned_items and record carrier conversion to move surviving entries,
   preserving key order and without cloning their payloads. Checked narrowing
   must handle excluded callback variants explicitly; arbitrary narrowing is
   forbidden. Empty survivors reuse existing absent environments.
3. Retarget later constructor, set, get, iteration, move and exception metadata
   to the applicable epoch. Do not change shared nominal declarations. Keep
   error contracts from surviving callbacks and preserve typed failures.

A moving survivor conversion may scan retained entries. Do not quietly turn
ordinary delete into an O(n) operation: emit conversion only where a proven
capture-release boundary requires a new native environment type. Record this
cost in developer evidence and cover it in the matched workload. The family
performance review must assess it before any broad efficiency claim. A later
static-slot representation may remove that cost without changing semantics.

## Acceptance sequence

First establish straight-line known keys, then record-field storage and moves.
Cover deletion of missing and repeated keys, replacement by another origin,
same capture in several entries, dynamic-key aliases and key normalization.
Keep lookup loans live until their scope ends. Follow with branches, loops,
try/catch/finally, fallible callbacks and exact disposal at every release.
Each implementation batch needs its own failing witnesses before source edits.

Add editor diagnostics and verified guide examples. Extend matched Tsrust and
TypeScript workload observations, independent oracle, source contracts, mutation
controls, hashes and existing hooks. Verify untimed native and Node-after-tsc.
Run affected compiler suites and all eight depth canaries after analyzer dispatch
changes. Preserve failed originals and resumed checks in SQLite. Format the whole
project before each source checkpoint. Do not describe B5 as accepted until
remaining array-element release and all named composition requirements are
reconciled and the feature-family acceptance is complete.
