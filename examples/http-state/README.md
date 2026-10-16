# Retained HTTP state

`main.tsr` moves configuration into a route callback. Concurrent requests borrow
that retained owner read-only, including across suspension. The response explicitly
clones its string; registering the handler does not clone the configuration. Separate middleware and
fallback handlers receive explicit configuration clones and follow the same retained
read-only contract.
Shutdown drains request tasks before the server releases its handlers. Request inputs
are prepared before starting jobs, so preparation cannot abandon earlier started work.

Handlers must own retained values and remain callable through read-only access.
Caller loans, program state, callback-valued state, mutable captures and consuming
captures are rejected. Plain owned configuration, immutable value types and supported
concurrent SQL pools, HTTP clients and async senders may be retained. Local LRU
caches and request/response/body resources cannot be shared this way. A captured
pool belongs to the handler; explicit close inside a repeatable handler is rejected.

This section establishes read-only handler state. It does not yet establish S21
migration recovery, a shared mutable cache, authentication or deployment acceptance.
