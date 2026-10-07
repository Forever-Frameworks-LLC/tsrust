# Contextual callable environments

This continues the captured-call-effects plan after 67b268a. The current body
origin identifies code, and BindingId identifies a source declaration. Neither
identifies a particular activation of a factory. Missing-capture guards correctly
refuse frames after producer slots have been projected away, but returned callback
effects remain unavailable. This plan preserves the existing ownership model.

## Retained failing witness

separate_factory_instances_preserve_captured_callable_contracts creates two
selectors from one factory, capturing one and two respectively. Both named and
arrow factories must preserve their producer slots. Invoking each selector returns
a different callable contract: one requires one argument and two requires two.
The current engine returns an opaque result instead. The independent Node witness
verifies identity, arity and results for these nonmutating captures. It does not
establish mutation, ownership or native Tsrust parity.

## Representation and convergence gate

Use an isolated environment store with stable handles. Represent callable
alternatives as correlated code-origin/environment pairs; independent sets would
fabricate a cross product between bodies and captures. Carry these alternatives
through Values, Binding slots, flat aggregate elements, aliases, parameters,
branches and pending return snapshots. An unknown alternative stays explicit.
Do not embed recursively growing capture vectors directly in request keys.

Allocation addresses must have a finite source-based domain. Distinguish creation
sites and finite producer argument/capture facts. Allocation-key projection must
exclude recursively accumulated environment handles; store handle edges in the
joined environment records instead. Repeated recursive requests must reuse stable
addresses and grow monotone records, rather than allocating fresh handles on each
worklist visit. Preserve separate creation sites and distinct captured callable
contracts. Merged allocations need conservative alias/update treatment. Do not
cap iterations or discard alternatives to force convergence.

Add store-level failing tests for stable allocation, independent sites, distinct
capture contracts, recursive edges, updates, alias joins and unknown alternatives.
Prove repeated solver requests stabilize without changing existing caller identity
or declaration signature rules before connecting the store to source validation.

## Creation, entry and projection gate

At callback creation/escape, derive capture behavior from the existing compiler
ownership and capture rules. Borrow local captures and transfer owned escaping
captures at the point established by the current lowering; do not invent shared
JavaScript cells or implicit cloning. Inspect current capture planning and native
lowering before deciding the snapshot point for mutable captures.

Preserve live borrowed bindings while their activation exists. A returned owned
callback must retain its required environment after producer-frame projection.
Entry resolves the correlated environment and installs its exact captured slots;
ordinary same-spelled caller slots cannot replace them. Keep the existing
missing-capture guard. Project callback writes to the correct environment and to
live caller aliases where borrowing rules permit it. Strong updates require an
unambiguous owned instance; joined instances require conservative updates.

Verify factory instances created at different sites and repeated sites, saved
aliases, argument passing, array storage, sibling callbacks, recursive groups,
rest/default parameters and transitive returned callbacks. Preserve snapshots
before finally, overriding returns and throwing cleanup. Unknown or disposed
environments must not become effect-free known calls.

## Ownership and native gate

Pair source effect witnesses with actual compiler/native fixtures. Verify original
owners drop exactly once on normal completion, callback throw, abandoned factories
and aggregate disposal. Preserve borrowed-capture diagnostics, consumed callback
rejection and readiness checks. Do not change runtime ownership to make the source
summary pass. Include independent Node references only for applicable observable
behavior; keep ownership-specific checks native.

## Acceptance and remaining scope

Repair the retained paired factory red and add compiler diagnostics proving the
wrong returned contract is rejected while the correct instance is accepted.
Complete IntelliSense/guide coverage for observable behavior and any new diagnostics.
Prepare matched workload counterparts, independent oracles, source contracts,
registrations and driver hooks without claiming measurements. Use bounded compiler
and native checks, all eight depth canaries after dispatch changes, and the whole
editor crate before acceptance. Store immutable evidence in SQLite and commit
explicit paths after whole-project formatting.

This environment work does not replace the remaining aggregate mutation, spread,
collection callback, opaque-region and native common-carrier requirements. Gate173
and feature-family acceptance remain open until those requirements are verified.

## Environment store kernel WIP

generic_call_environment_store.rs now provides stable abstract handles. Allocation
keys contain callback creation site, producer call site and explicit source leaf
capture facts; correlated environment edges are absent from keys. Capture slots
are ordered by BindingId, duplicate or missing IDs are rejected, and duplicate
source targets do not grow allocation keys. Context records retain binding state,
correlated body/environment alternatives and a change version. Version counters
are notification metadata, not allocation or request identity.

