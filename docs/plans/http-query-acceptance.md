# HTTP named query helpers

`Request.query(name)` returns `string | undefined`; `Request.queries(name)` returns
`string[] | undefined`. Both borrow the request and return independent values.
The body remains readable. The Router surface is `c.req.query(name)` and
`c.req.queries(name)`, with no assertion needed to make the call.

Values remain strings. Bare keys and explicit empty values are empty strings;
absent names are undefined. Names are case-sensitive after decoding. Repeated
values preserve wire order. Plus becomes space, valid percent-encoded UTF-8 decodes
once, and undecodable consecutive escape runs remain literal. Empty names do not
match. There is no implicit number, date or JSON conversion.

References reviewed 2026-10-18: [Hono request API](https://hono.dev/docs/api/request)
and [query decoder source](https://github.com/honojs/hono/blob/main/src/utils/url.ts).
This is the named subset, not complete Hono parity. No-argument object overloads
remain unsupported. Tsrust selects the first decoded matching name consistently;
Hono's raw-name fast path can prefer a later literal name over an earlier encoded
spelling. For example `%61=first&a=second` queried as `a` returns `first` here.
The behavior is explicit rather than inheriting that optimization-dependent order.

The paired workload checks ordinary valid queries against Node URLSearchParams
with an independent expected output. Malformed UTF-8 behavior deliberately does
not use Node's lossy replacement behavior. These are untimed witnesses.

Focused native checks pass for repeated values, independent results, encoded
names, case sensitivity, empty-name rejection, absence, malformed bytes and body
reuse. Wrong arity/type calls reject. Both catalog examples execute after format
round trips. The formatted Router example returns `one two, three` through a real
loopback request and shuts down. Its first sandbox attempt could not bind; the
permitted loopback run passes. The paired native/Node workload passes.

Three completion matrix checks pass, including the request row in all four modes.
The query hover/signature contract passes. Whole-project formatting and Git
whitespace checks pass. Evidence uses `tsrust-http-query-*` in canonical SQLite. No SQL,
full-family regression or performance timing was requested or rerun.
