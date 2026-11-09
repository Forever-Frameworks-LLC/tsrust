# Owned shell words

`split` uses the exact-pinned shlex 2.0.1 package. It parses shell quoting without
executing a command. Valid input returns `{kind:'some', value:string[]}`;
invalid quoting returns `{kind:'none'}`. Empty input returns Some with an empty
array. Returned strings move into Tsrust owned values.

Prepare the binding explicitly, then run:

```sh
tsrust bindings resolve examples/rust-owned-words/words.bindings.json --offline
tsrust run examples/rust-owned-words/main.tsr
```

Expected output:

```text
alpha
two words
世界
none
```

To regenerate from an already prepared shlex crate root:

```sh
tsrust bindings generate examples/rust-owned-words/words.bindings.json --crate shlex --source /absolute/path/to/shlex-2.0.1/src/lib.rs --select split,try_join,try_quote
```

Generation reports generic `try_join` and borrowed `try_quote` as unsupported.
It writes `words.api.tsr` type signatures, `words.stubs.rs` editable stub functions and
`words.bindings.generation.json` hashes/report beside the adapter. Existing
customized adapter source is preserved and causes regeneration to stop; edit it
explicitly or choose a new binding source. Stub files are not compiled; fill their concrete owned parameter/result signatures
and replace the `todo!` body before moving them into the adapter. Builds
never regenerate source or resolve packages automatically.

The retained `../rust-regex` example is the intentional compiler-owned regex
adapter witness. Its opaque owner policy does not allow arbitrary foreign
Drop-bearing types through this general owned-data bridge.

Prepared pinned rustdoc JSON can be supplied with `--rustdoc /path/to/shlex.json`.
The source path remains necessary for reachable data declarations. Other JSON
formats fail explicitly; source fallback is selected by omitting this option.
