# Conditional mounting

Run `tsrust dev` for native. Browser output currently uses the repository's bounded
`tools/build-ui-web.mjs` builder; integrated CLI web build/dev is still pending.

The ordinary JSX ternary builds only its selected branch. Nested conditions work
without a public branch factory or an extra component. A selected Counter keeps
its own count while its condition stays unchanged. Hiding or switching it releases
that subtree; selecting it again starts fresh. The neighboring input and Pulse
button retain their identities, focus and editing state. No invisible View changes
the row's spacing or accessibility tree.

Empty or absent derived text in a container also contributes no anonymous layout
item. Explicit Text components retain their own node, including when empty.

Conditions reading directly declared component state refresh after handlers.
Precomputed conditions and arbitrary prop-record state reads remain snapshots.
Factories obey ordinary owned reusable callback rules: moving a prebuilt Node out
of a capture is rejected instead of implicitly cloning it. Scalar conditional text
uses the existing text binding. Keyed lists, general reactive prop forwarding and
persistent routing remain separate work. Dynamic changes that violate a Button's
phrasing content, label ancestry, separator content or aria-hidden rule fail with
normal mount cleanup.
