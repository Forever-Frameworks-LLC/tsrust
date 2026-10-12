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

1. Integrate the internal deadline index below with public TTL operations.
   Specify duration conversion/clock overflow, refresh behavior, cleanup
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

## Monotonic deadline index

The storage core accepts optional absolute Instant deadlines. Its explicit
expire(now) operation moves out entries with deadline <= now. Reads update only
LRU recency; they do not extend deadlines. Replacement removes the old deadline,
including when the replacement has no deadline. Eviction, removal and clear also
remove deadline metadata before a slot can be reused. The ordered deadline index
contains at most one record per live entry; repeated updates do not accumulate
stale timer records. Expiration processes due entries rather than scanning the
whole cache. Returned expired values retain ownership until the caller disposes
or reuses them.

Five backend tests pass. Deterministic timestamp injection covers just-before and
exact expiry, equal deadlines, deadline removal/extension, elapsed deadlines,
slot reuse and 5,000 refresh/eviction operations. Drop counters verify that expired
values are released exactly once by their new owner. No sleep or wall-clock
timing determines results. SQLite evidence:
`tsrust-cache-expiry-{red,check,final,format}-3b666421.log`.

This is an internal expiration mechanism. It does not yet select public lazy or
eager cleanup, default TTL, duration units, overflow behavior or byte accounting.
The source-facing layer must invoke expiration before relying on live-entry
results; the raw LRU core deliberately requires an explicit supplied clock value.
Source/compiler/editor integration and matched workloads remain outstanding.

## Source key compatibility

The core now separates storage from key hash/equality/normalization policy.
Ordinary Rust-key tests keep their randomized Hash/Eq policy. The source adapter
in `key_policy.rs` reuses existing Map/Set operations directly, so floating-point
keys do not accidentally inherit Rust's unavailable f64 Eq/Hash contract or a
different equality rule. Neither policy clones keys. The adapter uses the existing
source key eligibility contract; it does not expand supported source key types.

A compiled native witness combines the actual Map/Set key runtime, cache core
and adapter. It verifies equivalent NaN payloads, canonical positive zero after
inserting negative zero, infinity, and distinct exact integers above 2^53 whose
hashes may collide. All six cache backend tests pass. SQLite evidence:
`tsrust-cache-keys-{red,check,final,format}-dfbb70eb.log`.
This closes a native integration mismatch; public registry/type checking,
expiration policy, editor surface and matched workloads remain outstanding.
