# Generic interface inheritance

Scope: explicit type arguments on interface bases, local and imported, including
renamed imports, forward declarations, multiple bases and chained generic bases.
The implementation substitutes structured annotations with the existing generic
substitution engine. It retains argument checks even when a base has no fields;
unknown types must not disappear with an unused generic parameter.

Imported record metadata now retains type parameters and linked private names.
Module linking preserves lexical type parameters during substitution. Derived
records remain nominal. There is no implicit conversion to a base, method dispatch,
cloning or shared ownership. Cycles, duplicate fields and wrong arity reject.
Generic defaults, method declarations and class inheritance are outside this section.

Editor fields come from checked record facts, including project-aware nominal
identities. Existing declaration documentation is preserved when updating field
types. Hover and completion cover both local and imported generic inheritance.
The retained project session compares clean and cached analyses after editing an
imported base's field from `T` to `T[]` and back.

The existing `interfaces` workload now has `Named extends NamedValue<string>`.
Its ABI, five observed cells, independent oracle, denominator and timing protocol
remain unchanged. Source guards and fingerprints track the new declarations. The
untimed runner `benchmarks/runtime/verify-interfaces.mjs` compiles both native and
Node-after-tsc drivers and compares them with the independent oracle. It removes
its temporary native artifacts. This is not a new performance measurement.

Focused evidence is retained in canonical SQLite under `tsrust-generic-inheritance-*`.
Initial test-build failures and the ownership diagnostic expectation repair remain
preserved. Acceptance counts and consumer refresh are recorded after final checks.
No SQL/task lifecycle semantics change; reuse accepted database evidence.

Final targeted results: six compiler interface tests and six project-session tests
pass. Editor interface tests pass 3/3; the InheritedItem agreement row passes;
the generic inherited-field completion row passes all four timing/error modes;
17 editor consistency checks pass. Eight matched native/Node cases and the
registration/source-guard test pass untimed. Complete-source hover passed first;
the real-server agreement check then exposed an incomplete-document recovery path
that omitted field facts. Recovery now retains checked fields preceding the edit,
and the affected editor checks pass. Whole-project formatting passes.
