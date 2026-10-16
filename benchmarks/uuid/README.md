# UUID round-trip workload

Run `TSRUST=/path/to/tsrust node benchmarks/uuid/run.mjs --verify`.
Both paths validate UUID text, copy to 16 bytes, set the last byte to 0..255,
construct an independent identifier, format/parse and observe its version and
last byte. The independent checksum is `4 * 256 + 256 * 255 / 2 = 33664`.
Node uses strict hexadecimal validation and built-in Buffer copies. No npm UUID
package is required. Random UUID generation is not timed or compared here.

Normal measurement retains three processes, eight warmups and 31 samples.
Timing remains unmeasured. Fingerprints, mutation guards and temporary cleanup
follow the existing matched-workload runners.
