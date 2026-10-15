# Rust regex bindings

`regex.rs` declares the functions and owned data that Tsrust can use. The library's
own types remain private to Rust. `regex.bindings.json` selects exact Cargo versions
and features; the adjacent lockfile pins the complete dependency closure.

```sh
tsrust bindings resolve examples/rust-regex/regex.bindings.json
tsrust run examples/rust-regex/main.tsr
# Found 314 at byte 6
```

Resolve once after cloning or changing dependencies. This downloads and vendors
Cargo sources into the example's ignored target directory. Ordinary checking and
emission never contact the registry; native builds use the locked vendored sources
offline. Package exports include those sources for relocation and offline builds.

Strings, buffers and record fields move across the boundary. Enum variants become
named records (`SearchResultFound`, `SearchResultMissing`) usable with value-producing
match expressions. Invalid regex patterns return a string error; no unsafe code or
serialization is needed in this binding.
