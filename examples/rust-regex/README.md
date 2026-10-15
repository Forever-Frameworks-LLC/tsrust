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
after cloning. This downloads and vendors
Cargo sources into the example's ignored target directory. Ordinary checking and
emission never contact the registry; native builds use the locked vendored sources
offline. Package exports include those sources for relocation and offline builds.

Strings, buffers and record fields move across the boundary. Enum variants become
named records (`SearchResultFound`, `SearchResultMissing`) usable with value-producing
match expressions. Invalid regex patterns return a string error; no unsafe code or
serialization is needed in this binding.

See [package commands](../../docs/guides/rust-packages.md) for feature changes, validated updates and previews.
