# Pinned URL parser patch

This is the published `url` 2.5.8 crate from the Rust URL project, licensed under
MIT OR Apache-2.0. The retained manifest, runtime sources, license files and
upstream unit/WPT tests come from that exact crate. Unused development lock,
registry extraction receipts and original pre-normalization manifest are omitted.

Upstream: <https://github.com/servo/rust-url/tree/v2.5.8>.
Issue: <https://github.com/servo/rust-url/issues/1163>.

Published crate archive SHA-256:
`ff67a8a4397373c3ef660812acab3268222035010ab8680ec4215f38ba3d0eed`.
Original `src/parser.rs` SHA-256:
`2dd4210e1773fc5d3fdc15ce506ecff30dbf8eb358a79bbbc2835cc30d513c3a`.

One runtime change restricts `last_slash_can_be_removed`'s reverse slash
search to the path and adds the path offset back to its result. The original
function already rejected every match before the path, so this removes repeated
authority scans without changing which slash can be removed. The Windows drive
letter/root checks remain unchanged. This fixes quadratic repeated `..` parsing
with a long authority; it does not implement another URL parser.

The second runtime change resets the internal host tag to `HostInternal::None`
after successful `set_host(None)` removal. Upstream clears the serialized host
and offsets but leaves the old tag, making `host`, `has_host`, `host_str` and
`domain` disagree with a fresh parse of the same serialized URL. The reset
preserves that invariant without reparsing or adding allocations. Focused unit
coverage retains query/fragment offsets, clears credentials and ports, and checks
domain/IP hosts, non-special/file URLs, repeated removal and host restoration.

The URL facade separately rejects empty-host assignments retaining credentials or
a port before calling the backend, to avoid the invalid offset state reported in
<https://github.com/servo/rust-url/issues/1174>.

Remove this local source patch when a compatible published release contains the
fixes and passes the same upstream, facade, scaling and relocated-export controls.
Dependency admission, measurement and platform acceptance are tracked in the
v0.2.0 checklist and the sole project progress note.
