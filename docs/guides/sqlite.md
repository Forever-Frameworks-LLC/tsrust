# SQLite

`std:sqlite` provides synchronous SQLite connections, prepared statements and transactions. It does not require async. Keep this blocking API off async executor workers.

```tsrust
import { open } from 'std:sqlite';

function main(): void {
  const db = unwrap(open(':memory:'));
  {
    const statement = unwrap(db.prepare('SELECT ?1 AS name'));
    const rows = unwrap(statement.query(['Ada']));
    console.log(unwrap(rows[0].text('name'))); // Ada
  }
  unwrap(db.close());
}
```

`open(path, busyTimeout?)` creates a missing database file or opens an existing one. `:memory:` creates a private in-memory database. Paths are literal filesystem paths, not SQLite URIs. Empty paths and embedded NUL are errors. Foreign keys start enabled. Lock waits default to 5000 milliseconds; pass 0 to fail immediately. The allowed range is 0 through 2147483647.

Prepare exactly one statement. `execute(parameters?)` returns affected rows and rejects statements with result columns before executing. `query(parameters?)` accepts only read-only statements with result columns and returns all rows in an owned array. Use SQL LIMIT to bound large results. Duplicate column names require aliases. Both methods reuse the prepared statement; all parameters must be supplied on every call. Prefer `?1`, `?2`, and so on. Bind user data instead of interpolating it into SQL.

| Parameter | SQLite storage | Row reader |
| --- | --- | --- |
| `int` | signed 64-bit INTEGER, exact | `row.int(name)` |
| finite `number` | REAL | `row.number(name)` |
| `string` | UTF-8 TEXT, including embedded NUL | `row.text(name)` |
| `Bytes` | BLOB, including empty buffers | `row.blob(name)` |
| `null` | NULL | `row.isNull(name)` |

Readers return Result and reject storage-class mismatches. There is no implicit integer-to-number or string-to-number conversion. Column names are exact and case-sensitive. SQL NULL is distinct from zero, empty text and empty blobs. Invalid UTF-8 TEXT is rejected during query. Rows own their data; text and blob readers return independent copies. Map rows into your named application records with a normal function, as in [people.tsr](../../examples/sqlite/people.tsr).

`db.begin()` starts an IMMEDIATE transaction and returns an exclusive borrower of the connection. Prepare statements through that transaction. Release its statements before `commit()` or `rollback()`. Both consume the transaction. Scope exit, return, or throw before a successful commit rolls back. A failed commit also attempts rollback. If rollback fails, further operations reject the connection; close it. If SQLite itself aborts a transaction, further statement work and commit return `transaction-ended` rather than silently running outside it. Raw SQL transaction/savepoint commands are rejected so they cannot bypass this lifecycle. See [rollback.tsr](../../examples/sqlite/rollback.tsr).

Statements borrow their connection or transaction and finalize on scope exit; `finalize()` releases one early. `close()` consumes the connection and is rejected while a borrower remains live. Connections also close on scope exit. SQLite handles cannot be cloned or transferred to a worker; rows can be mapped into ordinary owned data for transfer.

Errors carry `code` and `message`. Handle `busy`/`locked` for contention, `constraint` for integrity failures, `parameters` for binding counts, `column`/`type`/`utf8` for row reading, and `transaction-ended` for an automatic rollback. Other SQLite failures use `sqlite`; messages are human-readable, not stable identifiers.

This first surface does not add an ORM, schema inference, async database pools, migrations, savepoints, extensions or write queries with RETURNING.
