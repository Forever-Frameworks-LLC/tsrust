<!-- tsrust-docs
{"id":"reference-standard-library-bytes","group":"standard-library","order":20,"status":"partial","targets":["web","native"],"api":[{"receiver":"std:bytes","name":"Bytes"},{"receiver":"std:bytes","name":"ByteError"},{"receiver":"std:bytes","name":"fromArray"},{"receiver":"std:bytes","name":"length"},{"receiver":"std:bytes","name":"at"},{"receiver":"std:bytes","name":"set"},{"receiver":"std:bytes","name":"alloc"},{"receiver":"std:bytes","name":"resize"},{"receiver":"std:bytes","name":"append"},{"receiver":"std:bytes","name":"copy"},{"receiver":"std:bytes","name":"fromUtf8"},{"receiver":"std:bytes","name":"toUtf8"},{"receiver":"std:bytes","name":"ByteView"},{"receiver":"std:bytes","name":"ByteViewMut"},{"receiver":"std:bytes","name":"view"},{"receiver":"std:bytes","name":"viewMut"},{"receiver":"std:bytes","name":"toHex"},{"receiver":"std:bytes","name":"fromHex"},{"receiver":"std:bytes","name":"toBase64"},{"receiver":"std:bytes","name":"fromBase64"},{"receiver":"std:bytes","name":"toBase64Url"},{"receiver":"std:bytes","name":"fromBase64Url"},{"receiver":"std:bytes","name":"toUtf8Lossy"},{"receiver":"std:bytes","name":"fromUtf16"},{"receiver":"std:bytes","name":"toUtf16"},{"receiver":"std:bytes","name":"concat"},{"receiver":"std:bytes","name":"copyInto"},{"receiver":"std:bytes","name":"copyWithin"},{"receiver":"std:bytes","name":"equals"},{"receiver":"std:bytes","name":"compare"},{"receiver":"std:bytes","name":"indexOf"},{"receiver":"std:bytes","name":"readInt"},{"receiver":"std:bytes","name":"writeInt"},{"receiver":"std:bytes","name":"readUint"},{"receiver":"std:bytes","name":"writeUint"},{"receiver":"std:bytes","name":"readFloat32"},{"receiver":"std:bytes","name":"writeFloat32"},{"receiver":"std:bytes","name":"readFloat64"},{"receiver":"std:bytes","name":"writeFloat64"}]}
-->
# Bytes

Store binary data in owned Bytes buffers. Indices and lengths count bytes; elements must be in 0..255. Access, mutation, copying and resizing check their ranges. clone creates independent storage.


## Text and binary encodings

`toHex`/`fromHex` encode lowercase hexadecimal and decode either letter case.
`toBase64`/`fromBase64` use the standard alphabet with canonical padding;
`toBase64Url`/`fromBase64Url` use the URL-safe alphabet without padding. Decoders
reject whitespace, malformed padding, wrong alphabets and nonzero trailing bits.
Hex decoding rejects odd length and non-hexadecimal characters.

`fromUtf16(text, byteOrder)` encodes text into bytes, following `fromUtf8`'s
text-to-bytes naming. `toUtf16(bytes, byteOrder)` decodes bytes into text.
Both require `"little"` or `"big"` explicitly. No BOM is inserted, stripped or
used to guess byte order. Decoding rejects odd byte lengths and unpaired
surrogates; error offsets count bytes. Native strings contain valid Unicode.

`toUtf8Lossy` replaces malformed UTF-8 subsequences with U+FFFD. Existing
`toUtf8` remains strict. Both preserve NUL and BOM characters.

Every codec returns `Result<Bytes, ByteError>` or `Result<string, ByteError>`.
Inputs are borrowed; byte observers accept Bytes and both byte-view types.
Results own independent storage. There is no arbitrary input-size ceiling;
length arithmetic and native storage bounds remain checked.

## Combining, copying and comparing

`concat(left, right)` returns a new independent buffer. `equals(left, right)`
compares length and bytes; `compare(left, right)` returns exactly -1, 0 or 1 in
unsigned lexicographic order. A shorter matching prefix sorts first.
`indexOf(bytes, needle, fromIndex)` searches a literal byte sequence and returns
its first offset or -1. Its explicit start must be in 0..length(bytes), inclusive;
an empty needle returns that start. There is no encoding, normalization or
implicit offset clamping.

`copyInto(target, targetOffset, source)` copies a complete distinct-owner source
into a writable destination without changing its length. The complete source
must fit before any byte is written. Same-owner copying uses
`copyWithin(bytes, targetOffset, start, end)`, which copies [start, end) and
preserves the original bytes when ranges overlap in either direction. Both
support writable views, never insert a temporary allocation and leave every
byte unchanged after a failed range check.

## Exact integers and IEEE packets

`readInt(bytes, offset, width, byteOrder)` and
`writeInt(bytes, offset, value, width, byteOrder)` support widths 1, 2, 4 and 8.
`readUint` and `writeUint` use the same argument order but support only widths
1, 2 and 4. Signed width 8 preserves the complete signed int64 range. Unsigned
width 4 reaches 4294967295 exactly; full unsigned 64-bit values are unsupported.
Writes reject values outside the selected range rather than wrapping or
truncating them.

`readFloat32`/`readFloat64` take bytes, offset and byteOrder;
`writeFloat32`/`writeFloat64` also take value before byteOrder. They accept NaN,
infinities and signed zero. Float32 writes round number to binary32 with ties to
even, and finite overflow can become infinity. NaN payload identity is not a
public guarantee. Reads widen binary32 values exactly to number.

Every endian argument is explicitly `"little"` or `"big"`, including for
one-byte integers. Offsets count bytes and need no alignment. Width, byte order,
whole destination range and integer representability are validated before any
write. Unsupported integer widths return `width`, out-of-range values return
`integer-range`, and invalid or incomplete spans return `range`. Packet reads
and writes return Result values, allocate no buffer and never resize their
receiver. Read operations accept both view types; write operations require
Bytes or ByteViewMut. All arguments are required.
