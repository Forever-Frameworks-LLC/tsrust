# Owned native bytes

Import `Bytes`, `ByteError`, `fromArray` and `length` from `std:bytes`.
`fromArray(values)` borrows an integer array and copies valid elements into an
independent owned byte buffer. Each integer must be between 0 and 255 inclusive.
An empty array succeeds with an empty buffer. The input remains unchanged and reusable.

```tsrust
import { fromArray, length } from "std:bytes";

function main() {
    match (fromArray([0, 65, 255])) {
        Ok(bytes) => { console.log(length(bytes)); },
        Err(error) => { console.log(error.message); },
    }
}
```

This prints `3`. `length(bytes)` borrows the buffer and returns its byte count as
`int`; it neither copies nor mutates the buffer. Byte counts are not Unicode
character counts.

Construction returns `Result<Bytes, ByteError>`. Invalid elements produce code
`byte-range`, a readable message and the first invalid array offset. Lengths outside
`int` produce `length-range`; failed reservation produces `allocation`. These last
two errors omit offset, which reads as `undefined`. Reservation failure is recoverable,
but recovery from every process-wide memory exhaustion is not guaranteed.

Passing Bytes by value transfers ownership; use `ReadOnly<Bytes>` for borrowing.
Storage is private and record literals cannot construct a buffer. Console inspection,
string conversion, property reflection, object spread and value comparison are
currently rejected. Wider byte operations and resource handles remain in progress.


## Checked byte access and mutation

`at(bytes, index)` borrows the buffer and returns `Result<int, ByteError>`.
Indices are zero-based bytes, from zero through `length(bytes) - 1`; negative indices
never wrap. Out-of-range indices, including every index on an empty buffer, return
`index-range` with the requested index in `offset`. The buffer is unchanged.

`set(bytes, index, value)` borrows the buffer exclusively and returns
`Result<void, ByteError>`. A successful write replaces one byte without changing
length. Values must be integers in 0..255. Bounds are checked first; a bad index
returns `index-range`. With a valid index, a bad byte returns `byte-range` and that
index as offset. Either failure leaves every byte unchanged. A readonly parameter
cannot be passed for writing; passing the owner does not transfer or clone it.

Match a successful void result with `Ok(_)`. For example, after constructing a
buffer with `[65]`, `set(bytes, 0, 90)` succeeds and `at(bytes, 0)` returns `Ok(90)`.
The complete catalog examples print `255` for reading `[65, 255]` at index one and
`90` after writing the single-byte buffer. Both execute with the current compiler.

These operations belong to gate 188. Append, independent range copies and the
exact cleanup matrix remain in progress.


## Allocation and resizing

`alloc(length, fill?)` returns `Result<Bytes, ByteError>` with an independent buffer.
`resize(bytes, length, fill?)` exclusively borrows a buffer and returns
`Result<void, ByteError>`. Both default fill to zero. Length must be nonnegative and
fit native storage; fill must be an integer in 0..255. Length is validated first,
then fill, including empty, unchanged and shrinking requests. Invalid arguments
return `length-range` or `byte-range`; their offset is undefined.

Resizing preserves the existing prefix. Growth fills only new bytes; shrinking
removes the trailing bytes and may retain capacity. Growing again initializes new
bytes rather than restoring discarded contents. Failed reservation returns
`allocation` before changing the buffer's length or contents. This does not promise
recovery from every process-wide memory exhaustion. No arbitrary byte limit is imposed.

```tsrust
import { alloc, resize, length } from "std:bytes";

function main() {
    match (alloc(1)) {
        Ok(bytes) => {
            match (resize(bytes, 3, 65)) {
                Ok(_) => { console.log(length(bytes)); },
                Err(error) => { console.log(error.message); },
            }
        },
        Err(error) => { console.log(error.message); },
    }
}
```

This prints `3`. The original byte stays zero; the two new bytes are 65.
The owner remains usable after resize; readonly byte parameters cannot be resized.


## Appending and independent range copies

