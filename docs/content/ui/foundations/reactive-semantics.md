<!-- tsrust-docs
{"id":"ui-reactive-semantics","group":"foundations","order":155,"status":"partial","targets":["web","native"],"examples":["reactive-semantics"]}
-->
# Reactive semantics

Accessible names, descriptions and validation state can change without replacing
a control. The same props update browser attributes and native accessibility.
Typing, selection, focus and component state remain attached to the original node.

The example uses editable Button version 3. `tsrust ui add button --offline`
preserves an existing button file and installs the new source as `button-v3.tsrx`;
existing application imports stay unchanged. Compiler API 8 is required.

## Live properties

`aria-label`, `aria-describedby`, `aria-hidden` and `aria-invalid`
accept constants or `ReactiveValue` expressions. Expressions reading directly
declared component state become retained getters. Forward these props unchanged
through local components and replacement renderers. Reading a prop into a plain
precomputed scalar still creates a snapshot; use `useProp` when transforming it.

Undefined removes explicit names, descriptions and invalid state.
An absent hidden flag defaults to false. Empty names fall back to content where
the element permits it. Descriptions resolve their current space-separated IDs;
missing targets are ignored. Invalid state describes application validation and
does not perform validation or change the field's value.

## SVG defaults

An unnamed SVG defaults to decorative presentation and accessibility exclusion.
Adding a nonempty live label defaults it to a visible image. Removing that label
restores decorative semantics. Explicit role and hidden overrides remain checked:
a visible image needs a name, and presentation cannot be named. Paths keep their
geometry and identity throughout these changes.

## Validation and ownership

Metadata follows the same checked rules on every update. Names on generic text or
presentation fail. Regions need nonblank names. Invalid JSON or metadata does not
replace valid metadata. Getters must not mutate state; their errors propagate and stop a mounted
tree through normal cleanup.

Accessibility-hidden content cannot contain enabled controls. Disabling controls
and hiding their ancestor in one handler is checked against the final refreshed
subtree. Hiding alone never disables a control, removes layout or paints it away.
Use View.hidden for retained content that must also leave layout and interaction.

Each affected node owns one metadata evaluator. Its state captures remain weak;
disposing the subtree releases the evaluator. Constants use the static path.
Identity, tags, explicit roles, slots and size metadata stay construction-time values.
Existing Card wrappers can continue cloning static size metadata; live size
forwarding needs a separate compatible prop contract. Expanded relationships,
event/ref merging and focus scopes remain separate work.
