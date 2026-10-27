# Callable payload implementation progress

Read AGENTS.md, workflow and docs/handoffs/handoff-2026-10-06-callable-payload-red.md. HEAD stays cc4b487. Compiler changes are uncommitted and unaccepted; preserve four owner fixtures.

The retained callable-payload witness now passes native execution. All 38 adapter tests and bounded generic/default/value controls pass on the implementation before the final two audit refinements below. Exact failures, temporary trace and passing logs are stored in SQLite. Trace instrumentation was removed. No process is live.

Types now accepts an optional complete checked callable annotation map. Data-only users explicitly supply None. Checked annotations are supplied in value adaptation, contextual generic values, traversal staging and normal generic preparation. Constraint matching compares full annotation keys when callback/result identities differ, preserving ownership/access/omission/rest rather than shape-only compatibility. Cache source keys accept recursive callable annotations and exclude parameter names/spans. Function specialization rebuilds callable parameter/result source views after substituting generic T. Generic argument validation uses the callable resolver. Synthetic adapter parameters similarly rebuild callback views.

Data-only seed_functions cannot allocate checked callable identities. Callable applications retain full source-key instances and skip this bootstrap; checked requests publish their Type cache identities when contracts resolve. Retain and audit cache controls for capture identity and repeated signatures before acceptance.

Latest unverified refinements: full-key comparison now runs only for different IDs present in the callable annotation map; checked concrete closure reconstruction now preserves borrowed Read/Reusable/Once instead of claiming Owned. Add focused borrowed reconstruction witnesses and confirm these refinements. Audit key canonicalization for Default versus Owned, optional versus defaulted positions, and explicit/inferred access normalization. Do not assume first passing payload proves all metadata variants.

Next add key/ownership/cache/disposal/failure controls, editor protocol tests and matched scenario support. Run bounded compiler/depth and whole editor checks, refresh consumers, records and explicit commit. Five consumers and 93-observation workload stay at accepted 6f6885a. No timing or broad regression was run. Full Gate173 remains open.

Continuation prompt: Continue Tsrust from docs/handoffs/handoff-2026-10-06-callable-payload-progress.md. Finish ownership-preserving generic callable payload implementation and contract key audit. Preserve owner fixtures, original red witness and source/capture cache identities. Complete editor, matched support, bounded checks and SQLite acceptance evidence.

## Key and borrowed reconstruction controls

A retained failing key control exposed bare Default versus explicit Owned and Optional versus Defaulted producing different callable source keys. Keys now use resolved CallbackKind, Mode and omittable-position flags rather than raw enum ordinals; bare/owned and optional/defaulted contracts normalize while OwnedOnce and required positions remain distinct. Defaults still belong to the original declaration and are not copied into adapters. Parameter spelling and spans do not affect the key. The control passes.

A checked concrete borrowed mutable closure reconstruction control passes: its annotation remains Reusable rather than Owned. A new two-signature factory fixture passes native execution and strict TypeScript/Node output 1,2, retaining one counter across int and string callback payloads. The initial string arrow borrowed its argument while requesting a consuming contract; trace confirmed Read versus Consume. Positive fixture now explicitly states Owned<string>, and the original mismatch is retained as a rejection test. Both focused native controls pass. Temporary trace was removed.

All original failures, normalization, reconstruction, positive/negative controls and Node reference are stored exactly in SQLite. Generated Node output is removed and all processes are terminal. Current adapter suite contains 40 tests, but the full suite has not rerun after these changes. Ownership/disposal/failure and metadata/cache controls, editor, matched coverage, depth/whole-editor checks and acceptance records remain required. HEAD cc4b487 and consumers remain unchanged; implementation is uncommitted and unaccepted.

## Runtime ownership and editor controls

All 45 adapter tests pass on current source. New generated-native O0/O3 disposal control observes exactly four original captured owners and exact identity mask 30 across success and factory failure. Readonly and mutable callable payloads retain the caller's borrowed environment and permit later calls by the owner. A consuming payload returns its original captured record and remains single-use. Repeated invocation is rejected with existing closure-consumed diagnostic. Its initial test incorrectly expected transferred; corrected assertion preserves the rejection. No compiler weakening was needed.

The focused editor protocol test passes for concrete returned callable signature help, Function<int,[value:int]> hover and original factory navigation. Initial hover assertion expected arrow spelling; existing editor correctly uses Function spelling and the assertion was corrected. The once and readonly strict TypeScript references compile and match native output. Readonly TypeScript reference states T extends () => number because TypeScript requires that bound to call a generic input; runtime behavior matches Tsrust checked specialization. Mutable native control is inline and has no separate Node reference yet.

Exact native ownership, failed assertions, corrected editor/adapter tests and Node references are stored in SQLite. Generated Node output is removed; all processes are terminal. Next finish matched callable payload workload/guards, metadata/cache controls and any missing references, guide/records, depth canaries and whole editor. Consumers and 93-observation family remain at accepted 6f6885a; no refresh or timing was run. All implementation remains uncommitted and unaccepted. Gate173 stays open.

## Checkpoint closure

Focused callable payload support is accepted. This progress history is superseded by
docs/handoffs/handoff-2026-10-06-callable-payload-accepted.md, which records final checks and
remaining family audit.
