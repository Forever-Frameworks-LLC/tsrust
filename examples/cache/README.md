# Owned local cache

`owned-lru.tsr` demonstrates lookup, recency, replacement, explicit cloning,
owned eviction, rejected TTL inputs and independent resource-release observation.
Run with `tsrust run examples/cache/owned-lru.tsr` after cache integration acceptance.

`get` and `peek` borrow non-copy values read-only. Their operation may first dispose
expired entries, so the cache itself is exclusively borrowed until the result's
last use. Call `clone` explicitly when an independent value is needed. `remove`
and successful displacement return owned `CacheEntry<K,V>` records.

TTL is an optional positive integer number of monotonic milliseconds. No TTL
means no expiration. Reads do not refresh deadlines. Cleanup runs during cache
operations; there is no timer task. Failed TTL validation returns the original
key/value in `CacheInsertError<K,V>` without changing existing entries.

Keys currently require one string, int, number, boolean or enum type. Mixed-union
and object keys are rejected. Capacity bounds entry count rather than bytes.
Borrowed-view payloads, callbacks and task handles are unsupported. The local cache
cannot be cloned or transferred across workers. Concurrent loading, stampede
prevention and external caches remain separate acceptance work.

A normal read does not require a non-null assertion:

```tsrust
const value = cache.get(key);
if (value !== undefined) {
  console.log(value);
}
```

Handle absence explicitly because a key can be missing, expired or evicted.
The `!` operator is only for callers deliberately asserting that an entry exists;
it raises an error when that assumption is false.

In ordinary acyclic function bodies, a read releases its cache loan after its last
use, including after an `if` branch. In loops, deferred callbacks and exception
regions, use an inner block to end the read before another cache operation; those
contexts conservatively retain the existing lexical loan boundary. Named capture
plans also retain that conservative boundary. Aliases cannot bypass an active loan.

## Serialized loading with concurrent callers

`serialized-loading.tsr` constructs a cache inside one owned async task. Two
concurrent clients send owned requests through a bounded channel. Each client
has a dedicated bounded reply channel and at most one outstanding request.
Endpoints are startup arguments, never channel messages. Cache hits explicitly
clone strings for replies; the cache and its borrowed values never cross tasks.

The worker finishes each load before reading the next request. Requests for an
already loaded key reuse the cached value, so successful same-key misses cause
one load while the value remains cached. Loader failures are returned to callers
and not cached; a later request retries. Different keys also wait behind the
active load. This example does not provide parallel per-key loading or shared
promises. Capacity bounds entries and channel messages, not payload bytes.

The acceptance example handles six requests, closes channels, clears the cache
and settles every started job. Separate branches cancel a loader after its readiness
signal and handle a client that closes its reply channel. Settlement occurs in
finally even when the readiness wait throws. Cancellation does not promise to run
source finally blocks inside the cancelled loader; owned future state is dropped.
The backend release witness checks disposal independently of a cache read.

`benchmarks/cache/verify-loading.mjs` checks a compiled TypeScript counterpart
against the source example. Its matched scope is four successful reads served by
one load and two uncached failures retried separately. Native cancellation and
closed-channel checks are additional native acceptance, not cross-runtime parity
or timing claims. The scalar LRU workload remains the performance workload.
