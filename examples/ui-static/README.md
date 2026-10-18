# Paired static component

`screen.tsrx` is shared by the web and native entry points. It creates checked,
owned `std:ui` nodes through JSX. The web entry prints a complete semantic HTML
page; the native entry opens a GPU window until it closes.

```sh
tsrust run examples/ui-static/web.tsr > /tmp/tsrust-ui.html
tsrust run examples/ui-static/native.tsr
```

The static slice supports View, Text and disabled Button, with the utility
vocabulary documented in `crates/tsrust-ui/README.md`. It has no interactive state,
input handlers, hydration or navigation. Unknown/conflicting utilities throw a
readable error. Nodes cannot be cloned or sent to workers. View consumes children;
HTML generation borrows its tree; native run consumes the tree and releases it
with the window.

For a bounded graphical acceptance run, export the native source package and
build it with `--features ui-acceptance`. That explicit acceptance feature replaces
window waiting with one-frame presentation and normal event-loop shutdown. It is
not enabled by ordinary CLI builds and does not prove interactive behavior.
