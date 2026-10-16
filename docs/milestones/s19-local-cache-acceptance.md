# S19 local owned cache acceptance

The local `std:cache` source integration passes focused macOS acceptance. S19 is
not complete: concurrent loading, stampede policy and the family measurement review
remain pending. External cache clients remain separate scope.

`LruCache<K,V>` owns bounded entries. Capacity must be a positive int and invalid
construction raises a catchable RangeError. Get/peek purge expiration and return
read-only optional values; only get updates recency. Ordinary reads use an
undefined check, with no required non-null assertion. Non-copy reads retain an
exclusive cache-owner loan. Copy scalars are snapshots. Set returns displaced
owned entries, and invalid TTL returns the original owned inputs without changing
existing storage. TTL is positive monotonic milliseconds; reads do not refresh it.
Remove transfers ownership; clear, expiry, eviction-result disposal and scope exit
release owners. No hidden clone, shared payload owner or background cleanup exists.

The checked last-use proof covers acyclic local function bodies and branches.
Loops, deferred callbacks, named capture plans and exception regions retain the
existing lexical boundary; use an inner block there. Aliases retain their own
loans. This does not claim general nonlexical lifetime inference.

Supported keys are one string, int, number, boolean or enum type. Object and mixed
union keys reject. Values may contain owned data, nullable values and native
resources. Borrowed views, callbacks and task handles reject. Local caches cannot
be cloned or transferred to workers. Capacity is an item bound, not a byte bound.

## Accepted evidence

- Ten existing backend checks pass, including deterministic fake-clock expiry,
  collision/key semantics, bounded metadata and exact destruction.
- Seven compiler source checks pass, including 14 batched native catalog examples,
  overlap diagnostics, aliases/backedges, readonly projection, local escape,
  invalid capacity and resource-bearing failure reporting.
- The expanded resource fixture passes separately after adding rejected-input
  release observation, resource reads and nullable payloads.
- Two focused editor witnesses and their shared support check pass: aliases,
  generic hover/parameter help, live server reads and checked loan conflicts.
- The matched scalar LRU workload verifies native and Node-after-tsc checksum
  97936. No timing samples were retained; it remains unmeasured.
- Whole-project formatting changes two new sources, then a second pass changes
  zero files. Native fixture checks include token-preserving, idempotent formatting.
- Installed CLI/LSP hashes are recorded in SQLite. The active VS Code server was
  restarted and reports checking enabled with no workspace problems.

Exact original failed attempts and successful repairs are retained in SQLite as
`tsrust-cache-*.log`, with current evidence in `tsrust-cache-acceptance.json` and
`benchmarks/cache/cache-lru-2026-10-16T03-57-01-986Z-verify.json`. Failures included
missing error metadata, inference for stored optional reads, overly long loan
retention, and generated error reporting for opaque resource inputs; all are
repaired. Earlier borrowed-read rejection witnesses were strengthened to require
the actual owner-conflict diagnostic rather than accepting an unrelated error.

The accepted data-family full regression and SQL three-driver evidence were
reused. No full regression, database rerun or timing pass was triggered by this
local integration section. Complete remaining S19 loading policy before its family
milestone, then continue S20 and S21 without another approval checkpoint.
