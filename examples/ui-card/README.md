# Base UI + Nova Card content parts

The seven editable parts are adapted from the pinned MIT-licensed shadcn source.
Install with `tsrust ui add card@3.0.0 --offline`; source and full license are local.
Use `tsrust run examples/ui-card/src/inspect.tsr` for native assertions, or run the
project to see default, small, customized and clipped layouts.

Use semantic heading children where the surrounding content needs a heading.
Descriptions do not automatically label controls. Actions keep source/tab order.
Scoped `--card-spacing` changes all nested parts; a nested Card supplies its own
scope. `radius.xl` customizes extra-large corners and falls back to lg in older
themes. Generic font-sans/serif/mono are supported; named heading-font assets are not.

This is a working portable content port, **not full Nova Card parity**. Image
primitives/first-last image rules and inline-size containment/container queries are
not implemented. Header remains an ordinary grid; narrow intrinsic layouts may
differ from a CSS size-contained header. These missing capabilities remain in the
coverage ledger. No inert selector or renderer-specific Card switch hides them.
