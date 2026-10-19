# Semantic UI primitives

`tsrust run inspect.tsr` checks generated HTML. `tsrust run` opens the same tree
in the native renderer. The bounded Wasm builder accepts `main.tsr` for browser
acceptance. This example covers exact JSX attributes, headings, named regions,
alerts/status, separator orientation, hidden decoration and a disabled button.

A heading's tag supplies its accessible level; classes supply its visual size.
`aria-label` overrides accessible text without changing what is drawn. Invalid
roles/tags fail explicitly. This is primitive acceptance, not the finished Nova
component catalog or an independent screen-reader audit.
