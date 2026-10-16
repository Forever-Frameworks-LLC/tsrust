# Explicit Rust data adapters

Run `tsrust run examples/rust-data-boundaries/checked.tsr`.

These application wrappers demonstrate the existing restricted Rust bridge. They
are not additional built-ins or implicit conversions:

- `signed` and `unsigned` check 8/16/32/64-bit bounds, returning widened `int`.
  The latter cannot return unsigned values above i64::MAX as `int`.
- `float32` rejects nonfinite, overflow, rounding and underflow; its source result
  is still binary64. Negative zero survives the exact conversion.
- `incrementWide` performs real checked Rust u64/u128-range arithmetic. Source
  inputs/results remain decimal text, never `number`; 39-digit u128 cannot fit the
  bounded 38-digit Decimal type. Text accepts ASCII digits only, up to 39 bytes;
  output is canonical. No source u128 arithmetic or automatic numeric coercion
  is implied. Full signed i128 arithmetic remains unsupported at this boundary.
- `bytes` consumes an int array, checks each element, constructs Vec<u8>, and
  allocates the returned int array. The caller explicitly copies Bytes to/from
  that array. This is not a zero-copy byte bridge.
- `incrementOptional` maps a concrete public enum to Option<i64>, preserving
  absence and checked overflow. Source null/undefined need explicit application
  mapping to that enum. Direct Option and tuple signatures remain rejected.
- `pair` moves an owned record through a Rust tuple without cloning its String.

Failures are Result errors. No unchecked truncation, hidden ownership copy or
relaxation of external Drop rejection is used. Domain Date/Decimal/UUID objects
are not directly projected into arbitrary Rust crates; use their validated
serialization APIs and explicit application adapters. Bundled backend crate names
are reserved: binding packages cannot redeclare BigDecimal, Chrono, chrono-tz, UUID
or getrandom with conflicting pins/features. Direct access through such a binding
package needs separately accepted dependency-alias support.

This is boundary correctness acceptance, not a new performance workload or a
claim that copied byte adapters are as fast as the existing i64 buffer bridge.
The accepted Rust buffer/worker benchmark protocols remain unchanged.
