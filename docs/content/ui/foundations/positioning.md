<!-- tsrust-docs
{"id":"ui-parent-positioning","group":"foundations","order":70,"status":"partial","targets":["web","native"],"examples":["parent-positioning"]}
-->
# Parent positioning

Place a child within its UI parent without adding it to the parent's normal layout flow. The same logical boxes and checked utilities drive browser CSS and native layout.

## Relative and absolute boxes

Every UI layout box establishes a containing block. `relative` keeps a box in normal flow. `absolute` removes it from that flow and positions it within its immediate UI parent. Other children do not reserve space for it.

Give positioned content a surrounding View. Applying offsets directly to the application root is outside this shared parent-positioning contract.

Use positioned View boxes around inline content. Nested Text and inline Link descendants reject box/layout utilities; they share a shaped paragraph rather than creating independent layout boxes.

## Insets and spacing

Set independent sides with `top`, `right`, `bottom` and `left`. `inset` sets all four; `inset-x` sets left/right and `inset-y` sets top/bottom. Later matching classes override each side independently. `auto` clears a side.

Numeric offsets use project spacing. `top-2` uses `spacing.sm`; `right-4` uses `spacing.md`. A leading minus, such as `-left-2`, negates the offset. `px` means one logical pixel. Fractions and `full` resolve against the containing layout dimension and need a definite parent size for predictable placement.

The examples use a 256 by 192 border box, a 2-pixel border and 16-pixel padding with default tokens. Absolute offsets use its padding box. Relative offsets retain the original flow position. Browser and native geometry are checked against an independent box-model oracle, including negative offsets and high-DPI scaling.

## Interaction and limits

Positioning keeps source, keyboard and accessibility order. It does not provide an overlay stack or focus manager. Rounded clipping and scroll containers still determine which controls can be painted, pointed at and revealed by focus.

Supported offsets are the listed theme spacing tokens, fractions, `full` and `auto`. Arbitrary CSS expressions, `static`, fixed/sticky positioning, transforms and z-index remain unsupported. Metadata uses `data-slot`, `data-size` or `data-variant`; values are ASCII letters, digits and hyphens. Use checked data/group selectors rather than arbitrary DOM selectors.

Layout uses normalized native floating-point logical coordinates. Insets do not copy a subtree, introduce a state owner or change its cleanup lifetime.
