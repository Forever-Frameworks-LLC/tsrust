# Explicit SQL domain encodings

`tsrust run examples/sql-domain-values/sqlite.tsr` demonstrates exact text/blob
storage. Database acceptance is opt-in: `node tools/check-async-sql.mjs --docker`.
That runner also exercises PostgreSQL 18 and MySQL 8.4 typed columns through
explicit SQL text/byte encodings. It creates and removes disposable containers.

The Any-driver public API does not directly bind/decode Decimal, Date, Time,
Instant, TimeZone or Uuid. The application validates values, binds scalar text or
bytes, explicitly casts typed query results to text, and parses them back. These
are serialization adapters, not native typed Any-driver support.

- Amounts use exact NUMERIC/DECIMAL(38,18); the helper rejects excess integer digits
  or fractional loss before binding. SQLite uses TEXT, never NUMERIC affinity.
- Dates use the shared 1000..9999 year subset. Time and UTC timestamps reject
  sub-microsecond precision. PostgreSQL TIMESTAMP and MySQL DATETIME are explicitly
  UTC application columns; no session zone conversion is requested or inferred.
  Source nanosecond values need exact text storage or a separate nanosecond field.
- Named IANA zones are separate validated text; the instant does not retain one.
  Fixed-offset zones need an explicit offset-seconds column instead of this named
  zone mapping. Zone database provenance follows the source library pin.
- Durations use signed BIGINT nanoseconds, not SQL calendar intervals.
- UUID uses PostgreSQL UUID with text casts, MySQL BINARY(16), or SQLite BLOB(16).
  Byte order is the standard network order. Parse/length checks remain explicit.
- JSON uses PostgreSQL JSONB, MySQL JSON, or validated SQLite TEXT. Source JSON
  integer precision limits still apply; arbitrary-precision JSON numerics are not
  promised. SQL NULL is distinct from a JSON null field.

The example retains owned rows after closing the pool. It also verifies early
rejection of decimal scale/range, time precision and out-of-subset years. No
implicit clone, numeric rounding or ownership relaxation is used. Typed domain
arrays, SQL intervals, infinity/BC dates and arbitrary driver-specific types remain
unsupported; use a separately validated application representation if needed.

Primary contracts: PostgreSQL 18 [date/time types](https://www.postgresql.org/docs/18/datatype-datetime.html)
and MySQL 8.4 [fractional seconds](https://dev.mysql.com/doc/refman/8.4/en/fractional-seconds.html).
This section adds correctness witnesses, not a throughput claim; existing S18
performance protocols remain unchanged.
