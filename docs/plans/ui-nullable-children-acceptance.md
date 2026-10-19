# Nullable UI children: focused acceptance

Accepted portable prerequisite, 2026-10-19. `Child` and `TextChild` now include null
and undefined. Both are omitted before renderer construction; they neither create
placeholder nodes nor split adjacent text runs. Sibling order remains unchanged.
Native node representation and rendering are unchanged.

Components can return `Node | null | undefined`, including stateful components.
A returned subtree retains its local scalar owners. An absent result leaves owners
in lexical scope, so local callback environments release before those owners.
Returned expressions evaluate once. The private state attachment bridge validates
its actual result and rejects other data; it adds no public lifetime API. Owning
nullable child arrays still reject cloning and borrowed-to-owned forwarding.

Generated match blocks now receive the complete contextual union, like conditional
expressions. Previously the checker supplied only the present payload, rejecting a
valid absent arm. This also fixes ordinary source match expressions returning a
nullable owned node; the native inspector includes that independent witness.
The private attachment operation describes its result as checked from arguments,
rather than publishing a misleading fixed Node result hint.

`examples/ui-nullable-children` covers literal and variable absence, conditional
component results, two independent live counters, sibling/text order and semantic
button content. Its native inspector observes independent tracked-resource release
for present, null and undefined results, including an eagerly constructed local
callback subtree discarded by an absent return. Release occurs exactly once.

Three focused compiler witnesses pass, including ownership/payload rejections.
Nullable type/documentation help, existing scalar help, input-event consistency and
all three affected prop-type agreements pass. Fresh Wasm Chromium observations
match the independent Node-after-tsc oracle for initial content, both counter
updates, exact element order, accessible names and disposal. Prior GPU/rendering
and shared-runtime evidence is reused because this section changes the checked
bridge and state lowering, not either renderer. No full compiler/LSP regression,
SQL pass, new GPU smoke or performance timing was run.

Whole-project formatting verifies token preservation/idempotence, and successful
source witnesses execute after formatting. Original compiler errors, temporary
trace output, repairs, source hashes and untimed observations use
`tsrust-ui-nullable-*` in canonical SQLite. Temporary trace instrumentation was
removed. Consumer refresh and cleanup receipts accompany the source checkpoint.

This is initial-branch composition. Dynamic conditional mounting, derived text,
keyed lists and nullable signal payloads remain required separate work. Bare
boolean/numeric children remain explicit errors. Field and the remaining Nova
components/blocks are not accepted by this prerequisite. Example documentation now
also states the existing CLI gap accurately: browser output uses the repository's
bounded Wasm builder; integrated CLI web build/dev is still pending.
