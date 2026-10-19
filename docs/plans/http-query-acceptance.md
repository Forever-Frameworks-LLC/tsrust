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

## Nested request editor repair — 2026-10-18

The original editor witness used a standalone `Request` variable and missed
`Context.req`. Native field signatures rendered public type names while their
editor declarations used private identities. This made `ctx.req.queries` compile
successfully but return no hover when only `Context` was imported.

Native field signatures now retain the registered declaration identity, including
aliases and types not explicitly imported. Focused witnesses cover plain and
generic Context parameters, an inferred Router callback, an aliased Request import
and a user record named Request. Hover, parameter help and completion must agree.
Typing-speed matrix rows cover both typed and inferred nested request access.

The name `queries` is part of Hono's request API; it is not a Tsrust invention.
This repair does not complete Hono parity. The missing no-argument `query()` and
`queries()` object overloads and the encoded-name ordering difference above remain
open. Preserve the requested record shapes when implementing those overloads;
do not substitute Map or advertise the named subset as a 1:1 interface.

Focused acceptance passes: both query project contracts, three HTTP context
protocol checks, nested request editor consistency, both nested request matrix
rows in all four modes, and Context/Request checker agreement. Whole-project
formatting changes zero files/fingerprints. Native behavior is unchanged; existing
query execution evidence is reused. Repair evidence uses `tsrust-http-hover-*`
in canonical SQLite, including the original missing-hover reproduction.
