<!-- tsrust-docs
{"id":"reference-standard-library-network","group":"standard-library","order":24,"status":"partial","targets":["native"],"api":[{"receiver":"std:net","name":"IpAddress"},{"receiver":"std:net","name":"SocketAddress"},{"receiver":"std:net","name":"NetError"},{"receiver":"std:net","name":"parseIp"},{"receiver":"std:net","name":"parseAddress"},{"receiver":"std:net","name":"formatIp"},{"receiver":"std:net","name":"formatAddress"},{"receiver":"std:net","name":"isIPv4"},{"receiver":"std:net","name":"isIPv6"},{"receiver":"std:net","name":"TcpListener"},{"receiver":"std:net","name":"TcpStream"},{"receiver":"std:net","name":"UdpSocket"},{"receiver":"std:net","name":"Datagram"},{"receiver":"std:net","name":"listenTcp"},{"receiver":"std:net","name":"connectTcp"},{"receiver":"std:net","name":"bindUdp"},{"receiver":"TcpListener","name":"accept"},{"receiver":"TcpListener","name":"localAddress"},{"receiver":"TcpListener","name":"close"},{"receiver":"TcpStream","name":"localAddress"},{"receiver":"TcpStream","name":"peerAddress"},{"receiver":"TcpStream","name":"setNoDelay"},{"receiver":"TcpStream","name":"shutdownWrite"},{"receiver":"TcpStream","name":"split"},{"receiver":"TcpStream","name":"close"},{"receiver":"UdpSocket","name":"sendTo"},{"receiver":"UdpSocket","name":"receiveFrom"},{"receiver":"UdpSocket","name":"localAddress"},{"receiver":"UdpSocket","name":"close"}]}
-->
# TCP and UDP sockets

`listenTcp`, `connectTcp` and `bindUdp` return explicit lazy
`Task<Result<Owner, NetError>>` values for parsed numeric SocketAddress values.
Port-zero binds retain the OS choice; localAddress observes the assigned endpoint.
No timeout, retry, connection cap, backlog, reuse, Nagle, keepalive or linger
policy overrides are installed. Configure setNoDelay deliberately when needed.

TcpListener.accept borrows a listener and returns `[TcpStream, SocketAddress]`.
Closing the listener consumes it; accepted streams remain independent. Full
TcpStream supports std:io read/readInto/readExact/readAll/write/writeAll/flush/copy
through exclusive loans, returning IoError rather than NetError. No additional split
Arc or half boxes are required for these full-stream operations. Ordinary B05
working-buffer defaults, explicit bounds and partial-progress semantics apply.
TCP has no message boundaries. shutdownWrite delivers EOF while retaining reads;
full close consumes/drops the socket without an invented flush/error protocol.

Split consumes TcpStream and returns existing B05 Reader/Writer owners. Close the
Writer to await shutdown; its drop also sends EOF. The Reader can continue after
the Writer closes. The underlying socket lives until both owners drop. No clone,
reunite, public generic trait or protocol framing layer is introduced.

UDP sendTo sends one datagram, including an empty payload. receiveFrom requires
an explicit nonnegative capacity fitting the target and returns owned binary data
with its sender endpoint. Capacity zero still receives. Oversized messages keep
OS errors; insufficient capacity can discard excess bytes or return a platform
error. Empty data is never stream EOF. No reliable truncated flag or hidden total
byte cap is invented. Send and receive share the backend's concurrent operations.

Stable inputs remain loaned through Task settlement; inline temporaries move into
the Task. Started jobs use owned async factories. Cancelling a pending borrowed
operation leaves the retained socket usable; completed progress is not rolled
back. Pending accept/receive are cancellation safe according to Tokio semantics.
All endpoint owners are unique, nonclonable and Send. Addr/error/datagram snapshots
follow their ordinary data ownership. See examples/network for complete loopback,
half-close and empty-datagram applications.
