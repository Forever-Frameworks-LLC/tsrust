# Development-loop measurements

Run `python3 benchmarks/dev/baseline.py --compiler /absolute/path/to/tsrust`.
The default protocol takes three sequential samples each of 6-file and 130-file
projects. Each graph has one entry, one shared dependency and 4 or 128 leaf modules.
Every leaf imports the shared dependency; the entry invokes every leaf. Integer
outputs have an independent arithmetic oracle.

Each sample uses a new temporary project and initially empty native cache. It
measures initial cold startup, a new dev session with unchanged warm sources, a
single leaf edit, a shared-dependency edit, and a config output-path edit. The latter
checks watcher/config overhead while allowing native artifact reuse. Latencies run
from process launch or before writing the edit through receipt of validated program
stdout. Verbose build phase logs are retained in the same record. These are native
dev workflow measurements, not native-versus-Node runtime benchmarks.

After exit bookkeeping settles, each sample records three seconds of idle
supervisor CPU time and resident memory through `ps`, plus final cache bytes. CPU
resolution is limited by `ps`; a zero delta does not prove zero work. Resident memory
is the supervisor's current RSS, not peak tree memory or a leak test. Cache size
covers this short edit sequence, not long-session retention acceptance.

The JSON record includes all individual samples, source and compiler hashes, host,
current Git commit and harness hash. Completed samples are emitted even if a later
sample fails; a nonzero exit means incomplete acceptance. Temporary source, cache
and binary files are removed on exit. Import and verify closed records in SQLite.
Keep the same project shapes, compiler options, sample counts and harness protocol
when comparing implementations. Do not confuse identical-source warm startup with
fresh-edit compilation cost. Three samples establish an initial diagnostic baseline;
repeat distributions and more edit/lifecycle cases remain milestone acceptance work.

Run harness unit checks with `python3 benchmarks/dev/test_baseline.py`. CPU/RSS
collection requires local process inspection permission on sandboxed hosts.