Weak state and edge joins are monotone. Cyclic references reuse existing handles.
Unknown alternatives remain explicit, and joins do not fabricate cross-product
pairs. Malformed capture shapes, missing handle references and unknown edge slots
are rejected before record mutation. Four focused controls verify allocation,
correlation/cycles, canonical slot order and atomic rejection. All 40 other focused
checks pass with the paired factory red explicitly excluded.

The store is not connected to Engine, Values or callback creation yet. It has no
strong-update uniqueness or disposal policy. Next wire correlated alternatives
through source values and bindings, retain producer call-site context in requests,
and add environment-read dependencies so changed records schedule readers. Then
implement creation/entry/projection using existing capture ownership rules. Do not
claim recursive solver convergence from the store self-cycle test alone, and do
not include record versions or recursively growing edge graphs in context keys.

## Environment reader scheduling WIP

The environment store now records request-reader subscriptions separately from
allocation identity. A changed state or edge record queues each subscribed reader
once until the queue is drained. Unchanged and malformed merges queue nothing;
subscriptions to invalid handles are rejected. Reader notification does not add
versions or reader IDs to allocation keys. A failing stub witness preceded the
implementation, and the focused baseline passes 41 tests with the retained paired
factory red explicitly excluded. This kernel remains unused by Engine. Next connect
record reads to request subscriptions and drain changed readers into Engine's work
queue while wiring correlated value transport. No native or editor acceptance is
claimed for this isolated store change.

## Correlated value transport WIP

Values, bindings and flat aggregate elements now carry optional correlated callable
alternatives. An absent metadata field retains the existing body-origin path; when
joined with explicit alternatives, that path is converted to body-without-environment
or explicit unknown alternatives. Joins keep each body/environment pair together
and canonicalize the union, including two environments for the same body. No
independent environment/body cross product is constructed.

Transport covers source snapshots, assignments, declaration commits, formal entry,
rest elements, frame projection, assertions and selected array elements. Pending
return snapshots already carry Values and therefore retain these pairs. Opaque
fallback resets do not retain stale environment metadata. A failing alias witness
preceded the implementation; focused controls cover assertions, array storage,
indexing, unknown joins and formal entry. The focused baseline passes 42 tests with
the retained paired factory red excluded. Whole-project formatting changes no files
or benchmark fingerprints.

Engine still dispatches by origins and does not create or consume these environment
pairs. The new metadata is preparation, not factory behavior acceptance. Next add
producer call-site context, creation hooks, capture ownership-aware environment
initialization and frame entry, and connect reader subscriptions to scheduling.
Keep the missing-capture guard and the paired factory red intact. Native ownership,
disposal, mutable instance updates, recursion convergence and feature acceptance
remain pending.

## Read-only factory environment entry WIP

Engine now owns the environment store and includes producer call-site context and
an optional correlated environment handle in contextual requests. Expression
snapshots allocate environments for available nonmutable capture slots. Entry
subscribes the request to record changes, restores the exact captured identities
and retains the existing missing-capture guard. Changed records queue dependent
readers. Invocation and forEach consume correlated body/environment alternatives;
projected results remove producer slots from the original caller state.

The retained paired named/arrow factory test now passes without exclusion. Repeated
requests reuse the same request population. A compiler integration witness verifies
that the second instance rejects wrong<T>(value) with an arity diagnostic and
related declaration, while wrong<T>(value, value) passes. No source signature or
runtime ownership rule was changed.

This is read-only capture support, not complete factory acceptance. Mutable capture
slots still follow the previous live-caller path. Environment-backed requests
refuse contextual proof if captured slots change on any exit, or if live caller
slots conflict with the retained snapshot. Returned environment snapshots are also
checked against producer exit state; a new failing control exposed parameter
reassignment after callback creation and is now guarded. These conservative guards
must be replaced by verified ownership-aware local borrow and owned-instance
updates, not removed merely to make mutable examples pass.

The focused baseline passes 44 tests with no factory exclusion. All eight depth
canaries and affected compiler flow suites pass, excluding the independently
retained native common-carrier red. The whole editor crate passes. Exact logs are
stored in SQLite. Whole-project formatting changes no files or fingerprints.
Native factory ownership/disposal fixtures, mutable updates, recursive environment
solver convergence, transitive environments and matched workload acceptance remain
pending. Gate173 and the larger captured-effects plan remain open.

## Native polymorphic argument red

A new executable std:testing fixture, callable-environments/retained.tsr, exposes
an additional native prerequisite: factory(one) fails generic-inference for A.
The passing source effect test does not establish native transport of polymorphic
arguments. The integration test is retained failing and unskipped. Read
polymorphic-callable-transport-plan.md before lifting mutable environment guards;
it specifies checked argument identity, capture timing, point-sensitive instance
state, ownership/disposal controls and acceptance requirements. No compiler source
changed for this red checkpoint.
