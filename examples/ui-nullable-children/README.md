# Nullable UI children

Run `tsrust dev` for native. Browser output currently uses the compiler
repository’s `tools/build-ui-web.mjs`; integrated CLI web build/dev remains pending. `src/main.tsr` is the
entry; `src/app/page.tsrx` owns component code. Null and undefined children create
no node or text, preserve sibling order and do not split adjacent text runs.
Components can return `Node | null | undefined`; a present subtree retains its
local scalar state, while an absent result releases local state and callbacks.
Ownership still transfers exactly once. Nullable owning arrays cannot be cloned
or forwarded through borrowed storage.

The example selects branches at construction time. It does not claim dynamic
conditional mounting, keyed lists or reactive scalar state with nullable payloads.
Use a ternary with null/undefined for an absent branch; bare boolean and numeric
children remain unsupported. A present counter remains independently interactive.
The native inspector verifies markup and independent resource-release observations.