`append(bytes, values)` exclusively borrows Bytes and readonly borrows an int array.
It appends the values in order and returns `Result<void, ByteError>`. An empty array
succeeds without reserving storage. The combined length is checked first; then every
value must be in 0..255. Invalid length returns `length-range` without offset; invalid
value returns `byte-range` with its index in the input array. Reservation failure
returns `allocation` without offset. Every failure leaves length and contents intact.
The input array stays unchanged. This is an explicit integer-to-byte copy, not wrapping.

`copy(bytes, start, end)` readonly borrows Bytes and returns `Result<Bytes, ByteError>`
with an independent owner. Bounds must satisfy `0 <= start <= end <= length(bytes)`.
The half-open range excludes end. Equal bounds return an empty buffer, including at
the source end. Negative indices never wrap; bounds are never clamped. Start is checked
before end. Invalid start returns `range` with start as offset; invalid or reversed
end returns `range` with end as offset. Reservation failure returns `allocation`
without offset. Success and failure leave the source unchanged. Mutating either
owner later cannot affect the other. Coordinates count bytes rather than characters.

```tsrust
import { fromArray, copy, length } from "std:bytes";

function main() {
    match (fromArray([0, 65, 255])) {
        Ok(bytes) => {
            match (copy(bytes, 1, 3)) {
                Ok(part) => { console.log(length(part)); },
                Err(error) => { console.log(error.message); },
            }
        },
        Err(error) => { console.log(error.message); },
    }
}
```

This prints `2`. The selected buffer contains 65 and 255. No source owner transfers
or implicit clones occur. As with other byte operations, fallible reservation does
not guarantee recovery from every process-wide memory exhaustion.


## Explicit cloning, storage and cleanup

`clone(bytes)` readonly borrows a buffer and returns an independent Bytes owner.
It uses the ordinary clone operation rather than a separate byte-only spelling.
Changing or resizing either buffer does not change the other. Explicit cloning also
copies owned Bytes descendants in arrays, records, Maps and nullable/Result payloads.
Null and undefined alternatives remain absent. Clone does not return Result and does
not promise recoverable allocation failure. Assignment, parameter passing and storage
transfer owners without implicit cloning.

Bytes can live in generic parameters, records, arrays, Maps with supported scalar
keys, nullable unions and Result payloads. Borrowing parameters use ReadOnly or Mutable.
Escaped callable captures transfer owners; reusable callbacks can readonly inspect
their captured buffers. A Once callback can transfer its captured owner out. Normal
scope exit, early return, replacement, Result failure, throw, finally and discarded
closures release byte owners deterministically. Bytes needs no explicit close.

Bytes and records containing Bytes cannot be collection keys, inspected as console
objects, compared as values, reflected into properties or spread as records. These
forms produce source diagnostics. Copy/clone and explicit byte operations are the
supported ways to observe and transform buffers. Native thread-transfer metadata
describes backend eligibility only; source thread APIs are not yet implemented.


## Explicit UTF-8 encoding and strict decoding

`fromUtf8(text)` borrows a valid UTF-8 string and returns `Result<Bytes, ByteError>`
with independent encoded bytes. No normalization, BOM stripping or replacement occurs.
For example, encoding `é😀` produces six bytes, not two. Empty text produces an empty
buffer. Invalid native length returns `length-range`; failed reservation returns
`allocation`. Both omit offset, and no partial owner escapes.

`toUtf8(bytes)` readonly borrows Bytes and returns `Result<string, ByteError>` with
independent owned text. It rejects malformed, overlong, surrogate, out-of-range and
truncated sequences. The `utf8` error has message `Byte buffer contains invalid UTF-8.`
and the first invalid sequence's starting byte offset. For `[65, 195, 40]`, offset
is 1, the start of the malformed sequence. Valid bytes are checked once before
output reservation and copying. NUL and a leading U+FEFF are preserved. Empty input
produces an empty string; invalid bytes never turn into replacement characters.

Neither operation consumes or changes its input. Both require exactly one correctly
typed argument, with no coercion or defaults. Reservation failure returns `allocation`
without offset; this does not guarantee recovery from every process-wide memory
exhaustion. Encoding/decoding copies are explicit. Scoped zero-copy views are still
pending; UTF-8 support does not imply borrowed-view support or gate 189 acceptance.
