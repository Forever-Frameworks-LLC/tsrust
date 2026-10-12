# Standard-library frontend and build selection workloads

`node benchmarks/compiler/stdlib-selection/check.mjs` verifies the retained projects with the current `target/editor/debug/tsrust` and the existing runtime TypeScript installation. It checks compiler analysis, emitted helper selection, native cold/warm stdout and compiled TypeScript stdout. Temporary outputs and the isolated native cache are removed. It records no timings.

`manifest.json` registers phase boundaries and sample requirements for the next scheduled combined checkpoint. The used project calls an aliased native import through a user module; the unused project retains a native import but uses no native binding. Their TypeScript references use Node's explicit POSIX path implementation. Native ownership checking and TypeScript checking provide different guarantees; frontend ratios must describe their actual work. The TypeScript unused reference initializes its module binding, while native emission excludes the unused bridge. These programs provide matching stdout and a feature-selection witness, not matching startup semantics or a runtime speed comparison.

When a checkpoint measures these phases, retain source hashes, compiler/backend identities, exact commands, per-sample durations and cache state in SQLite. Cold native builds require a fresh cache per sample; warm builds require an accepted priming build, which is excluded from samples. Verify output and cache/compiler invocation behavior separately. Keep frontend/startup/build costs separate from the matched basename execution workload in the runtime manifest. No timing or allocation data has been accepted for these fixtures yet.

`TSRUST=/absolute/path/to/verified/tsrust node benchmarks/compiler/stdlib-selection/run.mjs --check-only`
runs an untimed acceptance pass over every registered phase. It verifies native
and compiled TypeScript stdout, emitted helper selection, cold misses, warm hits
and the used-to-unused feature transition. Temporary projects/caches are removed.

`run.mjs --output /absolute/path/to/new-result.json` collects the original eleven
samples per workload and phase. It refuses to overwrite an existing record. Each
native build includes `TSRUST_BUILD_TIMINGS=1` diagnostics in its measured wall
time so cache state is verified for each sample. Cold means a fresh native cache,
not a flushed OS disk cache or downloaded toolchain. Warm priming and executable
output validation are outside measured samples. TypeScript frontend checks use
`tsc --noEmit`; the output-equivalence check uses a separately compiled reference.
The record contains raw durations, commands, binary/source hashes and toolchain
identity. Import and verify completed records in SQLite before removing loose
copies. Runner acceptance is not evidence that the registered phases are measured.
