# Computed text

Run `tsrust dev` for native, or use the compiler repository's bounded Wasm builder
for browser output. `src/main.tsr` is the entry. Arithmetic and formatting stay in
the page or imported application functions; the framework supplies no increment
helper, public handle attachment or separate reactive Text component.

Within JSX text-child contracts, expressions reading directly declared component
state update after handlers. Supported scalar forms include get(), arithmetic,
string templates/operations, scalar conditional branches and top-level/imported helpers with declared
scalar return types. Getter expressions must be pure; they also run during initial
construction and refresh. Null/undefined getter results contribute no characters.
Integers keep all 64 bits; numbers use ordinary formatting, including -0 as 0.
Numeric literal/array children also retain their exact scalar type.

Direct Signal children remain live. Precompute a value outside JSX to request a
snapshot. Unknown/inferred helper returns, generic child contracts and state read
through arbitrary prop records remain snapshots; this section does not implement
reactive prop proxies, dynamic node mounting or keyed lists. Owning captures still
follow normal callback rules, without implicit cloning. Bare boolean literals
remain unsupported; explicit boolean state bindings display true/false.
