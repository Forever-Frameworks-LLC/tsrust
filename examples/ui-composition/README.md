# Owned UI composition

`tsrust run` opens composed controls. `tsrust run inspect.tsr` checks generated
HTML and forwarding existing string/Node arrays. The same main entry is accepted
by the bounded Wasm builder. The first button owns its inline content and updates
one retained integer binding. Other buttons demonstrate optional handlers and a
reactive disabled condition without a handler. State remains private to the tree.

View accepts mixed text, signals and Nodes. View as=span is a native layout box
with a span DOM tag; layout classes still apply. Button allows phrasing content
and rejects flow/nested interactive nodes. Numeric/boolean/null children remain unsupported. Computed strings/trees are
construction-time snapshots; direct signals refresh. This is not the
finished Nova component family or complete React child semantics.
