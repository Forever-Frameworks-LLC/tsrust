# Local caches: S19

S19's local owned cache depends on accepted S08/S09. Concurrent loading,
stampede prevention and task coordination remain dependent on S14. The local public API now passes focused acceptance; see
[local acceptance](../milestones/s19-local-cache-acceptance.md). Concurrent loading,
stampede policy and family measurements remain pending. The backend checkpoint
sections below retain historical evidence; their earlier next steps are superseded.

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

## Timed operation layer

`timed.rs` connects an injected monotonic clock to the storage core. Get, peek,
remove and length purge expired entries before observing live storage. Insert
validates a deadline before any mutation, then purges expiration before capacity
eviction. Thus an expired most-recent entry cannot cause eviction of a live
least-recent entry. Reads preserve the insertion deadline. Purging drops expired
owners; explicit removal still transfers a live entry to the caller.

The internal insertion contract accepts no deadline or a positive Duration.
Zero and Instant overflow return distinct errors containing the original owned
key/value. Failed insertion does not replace, evict or expire existing entries.
This prepares source integration without choosing public duration units or error
signatures. Public documentation and checker integration must expose cleanup as
mutation: even peek/length can dispose expired entries and cannot run while an
outstanding borrowed value forbids cache mutation.

All eight cache backend tests pass, including fake-clock expiry during reads and
capacity checks, unchanged live entries on invalid TTL, and allocation identity
of returned failed inputs. SQLite evidence:
`tsrust-cache-timed-{red,check,final,format}-b4eee6a3.log`.
The namespace is still not exposed; source acceptance and performance remain open.

## Borrowed read boundary

Three negative native compilations use the actual timed/core modules and reject
length/expiration, replacement and owner destruction while a borrowed result is
subsequently used. A positive fake-clock witness releases the borrow, expires the
entry and verifies exactly one disposal. All ten cache backend tests pass.
SQLite evidence: `tsrust-cache-loans-566e6a9a.log` and
`tsrust-cache-loans-format-566e6a9a.log`.

The current timed get/peek signatures require a mutable receiver and retain that
receiver loan through their borrowed result. Source integration must model that
exclusive owner loan even though the returned value itself is read-only. Do not
classify these as ordinary Map.get reads or allow an operation that would fail
only in native compilation. These checks establish the backend boundary; they
do not yet prove Tsrust diagnostics or source-level lifetime inference.

## Proposed public contract for review

The following is a proposal, not an accepted or available source API. Public
integration must not silently infer approval from the internal backend behavior.

| Operation | Proposed observable behavior |
| --- | --- |
| `new LruCache<K,V>(capacity)` from `std:cache` | Positive integer item capacity; invalid capacity raises a checked runtime error. No default expiration and no background task. |
| `get(key)` | Purges expired entries, updates recency and returns a read-only borrowed `V | undefined`. |
| `peek(key)` | Purges expired entries and returns the same borrowed result without updating recency. |
| `set(key, value, ttlMs?)` | Takes owned key/value. Optional TTL is a positive integer number of milliseconds from insertion, using the monotonic clock. Reads do not extend it. |
| Successful set result | `Ok` contains the displaced owned key/value entry, or undefined if nothing was displaced. Expired entries are disposed before live-entry capacity eviction. |
| Failed set result | `Err` contains a stable error code and the original owned input key/value. Invalid TTL or deadline overflow leaves the cache unchanged. |
| `remove(key)` | Purges expiration and transfers the live key/value entry, or returns undefined. |
| `size()` | Purges expiration and reports the number of live entries. It is a method because it can perform cleanup. |
| `clear()` | Disposes all retained entries. Scope exit also disposes them deterministically. |

Get/peek retain an exclusive cache-owner loan until the borrowed result's last
use. That prevents further cache operations during the loan, even another lookup.
The returned value itself remains read-only. Explicit clone(value), when supported,
provides an independent value; remove transfers ownership without cloning. No
automatic sharing, hidden clone, concurrent access or background expiration is
introduced. Item capacity does not promise a payload-byte bound.

The ownership restriction and operation-driven cleanup are the material review
decision. After that decision, implement the complete constructor/operations as
one source integration family with diagnostics, catalog help, examples, source
ownership tests and matched workloads. Backend evidence alone cannot close S19.


## Owner decision (2026-10-12)

Approved: get/peek borrow stored values; explicit clone creates an owned copy when supported. Expiration cleanup occurs during cache operations. Further operations are rejected while an outstanding borrowed result is still used.

## Public integration acceptance

The source integration now covers constructor and operation metadata, native type
identity, owned eviction/error records, exclusive owner loans for read-only
lookups, editor signatures/examples and a matched scalar LRU workload. Focused source acceptance passes; the complete S19 family remains open.
The initial public key surface deliberately accepts one scalar or enum type;
mixed-union and object keys are diagnosed rather than emitted with incompatible
Rust query types. Concurrent loading and stampede prevention remain outstanding.

The first last-use proof is deliberately limited to acyclic local function bodies.
Branches preserve exclusive loans until every later reference is past. Aliases
carry their own retained loans. Loops, deferred callbacks, named capture plans and
exception regions keep the existing lexical boundary; use an inner block to end
a read before another cache operation there. This avoids claiming general
nonlexical lifetime inference or weakening existing cleanup/borrow checks.

## Serialized loading integration acceptance

The source witness `examples/cache/serialized-loading.tsr` uses one cache-owning
async worker and bounded request and per-client reply channels. It serializes
loads, explicitly clones reply strings, caches successful loads and retries
failures. This is a bounded fixed-client composition, not a generic shared-cache
API. Different keys wait behind each other. Endpoint-bearing channel messages
remain prohibited.

The witness verifies four successful reads with one load, two uncached failures,
cancellation after a readiness signal with settlement in finally, and a closed
reply receiver. An independent backend counter verifies exactly one drop of a
cached owner when its suspended future is dropped. No source finally guarantee
is added for cancellation. The compiled TypeScript counterpart matches successful
reuse and retry counts only; it makes no cancellation or ownership parity claim.

Integration repairs mutable emission for owned closure captures and permits
explicit cloning of CacheEntry/CacheInsertError only when their fields support
independent cloning. Resource payload cloning still rejects. Cache catalog fields
now complete as properties rather than methods. All three cache receiver types
pass checker agreement; four completion rows pass all four timing/error modes.

SQLite holds 25 exact `tsrust-cache-loading-*` records, including original failures,
focused repairs, native/Node verification, formatting, installed hashes and the
acceptance summary. Whole-project formatting changed one new file, then zero;
installed CLI/LSP consumers are refreshed and the restarted editor reports no
workspace problems. Previous compiler/resource and database evidence is reused.
No SQL or task-lifecycle runtime behavior changed; no three-driver rerun was needed.

S19 remains open: generalize the bounded loading/shutdown contract beyond this
fixed-client witness, then complete the cache family regressions and scalar LRU
measurements. No timing has been performed for the new cache workloads. Continue
S20 and S21 afterward without another permission checkpoint.
