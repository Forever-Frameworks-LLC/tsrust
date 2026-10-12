# Local caches: S19

S19's local owned cache depends on accepted S08/S09. Concurrent loading,
stampede prevention and task coordination remain dependent on S14. No public
cache API is accepted yet.

## Owned LRU storage

`stdlib/cache/lru.rs` implements a positive item-capacity storage core. Each
key/value pair has one owner. Hash buckets contain slot indices rather than
duplicated keys; equality resolves collisions. Slots carry a doubly linked
recency order and removed slots are reused. Storage slot count never exceeds
item capacity, including after repeated insertion/removal. This is an item bound,
not a bound on payload bytes or allocator overhead.

An internal get mutates recency and returns a borrow. Peek borrows without changing
recency. Insert makes the new entry most recent and returns the replaced or
least-recently-used entry, including its owned key. Remove transfers the entry.
Clear and dropping the cache dispose retained entries. No Clone bound, hidden
value copy, shared owner or integer handle is used for source values. Slot indices
are internal collection bookkeeping. Randomized hashes provide expected constant
lookup cost; deliberate collisions require equality scans within their bucket.

Three backend tests pass: non-Clone keys with forced collisions and allocation
identity, 5,000 operations compared against an independent deque model with
structural invariants, and exact disposal after eviction/removal/clear/drop.
SQLite evidence: `tsrust-cache-{red,check,final,format}-669e7d3c.log`.
These are runtime implementation tests, not public compiler/native/editor
acceptance or performance measurements.

## Remaining acceptance

1. Add TTL expiration using the monotonic clock and deterministic injected-clock
   tests. Specify expiry boundaries, clock overflow, refresh behavior, cleanup
   timing and interaction with replacement/eviction before public acceptance.
2. Define public construction, borrowed reads, owned removal and eviction results,
   with explicit positive-capacity diagnostics and documented item/byte limits.
   Compile-time loans must prevent mutation or destruction while a returned view
   remains in use. Cloning cached values stays explicit.
3. Integrate generic native types, registry bindings, checked ownership and native
   emission. Complete catalog signatures/help/examples and editor navigation.
4. Add source/native lifecycle and rejection fixtures, matched TypeScript workloads,
   independent result oracles and current source fingerprints during integration.
   Keep timing separate from correctness acceptance.
5. Run full regressions and matched measurements at the cache family milestone,
   review comparable performance and refresh installed consumers. No concurrent
   or external-cache behavior is inferred from the local storage core.
