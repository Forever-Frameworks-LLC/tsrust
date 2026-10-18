# Paired interactive counter

Two calls to `Counter` create independent state owners. Returning a component transfers its local signal owners into the returned tree
automatically, through ordinary checked moves. No `ownState` call is needed;
text bindings and handlers receive non-retaining references internally.
Use `count` directly in both JSX and callbacks; no public `handle` or `SignalHandle` exists. Ordinary `Text` accepts mixed literal
and reactive children, such as `<Text>Count: {count}</Text>`; there is no separate
reactive text component. State exposes `get()` and `set(value)`; arithmetic belongs
to the application handler. Checked integer overflow leaves the old value intact.
Reading or writing disposed state throws `UiError`. Components execute once; handlers update only changed bound text. Native
layout and paint are currently rebuilt for a redraw, not incrementally patched.

Run native with `tsrust run examples/ui-counter/main.tsr`. Both buttons use pointer
and keyboard activation; disabled buttons cannot mutate state. Native AccessKit
exposes enabled click/focus actions and a visible focus outline. Input editing,
IME, conditional subscriptions, async handlers and component replacement are not
implemented by this counter slice.

For the owner-selected bounded Wasm backend, install the matching wasm-bindgen CLI
and Rust wasm32-unknown-unknown target, then run:

```sh
TSRUST=/path/to/tsrust WASM_BINDGEN=/path/to/wasm-bindgen \
CARGO_TARGET_DIR=/path/to/shared-target \
node tools/build-ui-web.mjs examples/ui-counter/main.tsr /tmp/new-counter-site
```

Serve that directory over HTTP. Its JavaScript host owns returned mount objects
and frees them on page exit. Call `window.ui.dispose()` before explicitly removing
the screen. No per-click network request is made. Handlers run as checked Rust in
Wasm. The build tool rejects service/file/task runtimes instead of pretending they
work in a browser. It reports artifact bytes and hashes, not a performance result.

Browser callbacks record a readable failure on `mount.error()` and release the
state tree. Event listener wrappers remain until the host frees the mount. This
bounded behavior is explicit; automatic DOM-removal detection and richer error
boundaries are not implemented. Native handler failures close the loop and return
`UiError`. Neither backend silently resumes a failed handler.

Acceptance is recorded in the current compiler handoff; this file documents the
contract and commands, not an assertion that pending checks have passed.

The first component-state form is a direct `const state = signal(initial)` declaration
inside a function returning a UI node. State factories, aliases of the signal
constructor and state creation inside anonymous callbacks are not yet component
ownership forms. State references cannot transfer the component owner. Cloning state and
returning state in place of a UI node are rejected. This is construction-time ownership, not a hook-order
or rerender convention.
