# Generic materialization compilation workload

Run `python3 benchmarks/compiler/function-dependencies/run.py --generic-records
--output /private/tmp/tsrust-generic-materialization.json` as one command. Add
`--measure` only when collecting the registered two warmups and eleven samples.
Use `--only 32 512` for representative untimed validation.

Native and TypeScript sources contain matching chains of generic declarations
with array fields. Native specialization materializes every reachable nominal
instance; TypeScript has different structural checking contracts. Both entry
points construct an empty root array, whose independent output oracle is `0`.
Fresh-process checks and emission exclude backend compilation and runtime costs.
The driver preserves exact sources, hashes, compiler identity and original results.
This workload is registered but unmeasured until separate SQLite evidence exists.
