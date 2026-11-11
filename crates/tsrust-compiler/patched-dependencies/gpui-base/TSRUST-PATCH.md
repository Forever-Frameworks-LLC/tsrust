# GPUI base 0.9.0 resize release patch

Source: the published gpui-base 0.9.0 crate, SHA256
`cff6377e30f0c857681662e7edc7286bf30f5606033390401b0706d7bd930833`.
The Apache-2.0 license and upstream source/provenance are retained.

`src/resizable/panel.rs` applies the final mouse-up position before finishing a
resize. A single displaced mouse move can be consumed by drag activation, so
waiting for another move loses fast/coalesced gestures. The existing resize
algorithm still owns bounds, fixed panels and redistribution; completion emits
one event. The resize worker returns on an unchanged requested size before
synchronizing painted sizes, preserving a move made before the next paint.
A focused test in `src/resizable/mod.rs` covers one move and a release at a newer
position; the existing same-frame/repeated-release test covers unchanged releases.
The normalized Cargo manifest adds a standalone workspace
only so this managed dependency can run its existing tests independently.

Use the compiler-owned patch payload and unchanged Cargo registry sources.
Remove this patch when an admitted upstream release includes equivalent release
handling and the one-move/native editor gestures pass against that release.
