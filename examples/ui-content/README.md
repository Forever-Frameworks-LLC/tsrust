# Keyboard content and separators

`src/app/page.tsrx` exercises editable Base UI + Nova Kbd/KbdGroup and Separator
source with composed button content, a live scalar child, horizontal/vertical
layout and a nested metadata styling context. `inspect.tsrx` checks initial native
serialization. The visible working application remains the standalone
`../tsrust-counter` project beside the compiler repository.

Kbd displays keyboard text; it does not install shortcuts. Both parts render kbd
containers, as in the pinned source. Children retain source order and checked
ownership. Separator defaults to horizontal, has an accessible nonfocusable
separator role, and accepts no children. Its vertical line stretches in the parent
cross axis; horizontal lines fill the available width. Caller classes override
properties in source order. Colors and spacing come from the configured theme.

The bundled Tsrust props describe the portable subset explicitly. Arbitrary DOM
props, Base UI render callbacks/style callbacks, SVG/icon children and a Tooltip
component are not implemented here. Unsupported props fail compiler checking.
The ancestor tooltip-content styling predicate is implemented and independently
exercised using an ordinary named styling context. No hidden tooltip behavior is
claimed. Field composition, named container queries and other component families
remain pending.
