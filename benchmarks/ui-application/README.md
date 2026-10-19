# UI application contract witness

Run `TSRUST=/path/to/compiler node benchmarks/ui-application/verify.mjs`.
The native program renders installed-style local components with project theme
JSON. A separately compiled TypeScript oracle reads only the authored theme and
predicts colors, sizes and large button padding. Fixed semantic assertions cover
counter text and button content. The runner records exact input hashes and removes
its temporary executable and TypeScript output. This is untimed correctness
support, not a throughput or interactive browser measurement.
