# Bundled component notices

These notices accompany the editable component sources shipped by the offline UI
catalog. They do not assign ownership of the compiler or renderer to an upstream
component author. New projects receive one file per distinct notice in their own
`attributions/` directory, and component headers point to that shared file.

Keep the applicable notices with distributed copies. Vendored Rust dependencies
retain their separate license files in the exported package's `vendor/` tree.

## Supplemental Valuable notice

`valuable-0.1.1-MIT.txt` is the exact upstream repository-root LICENSE at
[commit 9efc29b6e58cef28f6566a47aa7e142a55fead77](https://github.com/tokio-rs/valuable/blob/9efc29b6e58cef28f6566a47aa7e142a55fead77/LICENSE).
The published `valuable 0.1.1` archive declares MIT but omits that file. Its
`.cargo_vcs_info.json` records the same commit and `path_in_vcs: valuable`; the
commit's `valuable/Cargo.toml` identifies version0.1.1 and MIT. Archive SHA256:
`ba73ea9cf16a25df0c8caa16c51acb937d5712a8429db78a3ee29d5dcacd3a65`. Exact supplemental LICENSE SHA256:
`ed60d479b8fd1f64e9cbc3de449a16a53ac1b3d1b6aeb9bf9d190a8e93061b44` (1065 bytes).

Packaging and portable export add this notice under
`attributions/dependencies/valuable-0.1.1/LICENSE` only for that exact pinned
archive identity. Cargo-vendored files and `.cargo-checksum.json` stay unchanged.
This is an attribution supplement, not a dependency-source patch.
