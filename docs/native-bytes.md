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
