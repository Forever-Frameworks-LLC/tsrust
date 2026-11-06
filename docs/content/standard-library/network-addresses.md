<!-- tsrust-docs
{"id":"reference-standard-library-network-addresses","group":"standard-library","order":23,"status":"partial","targets":["native"],"api":[{"receiver":"std:net","name":"IpAddress"},{"receiver":"std:net","name":"SocketAddress"},{"receiver":"std:net","name":"NetError"},{"receiver":"std:net","name":"parseIp"},{"receiver":"std:net","name":"parseAddress"},{"receiver":"std:net","name":"formatIp"},{"receiver":"std:net","name":"formatAddress"},{"receiver":"std:net","name":"isIPv4"},{"receiver":"std:net","name":"isIPv6"},{"receiver":"std:dns","name":"lookup"}]}
-->
# IP addresses and system lookup

`std:net` parses address values without opening sockets or performing DNS.
`parseIp` accepts std IPv4/IPv6 grammar; `parseAddress` accepts an IP and port,
with brackets around IPv6. Numeric socket scope IDs survive formatting. Bare
IpAddress has no zone/scope. Hostnames, implicit trimming and port guessing are
not added. Invalid text returns `Result` error code syntax. `formatIp` and
`formatAddress` return owned canonical text. `isIPv4`/`isIPv6` observe parsed values.
Addresses are independent cloneable Send snapshots, with no mutable fields.

`std:dns.lookup(host, family?)` returns an explicit lazy
`Task<Result<IpAddress[], NetError>>`. Family is any by default; ipv4/ipv6 filter
the system result without changing resolver policy. Ordering and duplicates are
preserved; a filter can yield an empty array. Literal addresses use the backend
fast path. There is no added cache, TTL, record-query API, retry, timeout or cap.
A running OS resolver call may finish after cancellation releases its awaiter.
Caller timeout never proves that a resolver thread was forcibly stopped.

Stable host text is loaned through task settlement. Inline temporaries move into
the Task. Started work must own its host through an async factory; successful
results own their address storage. NetError owns code, operation, message and
optional osCode. OS messages/numbers are retained; parse failures omit osCode.
See examples/network-addresses for address and literal-resolver examples.

TCP/UDP endpoints compose with these address values. See the socket reference and
examples/network for full-stream B05 I/O, independent halves and datagrams.

When retaining or forwarding a Task from a stored `lookup` function value, keep
its borrowed host and family values in named storage. This follows the same
borrow rules as an ordinary `ReadOnly` async task factory. Calls with temporary
arguments can be awaited directly. Direct built-in calls capture temporary text
into their Task. Use an owned async factory before starting work as a Job.
