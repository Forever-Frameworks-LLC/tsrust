# Rust regex bindings

`regex.rs` declares the functions and owned data that Tsrust can use. The library's
own types remain private to Rust. Package commands own `regex.bindings.json`,
including exact versions, features and the complete Cargo lock. The adjacent
lockfile remains as a retained legacy input; managed generations use their embedded lock.

```sh
tsrust add regex@1.12.2 --project examples/rust-regex
tsrust run examples/rust-regex/main.tsr
# Found 314 at byte 6
```

Use package commands to change dependencies; no handwritten JSON is needed. Restore
locked sources with `tsrust bindings resolve examples/rust-regex/regex.bindings.json`
after cloning. This explicitly fetches the exact locked sources into Cargo's shared cache.
Add `--offline` to require already-cached sources. Ordinary checking, emission and
native builds do not download or create vendor copies. Native builds reuse locked
shared sources offline. Portable exports require a separately prepared source pack:

```sh
tsrust bindings resolve examples/rust-regex/regex.bindings.json --portable --offline
tsrust emit examples/rust-regex/main.tsr --package -o target/regex-export
```

The source pack uses Cargo vendor and includes notices and checksums. The exported
package can be relocated and rebuilt offline.

Strings, buffers and record fields move across the boundary. Enum variants become
named records (`SearchResultFound`, `SearchResultMissing`) usable with value-producing
match expressions. Invalid regex patterns return a string error; no unsafe code or
serialization is needed in this binding.

See [package commands](../../website/src/content/docs/tools/rust-packages.md) for feature changes, validated updates and previews.
