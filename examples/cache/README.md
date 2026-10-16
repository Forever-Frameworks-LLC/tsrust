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
