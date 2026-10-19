# Conditional UI mounting

Implementation prerequisite for the current Base UI + Nova component library.
Ordinary JSX ternaries reading directly declared component state own lazy branch
factories. Nested conditions and direct component return expressions use the same
contract. Only the selected branch is constructed. An unchanged selection keeps
its local state; removal releases that subtree before constructing its replacement.
Selecting it again constructs fresh state. Ordinary callback ownership remains
checked: a reusable factory cannot move a prebuilt Node from its capture.

Transparent internal slots add no HTML element, Taffy layout item or AccessKit
node. Empty/absent derived text in containers also adds no anonymous layout item;
an explicit Text remains its own node. Native and browser traversal flatten the
same visible children. Semantic validation runs again after replacement, including
Button phrasing, labels, separators, hidden controls and the depth bound.

IDs belong to owned nodes rather than changing preorder positions. Existing
controls keep their IDs when siblings mount or disappear; retired branch nodes
never donate IDs to new nodes. The browser reconciles surviving elements and
removes retired elements before ordering children, preserving input focus,
selection and composition. Four mount-owned capture listeners serve current and
new controls, including nonbubbling synthetic events. Retired controls retain no
framework handler. Native painting/accessibility use those same IDs; retained
Parley editors survive, and removed editors cancel composition before release.

The private native factory checks ordinary owned callback contracts and reuses
prepared concrete child types. Native emission handles callback, stored-callable
and concrete-closure identities. Child conversion retains integer widths and
owned subtrees. The text-run bridge is an internal Rust runtime ABI helper, hidden
from Rust docs and absent from the public Tsrust module. No public factory,
lifetime handle or separate reactive component is added.

`examples/ui-conditional-mounting` exercises nested independent counters, retained
input editing, remount reset, absent text and transparent root output. The browser
observer compares application transitions with an independent Node-after-tsc
oracle, then checks identity, focus, selection, IME, theme, accessibility, retired
events and disposal. A second entry checks lazy-construction errors and an
independent healthy mount. Both are untimed observers.

Focused acceptance passes on 2026-10-19. The 70 completed runtime witnesses plus
one new empty-text layout witness cover 71 distinct cases. Two new compiler
witnesses pass, including native root/project execution and six explicit rejection
observations. Existing computed-text and nullable compiler evidence is reused;
conditional/computed hover and completion pass. Only affected repairs were rerun.

Three fresh offline locked Wasm exports pass real Chromium observers: nested
conditional mounting, lazy-construction failure and the existing editable-control
witness affected by event delegation. Application output agrees with independent
Node-after-tsc oracles. Focus, selection and composition survive sibling changes;
new controls receive theme styles; retired events are ignored. Noncontrol input
events are ignored, and nonbubbling control events still work.

The native package builds offline/locked with ui-acceptance and presents a real
GPU window through three resizes. The initial vendored dependency build exceeded
its 240-second limit before launching the window; its process group was killed.
The same package/target resumed with completed dependencies and passed. The entry
matches the run result and propagates errors, so a failed window is not accepted
merely because a Result was discarded.

Original failures include the AST move repair, preparing concrete native child
types before callback resolution, ordinary Result unwrapping in the private
facade, and exporting the hidden text-run Rust ABI helper across runtime crates.
Two TypeScript observer setup repairs were made without rebuilding the accepted
site. Source hashes, original logs, timeout/resume and untimed observations use
`tsrust-ui-conditional-*` in canonical SQLite, with exact restoration verification.
No full compiler/LSP regression, SQL pass or performance timing was repeated.

This does not complete the component inventory. Keyed lists, generic reactive
props, arbitrary prop-record state reads, persistent routing, remaining Nova
families/blocks and integrated CLI web build/dev remain required work. Precomputed
conditions retain ordinary snapshot behavior. A direct local-state conditional
component result represents a live slot even when no visible node is selected;
it is not an externally observable null snapshot.

Design follows [the UI vision](../../UI_VISION.md), the pinned component inventory
and the [conditional control-flow reference](https://docs.solidjs.com/concepts/control-flow/conditional-rendering).
The public syntax remains an ordinary JSX ternary. Taffy's pinned Display enum
has no contents mode, so native fragments flatten into their actual parent.

Consumer receipt: CLI, installed LSP, editor override and shared-target binaries
are refreshed from accepted source checkpoint `2f610a65`. The standalone counter
at `/Users/seancotter/Desktop/dev/learning/tsrust-counter` now shows computed double
values, threshold-dependent text and a removable second counter. Existing counter
arithmetic, threshold, local button, theme and installation lock are preserved.
Its README identifies the working page and current supported scope. Installed
project check and formatting pass; a real installed-server open document reports
no diagnostics and retains `get(): int` help. Installed conditional/computed editor
witnesses also pass. The live editor restarts with exact settings restoration.
This demo update reuses the accepted native/browser transition evidence above;
no additional full regression, GPU launch or timing is claimed.
