# Query decoding witness

Run `TSRUST=/path/to/tsrust node benchmarks/http-query/verify.mjs`.
Native and Node-after-tsc outputs must match an independent literal oracle for
valid query encoding, repeated values, absence, bare keys and embedded equals.
This is untimed boundary acceptance. Node URLSearchParams is only a reference for
these valid inputs: its replacement behavior for malformed UTF-8 is not claimed.
Malformed-input preservation and request body ownership have native regression
witnesses. The runner records exact source fingerprints and removes executables.
