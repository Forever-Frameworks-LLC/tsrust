# Optional explicit generic call progress

Read AGENTS.md, workflow and both generic value plans. Accepted implementation is
2b5166b. Red witnesses are committed at 3b0ff7f. Current parser/preparation changes
are a preparatory checkpoint, not feature acceptance. Full Gate173 remains open.

## Requirements and current evidence

optional-explicit-generic.tsr requires absent optional invocation to skip its
argument effect, then present explicit int invocations to retain one counter. Its
strict TypeScript/Node counterpart passes output 1,2 and assertions. Native first
fails before invocation at false ? make() : undefined with unsupported: Void unions
are not supported. An additional existing-alias witness avoids that fresh result
hint and fails because explicit preparation rejects the nullable closure payload.
Both original logs are retained exactly in SQLite.

Parser explicit_call_expression previously removed OptionalBoundary from a named
callee unconditionally. That removal is gone. A focused syntax control passes:
the optional receiver boundary and explicit call span metadata remain intact.
Generic source cache lookup now unwraps optional receiver syntax while preserving
that syntax. Checked explicit-name validation examines a nullable payload before
rejecting a nongeneric value. Compilation passes. The alias witness now reaches
arity: Expected 0..=0 arguments, found 1, proving selected receiver calls still read
an unresolved empty prototype. Neither native requirement is accepted yet.

## Next implementation

Do not erase ?. or treat empty prototypes as real signatures. Refactor dedicated
deferred generic preparation so a selected callable expression can specialize the
original declaration using its retained capture context and explicit call metadata.
The selected receiver must retain its real ownership roots and selected_access
checks. Reuse normal argument contracts, omission/rest and method cache identity;
do not synthesize a new owner or rebuild the factory. Receiver evaluation occurs
once before arguments, and absent branches evaluate no arguments. Native method
emission must use the original selected environment. Keep preparation outside
shared recursive dispatch frames. Existing deferred_generic_call currently couples
specialization to a named binding via Self::access; split that responsibility rather
than fabricating a binding with no connection to the nullable owner's flow state.
checked_callable_expression_call currently reads closure.parameters and result
before deferred preparation, explaining the empty-prototype arity failure.

Fresh nullable factory selection needs result preparation before conditional hints
join void with undefined. conditional_plan currently gathers expression_type_hint
before the factory's omitted result has been checked. fresh_generic_origin already
prepares named/generic results using type hints without runtime evaluation. Put any
needed nullable-factory hint preparation in a dedicated bounded helper, preserve
branch capture/readiness contexts, and retain real void-union rejection. Do not
claim unknown results are void or execute source arguments during discovery.

Add present and absent aliases, fresh factories, null/undefined alternatives,
argument order/failure, mismatch/arity, borrowed/once restrictions and exact disposal
controls. Complete editor, strict references and matched support. Run all eight
depth canaries and whole editor before acceptance. Field explicit applications have
a separate parser restriction to Array.of and broader generic storage/type-contract
questions; do not silently admit type arguments on monomorphic stored callbacks.
Continue that audit after optional receiver support.

## State and continuation

Exact records use /private/tmp/tsrust-optional-explicit-{red,reference,alias-red,first,
second,parser}.log in benchmarks/records.sqlite. Original compile errors and
intermediate arity failure remain preserved. Loose logs and generated Node output
are removed; SQLite quick_check passes. No process is live. Five executable consumers
still represent accepted 2b5166b; warm debug test artifacts reflect unaccepted work.
No depth/editor/matched acceptance or timing is claimed for this increment.

Four owner fixtures remain uncommitted and untouched. Use explicit Git paths;
never stage comma-contexts/guards.tsr, interfaces/shapes.tsr, map-for-each/values.tsr
or map-group-by/documented.tsr.

Continue Tsrust from docs/handoff-2026-10-06-optional-explicit-progress.md. Finish
receiver-aware deferred generic preparation and fresh nullable factory result hints.
Preserve optional short-circuit effects, capture/owner identity and original red
witnesses. Complete focused ownership/native/Node/editor/matched checks, depth
canaries, whole editor and SQLite evidence before accepting this feature.
