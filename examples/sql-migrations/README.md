# Versioned SQL migrations

Run from the repository root with a disposable persistent database:

```sh
DATABASE_URL='sqlite:/tmp/tsrust-service.sqlite?mode=rwc' tsrust run examples/sql-migrations/main.tsr
```

The same files also work with PostgreSQL and MySQL URLs. The example applies two
versions, repeats them without inserting another row, then verifies the service's
required columns. Ship the migration directory with the executable; paths resolve
from its working directory, and package export does not embed SQL files.

Migration directories are trusted deployment code. Direct `.sql` files must use
unique positive signed-64-bit versions and nonempty descriptions. Other file types
are ignored. Down migrations and `-- no-transaction` are rejected. Scripts may
contain multiple statements but must not issue transaction/session control or
modify `_sqlx_migrations`. Do not let request input select directories or SQL.

Run migrations before accepting requests, with a backup and one deployment owner.
PostgreSQL and MySQL use database migration locks; SQLite uses its transaction
locking. No lock-wait deadline is added: wrap the migration Task in the existing
bounded task composition when required. Cancellation closes its connection through
tracked cleanup; the pool remains owned and reusable. In-memory and temporary SQLite databases are rejected
because closing a failed migration connection could destroy the database.

`migration-drift` means an applied file changed or is missing. Restore the original
bytes; add a new version for changes. Checksums do not detect arbitrary database
edits: keep explicit startup queries for required table/column shapes. These checks
do not prove every constraint or index matches an ideal schema.

`migration-failed` means execution failed. PostgreSQL and SQLite roll back that
migration, not earlier successful versions. Correct an unapplied script and retry.
MySQL DDL can commit partially, leaving `migration-dirty`. Stop deployment, inspect
schema and ledger, restore the exact pre-migration state from verified recovery
instructions/backup, then have an operator reconcile only that failed version's
ledger entry before retrying. Never automatically delete the ledger, mark a failed
migration successful, or continue serving with an unknown schema. The integration
test demonstrates explicit restoration on its own disposable tables only.

The API provides no automatic repair, rollback/down workflow, SQL translation,
production backup tooling, or whole-schema comparison. Regular transaction methods
still reject DDL and session control.
