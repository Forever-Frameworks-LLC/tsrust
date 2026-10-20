<!-- tsrust-docs
{"id":"component-badge","group":"components","order":90,"status":"partial","targets":["web","native"],"component":"badge","examples":["badge-variants"]}
-->
# Badge

Label a status, category or count. Badges have compact pill geometry and six palettes: default, secondary, destructive, outline, ghost and link.

## Install

Run `tsrust ui add badge --offline`. Import `Badge`, `badgeVariants` and their types from `src/components/ui/badge`. Source stays editable in your project. Compiler API 5 is required for shared text flow and decoration utilities. Installation uses bundled source and puts the attribution in `attributions/`.

## Variants

Omitting `variant` selects `default`. Explicit `null` removes palette classes and `data-variant`, retaining the base geometry. Live variant and class expressions update the existing node. `badgeVariants()` returns the same classes without constructing a component; its optional options object accepts a static `variant` and `className`.

A plain badge is a `span` without an implicit role, keyboard stop or action. The `link` palette changes appearance; use an actual Link when content navigates. Communicate status with words, not color alone. Use an application live region when a changing count needs an announcement.

## Composition

`render` runs once with owned, styled View props. Return a Link or Button and forward the attributes and children your control needs. No wrapper or hidden click handler is added. Link-only hover palettes use the checked `[a]` predicate. Buttons retain native keyboard activation and focus rings.

The examples deliberately put replacement labels and handlers in application code. The component does not implement navigation, counters or dismissal policy. Full Base UI event/ref merging remains under development; handlers are not automatically composed.

## Sizing and customization

The default height is five spacing units, 20 logical pixels with default tokens. Horizontal padding is two units; the type size is three quarters of `typography.base`. `rounded-4xl` uses twice `radius.xl`, falling back to `radius.lg` when xl is absent. Radii scale together to fit the actual box. Caller classes override individual properties.

`whitespace-nowrap` collapses spaces and prevents soft line breaks. For a long label, override height with `h-auto`, add `whitespace-normal` and constrain width. `whitespace-pre` preserves spaces and hard breaks without soft wrapping; `whitespace-pre-wrap` preserves them with wrapping. Stored strings and editable values stay unchanged. Selection follows displayed text.

## Themes and accessibility

Colors come from the enclosing theme: primary, secondary, destructive, muted, foreground, border and ring tokens. Both targets resolve nested light/dark scopes. Test custom foreground/background pairs for contrast, including hover and destructive palettes; changing tokens cannot guarantee contrast automatically.

Keyboard rings use three logical pixels. `aria-invalid` changes the border and ring without inventing validation behavior. No arbitrary ARIA is accepted silently. SVG icons, icon-dependent padding, animated transitions and complete Base UI event/ref composition remain unsupported. This page documents the implemented portable subset.
