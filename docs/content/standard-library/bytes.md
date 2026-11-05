<!-- tsrust-docs
{"id":"reference-standard-library-bytes","group":"standard-library","order":20,"status":"partial","targets":["web","native"],"api":[{"receiver":"std:bytes","name":"Bytes"},{"receiver":"std:bytes","name":"ByteError"},{"receiver":"std:bytes","name":"fromArray"},{"receiver":"std:bytes","name":"length"},{"receiver":"std:bytes","name":"at"},{"receiver":"std:bytes","name":"set"},{"receiver":"std:bytes","name":"alloc"},{"receiver":"std:bytes","name":"resize"},{"receiver":"std:bytes","name":"append"},{"receiver":"std:bytes","name":"copy"},{"receiver":"std:bytes","name":"fromUtf8"},{"receiver":"std:bytes","name":"toUtf8"},{"receiver":"std:bytes","name":"ByteView"},{"receiver":"std:bytes","name":"ByteViewMut"},{"receiver":"std:bytes","name":"view"},{"receiver":"std:bytes","name":"viewMut"},{"receiver":"std:bytes","name":"toHex"},{"receiver":"std:bytes","name":"fromHex"},{"receiver":"std:bytes","name":"toBase64"},{"receiver":"std:bytes","name":"fromBase64"},{"receiver":"std:bytes","name":"toBase64Url"},{"receiver":"std:bytes","name":"fromBase64Url"},{"receiver":"std:bytes","name":"toUtf8Lossy"},{"receiver":"std:bytes","name":"fromUtf16"},{"receiver":"std:bytes","name":"toUtf16"}]}
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
