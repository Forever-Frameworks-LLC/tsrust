# Native SQLite task repository

Focused correctness acceptance passes for the repository source in `f729211`.
This is the domain/persistence layer of the prototype, not the HTTP service.
Generated private-context dispatch, access policy, frontend consumers and native
service lifecycle acceptance remain pending.

Source is `examples/backend-tasks/repository.tsr`, migration
`examples/backend-tasks/migrations/1_tasks.sql`, native witness
`examples/backend-tasks/witness.tsr` and runner `tools/verify-task-repository.mjs`.
The implementation reuses `std:sql` pooling, parameter binding, migrations and
checked row access. Database handles are borrowed private inputs, never wire DTOs.

Create/update/delete use atomic `RETURNING` statements rather than a connection-
local last-insert-id query or a separate read that could observe another write.
Titles are trimmed and limited to 1–200 UTF-8 bytes, matching both Tsrust string
length and SQLite's BLOB-length constraint. Ids are positive safe JavaScript
integers, also constrained by SQLite. List cursors are exclusive, ordered and
nonnegative, with a required limit from 1 to 100. Invalid domain input and missing
tasks return named plain-data failures. SQL/runtime errors remain private and
must be projected at the upcoming server boundary.

## Verified evidence

- Whole-project formatting and stable second pass: new fixtures were formatted;
  subsequent runs changed zero files/fingerprints. The formatted project passes
  `tsrust check examples/backend-tasks/witness.tsr`.
- `node tools/verify-task-repository.mjs` passes with the final HTTP compiler.
  It compiles the witness once, runs the native executable against a fresh disk
  SQLite database with a space in its path, then starts another process against
  that same database. The second process verifies the exact retained title,
  completion flag and row count after migrations run again.
- Native assertions cover empty state; create/trim; blank and over-limit titles;
  200-byte ASCII and Unicode boundaries; fifty accepted rocket emoji and fifty-one
  rejected; one-row pagination/cursor ordering; update and invalid-update
  preservation; missing update/delete; negative ids; bounded/zero list limits;
  and a SQL-looking title that is stored literally without changing the table.
- The runner checks both exact success messages, verifies a nonempty on-disk
  database and removes the temporary database directory after both processes
  release it. It retains one native executable for affected `--run-only` checks.

The build/execution session `80514` completed with exit 0; concise output is in
`target/task-repository-native.log`:
`Task repository: native CRUD, validation, parameterized SQL, bounded cursors and process-restart SQLite persistence passed.`
Compiler: SHA-256 `9eefc40ff18da31e80d51a5c5069db27a80de28437789d45d876b756b5e60338`.
One shared Cargo target/job and `CARGO_INCREMENTAL=0` were used. Normal private-temp
permission was used for native compilation. No desktop interaction, database
analytics import, performance timing, broad regression matrix, CI or publication
is claimed.

## Next acceptance

Register only declared operations through generated native HTTP dispatch. Hide
only an explicitly selected borrowed native Context parameter, with application
state and authorization outside wire data. Keep server validation derived from
the checked contract, including tuples and unions; never use unchecked casts to
bridge missing decoding support. Preserve client compatibility and source/output
ownership during regeneration. Then expose this repository through an application-
owned authenticated service and connect the named query wrappers and frontends.
