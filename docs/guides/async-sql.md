# Async SQL

`std:sql` uses SQLx for PostgreSQL, MySQL and SQLite. Database operations return lazy Tasks. Await them to perform I/O; catch `SqlError` for failures. Row accessors return `Result` and use `unwrap` or `match`.

```tsrust
import { connect } from 'std:sql';

async function main(): Task<void> {
  const db = await connect('sqlite::memory:', 1);
  await db.execute('CREATE TABLE people(id BIGINT, name TEXT)');
  {
    const transaction = await db.begin();
    await transaction.execute('INSERT INTO people VALUES (?, ?)', [1, 'Ada']);
    await transaction.commit();
  }
  const rows = await db.query('SELECT id, name FROM people');
  await db.close();
  console.log(unwrap(rows[0].text('name'))); // Ada
}
```

Use a PostgreSQL or MySQL URL for a server database. Read credentials from the environment rather than committing them. PostgreSQL uses `$1`, `$2` placeholders; MySQL and SQLite use `?`. SQL text and schema types remain database-specific. SQL text and parameter arrays move into the lazy task when constructed. Use `clone` when an owned argument must be reused. Bind values; do not concatenate untrusted values or identifiers into SQL.

A pool defaults to five connections and a 30000 ms acquisition timeout. `connect(url, maxConnections, acquireTimeout)` accepts 1..1024 connections and 1..31536000000 milliseconds. Acquisition timeout bounds waiting for a connection, not statement execution. Use one connection for a private in-memory SQLite database.

Transactions retain a pool borrow. Pending transaction operations hold exclusive transaction borrows. Commit and rollback consume the transaction; leaving scope without committing schedules rollback. A database/parameter failure or cancellation during transaction query/execute invalidates that transaction and schedules rollback. Cleanup survives task cancellation and completes before an owning job finishes joining. Cleanup can wait for an in-flight database statement; cancellation is not a server-side query deadline. Closing a pool consumes it and waits for connection cleanup. Cancelling a commit or losing its response cannot prove that the server did not commit; reconcile before retrying non-idempotent work.

Rows own their results and can outlive the pool. `int`, `number`, `boolean`, `text`, `blob` and `isNull` use exact column names. Missing columns, incompatible types and non-finite numbers fail. Alias duplicate column names. Cast database-specific types into supported scalar types in SQL. Queries materialize all rows and may perform writes when SQL uses a returning clause. PostgreSQL Boolean columns support `boolean`; SQLite and MySQL integer-backed flags use `int`.

Each operation accepts one prepared statement. Raw transaction/session control is unsupported. Transaction DDL is excluded because MySQL can commit it implicitly. Run schema changes outside transactions only when that behavior is intended. Migration orchestration, streaming cursors and compile-time database schema checking are not included.

## Acceptance

Database acceptance is explicitly opt-in:

```sh
node tools/check-async-sql.mjs --docker
```

The runner creates disposable PostgreSQL and MySQL containers bound to loopback, tests SQLite too, then removes only its own containers and volumes. Normal Cargo tests skip the database integration test. Compiler/editor tests remain enabled. Rerun database acceptance when SQL runtime, SQLx dependencies, async lifecycle, SQL ownership or native emission changes.
