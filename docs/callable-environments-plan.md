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
