# tsrust-ui

The retained UI model behind `std:ui`: owned `Node` trees, signals, computed
values, text bindings, branches, controlled inputs, utility classes and themes.

This crate has no renderer. The compiler embeds these sources into generated
programs. A program that calls `run` also gets the GPUI renderer
(`crates/tsrust-compiler/src/stdlib/ui/gpui.rs`), which draws the tree with
GPUI Kit. Its dependencies come from the separate locked set in
`crates/tsrust-compiler/gpui-dependencies`.

The compiler and LSP use the crate with default features disabled, to validate
utility classes and themes at check time.

`cargo test -p tsrust-ui` covers the model: refresh, activation, input,
branches, styles and themes. GPUI behavior is covered by building and driving
`experiments/gpui-kit-admission/tsrust/main.tsrx`. See
[the platform plan](../../docs/plans/gpui-application-platform.md).
