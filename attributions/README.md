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

## Supplemental Android, JNI and wasm notices

These published archives also omit a license file. Each supplement is the exact
upstream MIT text at the commit recorded in the archive's `.cargo_vcs_info.json`.
For the dual `MIT OR Apache-2.0` crates it is the MIT alternative. The crates
compile only for Android (`jni`, `jni-macros`, `jni-sys-macros`,
`rustls-platform-verifier-android`) or `wasm32-unknown-unknown` (`rsqlite-vfs`),
but Cargo vendors them with the backend lock.

| Package | Upstream file at commit | Archive SHA256 | Supplement | Supplement SHA256 |
|---|---|---|---|---|
| `jni 0.22.4` | [jni-rs `LICENSE-MIT` @5ae9458a](https://github.com/jni-rs/jni-rs/blob/5ae9458a4ec44c5318f37ddc7569c1d4ae8a69e7/LICENSE-MIT) | `5efd9a482cf3a427f00d6b35f14332adc7902ce91efb778580e180ff90fa3498` | `jni-0.22.4-MIT.txt` | `fea1d5bf3dd71605ce5d7d2ff695c1837e914c77195e523a86b5391716477960` |
| `jni-macros 0.22.4` | [jni-rs `LICENSE-MIT` @33045a12](https://github.com/jni-rs/jni-rs/blob/33045a124105c939d1e2cbdcb5a39e5d868ffa03/LICENSE-MIT), identical bytes | `a00109accc170f0bdb141fed3e393c565b6f5e072365c3bd58f5b062591560a3` | `jni-0.22.4-MIT.txt` | `fea1d5bf3dd71605ce5d7d2ff695c1837e914c77195e523a86b5391716477960` |
| `jni-sys-macros 0.4.1` | [jni-sys `LICENSE-MIT` @64d77b7a](https://github.com/jni-rs/jni-sys/blob/64d77b7a5f119d7b55b4e2c169a4668067ff59e6/LICENSE-MIT) | `38c0b942f458fe50cdac086d2f946512305e5631e720728f2a61aabcd47a6264` | `jni-sys-macros-0.4.1-MIT.txt` | `1d85bd754b04ceec93e98e890edd1fa3c6a22e81bcb32135806beeccefa51cd1` |
| `rustls-platform-verifier-android 0.2.0` | [rustls-platform-verifier `LICENSE-MIT` @252e2516](https://github.com/rustls/rustls-platform-verifier/blob/252e25161a91af476cbea620e29d277592d27ec7/LICENSE-MIT) | `eec689c0bc40ff2458a5977b6619cb718087084a18e02a131c599b62d05e1a5f` | `rustls-platform-verifier-android-0.2.0-MIT.txt` | `1c7cf76689c837a68ed8d704994e52a0f2940c087958f860d17f3186afbdcc0c` |
| `rsqlite-vfs 0.1.1` | [sqlite-wasm-rs `LICENSE` @1bb30978](https://github.com/Spxg/sqlite-wasm-rs/blob/1bb309784f25c401ddf95131dcde4c4cdad7aee7/LICENSE) (workspace `license = "MIT"`) | `c51c9ae4df8a7fba42103df5c621fa3c37eccf3a3c650879e90fc48b11cc192c` | `rsqlite-vfs-0.1.1-MIT.txt` | `e1c1975c3474cce31722836f101a40ffaf1cc784e4d2f38ce3e77b9712934e78` |

Packaging and portable export write each supplement to
`attributions/dependencies/NAME-VERSION/LICENSE` only for that exact archive
checksum. Every other crate in the backend and native closures ships its own
license file.
