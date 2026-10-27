Current continuation: [captured rest checkpoint](handoff-2026-10-06-gate173-rest-values.md).

# Handoff: captured optional/default function values

Read AGENTS.md, docs/development-workflow.md, docs/plans/nested-function-declarations-plan.md
and milestones/173-nested-function-signatures.md first. Inspect Git for this checkpoint
hash. Gate172 is accepted at d74f34d; named result inference landed at dd43e11.
Gate173 and the full goal remain incomplete.

Preserve four unrelated uncommitted fixtures under crates/tsrust-compiler/tests/fixtures:
comma-contexts/guards.tsr, interfaces/shapes.tsr, map-for-each/values.tsr and
map-group-by/documented.tsr. One agent in this tree. Explicit Git paths only.

## Implemented behavior

Captured declarations with optional or default parameters can now be values.
local_function_values::Header retains initializer metadata. A generated wrapper
converts a default parameter into an optional supplied slot and forwards undefined;
the original declared function evaluates its default. Do not clone a default into
the wrapper body: that duplicates evaluation semantics and can transfer an owned
capture twice. A supplied argument skips the default, including side effects and
failure. Defaults may refer to earlier parameters and captured variables.

Borrowed default inputs can produce reusable values. A consuming default capture
produces once invocation under existing ownership rules. No implicit clone or
shared cells are introduced. Borrowed closure loans still end at their established
scope boundary; the evaluation fixture scopes its action before reading the mutable
default counter. The original draft failed on that legitimate loan rule; the evidence
is retained. An early assertion incorrectly assumed every owned-string default
must consume; the corrected control returns the string to prove actual consumption.

The new five-test captured_function_defaults suite checks omission, explicit
undefined, supplied arguments, exact default side effects, optional fallback,
dependent defaults, borrowed versus consuming captures, array callback index
arguments, failures and invalid argument types/arity. A stale default-value rejection
in nested_function_captures is removed and replaced with positive native array
assertions. Generic values remain unsupported; their original negative witness is
preserved. The diagnostic now names generic/rest signatures as the pending forms.

A throwing-default fixture skips failure for a supplied argument, then checks its
exact string failure for omission. Checked addition also permits a runtime-error
alternative, so its catch narrows string and rethrows every other error. This is an
assertion-context correction, not a weakened payload check.

## Editor projection repair

The generated wrapper call has the same span as the source function-value name.
Its scalar result could overwrite the callable type and remove signature help.
projections::checked_type_facts identifies wrapper closure spans whose sole return
call and callee share the closure span. It excludes only the hidden call result at
those spans. Do not exclude all same-span generated calls: numeric coercion,
object-spread and in-operator facts depend on other generated calls. The original
broad filter failed three editor witnesses; the narrowed filter passes the complete
editor crate, including those witnesses and the new optional/default signature-help
case. No editor implementation code changed; the source facts were wrong.

## Matched workload and remaining scope

nested_function_defaults is appended at END of manifest. Its eight observations
cover captured default calls, supplied/undefined/omitted arguments, exact default
counter updates, optional fallback, a consuming string default and input length.
Matched TypeScript, independent oracle, source mutation guards, registration, native
ABI and oracle/run/profile/report hooks are complete and verified untimed. Its full
implementationAccepted flag stays false until gate173 acceptance. Existing result
registration now finds its own row rather than demanding it stay last forever.

Next implement generic function values and captured rest-signature values, retaining
source identity, parameter access, arity and ownership. local_function_values::header
still rejects type parameters and rest signatures. Add real red tests first; do not
merely remove diagnostics or make users write arrows. Review contextual specialization,
polymorphic value uses, callbacks and retained captured environments together.

Then audit inferred callable factories, parameter patterns, abstract callbacks,
imported nominal types, catch/finally scopes, deep nesting and fixed-array caller
storage. Shared inferred returns currently require uniform types. Full type-alternative
inference needs direct requirements and native evidence before a parity claim.
Named function expressions remain a separate feature. Preserve all gate172 exact
ownership/disposal and dead-inner behavior. Complete full gate173 and the plan audit
before meaningful family regressions and timing; gate numbers alone do not trigger it.

## Checks and environment

The default suite has five verified tests (a four-test batch and one factory supplement). Earlier focused capture/default/owned checks
pass 31 (default3, capture17, owned11), supplemented by the final four-test default batch and an independent returned-factory
witness. All eight canaries pass 63. Complete editor passes 280, including 149
intelligence tests. Two source registrations pass. The independent Node evaluation
fixture passes. Five matched native and Node-after-tsc inputs pass untimed.

Use CARGO_TARGET_DIR=/private/tmp/tsrust-gate100-editor-target and CARGO_INCREMENTAL=0;
offline locked Cargo. Node is /Users/seancotter/.nvm/versions/node/v24.11.1/bin/node.
ReadOnly<T> is current borrowing syntax. Refresh five consumers atomically via
.refresh-tmp then rename. Preserve library caches and current consumers while pruning
obsolete executable test/native scratch at handoff. Tests may relink after pruning.
No archives, source snapshots, binary backups, routine full compiler matrix or timing.
SQLite is canonical: import original logs/results, verify exact readback, then remove
loose copies. Preserve failed attempts too. Hourly checks remain paused.

A returned-value factory witness also passes: two independently owned defaulted
callbacks preserve separate scalar state, accepting omission, supplied values and
undefined through the explicit optional callable result contract. Inferred callable
factory results still require the remaining gate173 audit.

## Durable checkpoint evidence

Canonical final untimed record: benchmarks/runtime/results/2026-10-06T21-02-40-677Z-verify.json
in benchmarks/records.sqlite. All 113 compiler source fingerprints, all oracle inputs
and matched source hashes match this checkpoint. It records five correctness inputs
and no timing samples. The earlier passed record is retained separately; its source
fingerprint predates the narrowed projection repair.

All 26 original logs/results and 3 actual scratch test inputs
were imported and verified byte-for-byte before removing loose copies. This includes
original unsupported/duplicate-transfer witnesses, loan and assertion drafts, the
broad projection regressions and their repaired checks. The original scratch inputs
are gate-source records captured-default-value.tsr, captured-owned-default.tsr and
captured-default-reference.ts. They are inputs used by checks, not repository snapshots.

Pruned 51 obsolete executable test outputs and native/example scratch.
Five refreshed consumers and library caches remain; tests will relink.
