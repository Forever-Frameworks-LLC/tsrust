# Responsive documentation navigation

The navigation repair uses `bf9e9c92` plus the accompanying capture, landing-section
and observer repairs. The actual dev preview passes the focused browser scenario.

Sidebar page styling compares the full normalized route. Section styling compares
the section independently. Ancestor `aria-current=location` semantics remain valid
without painting every ancestor as the selected page. Both theme palettes preserve
the exact selection color.

Below 1024 logical pixels, Browse docs opens the navigation in the reading area;
closing or selecting a different route returns to a full-width article. Desktop
keeps its sidebar and section navigation. The right contents rail appears from
1280 pixels. The compact header retains native drag-region space. Routing and
article state remain owned by the existing application location/Route contracts.

Shared `hidden` display utilities remove retained subtrees from layout, paint,
keyboard/pointer interaction and accessibility. Responsive flex/grid can show the
same subtree without disposing its state or callbacks. Existing inline text rules
continue to reject independent box/display utilities inside a paragraph.

Dev-server identity now accompanies revisioned assets and polling. A restarted
server cannot silently reuse another server's versioned JavaScript/Wasm. Mixed
session/revision assets reject with 409. The reported computed export diagnostic
is absent in the refreshed CLI, installed-LSP protocol and actual browser. The
original console report was not independently reproduced; no export alias or
suppression was added.

Actual browser acceptance found a Wasm trap before first mount. Binary inspection
measured 309632 stack bytes in the app root and 432400 in Landing before nested
calls. Landing now composes named section functions, reducing the active frame
chain without increasing the Wasm stack. The original failure is retained. General
compiler temporary-frame reduction and initial web loading remain open.

Focused checks: two native/model responsive visibility tests; formatted compiler
and installed-LSP computed/visibility witness; the new completion row in settled,
immediate, paced and unrelated-error modes; three dev-server unit tests; current
CLI export/utility check; and actual browser clicks, console/mount checks, exact
selection in both themes, disclosure, route selection, resize restoration and
viewport bounds at 390/768/1023/1024/1440. Phone and desktop screenshots were
inspected. The first color observer was repaired to measure alpha through Canvas,
rather than depending on one browser's transparent-color serialization.

Reuse prior OS-window startup, component behavior, ownership and route-scroll
acceptance. No full compiler/LSP/SQL suite or performance benchmark ran. Build
latency remains material: the accepted normal dev rebuild took 115158 ms; this is
a diagnostic observation, not a matched benchmark measurement. Workspace-wide LSP
scan cost remains open; a current executable and focused protocol checks do not
prove the whole repository scan has completed.

Canonical evidence uses `tsrust-docs-navigation-evidence.json` and its associated
closed records in SQLite. Port 4317 remains the actual dev watcher, not a static
export. The CLI/editor include the shared visibility contract from `bf9e9c92`.
