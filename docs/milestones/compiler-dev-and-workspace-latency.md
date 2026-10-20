# Dev compiler and workspace scan latency

The performance repair preserves checked lifetime and ownership rules. Analyzer
contexts share recursive native shape indexes only after exact record, union and
scoped callback-family comparisons. In-place changes invalidate the shared index;
old contexts retain their original immutable shapes.

Rust emission now finds numbered runtime support references in one pass instead
of searching the complete output for every record or union. Prefix collisions,
leading zeroes, missing suffixes and integer overflow retain the previous selection
behavior. For the unchanged documentation input, the emitted 31,098,546 Rust bytes
match the earlier implementation exactly: SHA-256
`a036c0cd5762ccede1175d6eed835c8a9f926839504258bc252f075aff953922`.

Workspace scans check configured application entries first. A successful graph
covers imported files in that same project; unreferenced files, test entries,
nested project entries and failed graphs still receive independent checks.
Completed entries survive interruption. Foreground work interrupts between entries,
and current open-document diagnostics publish without waiting for the full scan.
Dependency observations remain independent; identical source text shares storage.
No cache size limit or reduced checking mode was introduced.

## Observed latency

These are diagnostic observations on the actual repository and dev preview, not
matched runtime benchmarks. Source checkpoints before the final repair are
`9ba2bd21` and `62015020`.

- Real docs startup: 28,750 ms; warm watcher rebuild: 14,870 ms. Earlier observations
  were 91,769 ms startup and 104,104–114,717 ms warm rebuilds.
- Warm checking: 6,058.8 ms; emission: 4,504.0 ms; web backend: 1,739.6 ms. Earlier
  emission on the same generated output took 73,454.4 ms.
- Full repository LSP scan: 133,894.5 ms cold; 1,068.0 ms for a subsequent scan with
  one changed closed file. Foreground imported-member hover answered in 238.8 ms.
  Peak child resident memory was 482,328,576 bytes. All three stale/member counters
  stayed zero and the server exited normally.

The repository scan includes a synthetic closed-file sentinel and foreground
module. It overlaps the dev rebuild and one harmless landing comment edit; this
is not a quiescent cold-scan benchmark. Earlier timing probes did not canonicalize
macOS temporary URIs, so their missing completion notification and timeouts are
not scan-duration evidence. Their original records remain retained.

Cold repository scanning still takes over two minutes. An individual project
analysis remains non-preemptible; foreground priority applies between entries.
HTML-first loading, executable splitting and further incremental semantic checking
remain separate work. No constant-time or instant-reload claim is implied.

## Focused acceptance

Three exact lifetime-cache witnesses, four recursive-shape witnesses, native
parameter checks, one runtime-support equivalence witness, six project-cache
checks, eight workspace scan checks, six workspace snapshot checks, source-text
sharing and the installed-server foreground protocol witness pass. Successful
configured graphs, failed imports, newly unreferenced files, imported tests,
nested projects, interruption and stale publication rejection are covered.

The real dev preview stays on port 4317. Both normal SDK consumers, both installed
consumers and the configured editor server are refreshed. The active editor has a
new server process; settings bytes and nanosecond modification time are restored
exactly. Installed-server protocol checks verify current imported hover and
open-document diagnostics. Accepted native UI behavior and SQL evidence are reused;
no full compiler/LSP/SQL matrix or runtime benchmark pass was run for this repair.

SQLite retains the exact original profiles, successful and failed checks, consumer
hashes and timing observations. Closed loose records are removed only after
independent byte/hash verification and database integrity checks.
