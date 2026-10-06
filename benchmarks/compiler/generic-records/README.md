# Generic record frontend scaling

Run `python3 benchmarks/compiler/generic-records/check.py` to validate deterministic sources with 1, 16, 64 and 128 distinct nominal arguments to `Box<T>`. Each argument has the same scalar field shape but distinct nominal identity. Holder records require materialization even when the entry function is empty. The generator checks native analysis/emission and strict TypeScript checking without timing them.

The manifest registers separate frontend check and emit costs. This is not a runtime throughput benchmark or an equivalent type-system coverage claim. Native code generation and ownership/layout checking differ from TypeScript structural checking. Preserve the generated sources and individual samples in SQLite when the scheduled checkpoint measures them; no measurements are accepted yet.

The runtime manifest separately registers `generic_record_nested_int`, which validates equivalent nested record checksums. Optimizers may erase the intermediate records, so its timing cannot establish allocation counts or frontend specialization speed.
