# Worklist

Native platform-validation application under development. The API and desktop
are separate Tsrust projects, sharing contracts in `shared/tasks.tsr`.

Start the development database from the repository root:

```sh
docker compose -f apps/tasks/compose.yaml up -d --wait
```

Run the API from `apps/tasks/api` with the current verified compiler:

```sh
DATABASE_URL=postgres://worklist:worklist-local@127.0.0.1:5443/worklist \
  ../../../target/release/tsrust dev
```

The development API binds `127.0.0.1:4317`. `WORKLIST_ADDRESS` can select another
loopback address. This local validation service has no public authentication or
remote deployment configuration. Migrations run before listening.

## HTTP contract

- `GET /snapshot` returns all tasks and their committed change cursor in one
  database snapshot.
- `GET /changes?after=CURSOR&limit=200` returns ordered change records after that
  cursor. Page size is 1–500. Apply the entire page and its returned cursor in one
  local transaction, then request another page while `hasMore` is true. An empty
  page preserves the requested cursor. A cursor beyond the current server history
  returns `409 reset-required`, requiring a fresh snapshot.
- `PUT /tasks/ID` accepts the fields in `WriteTask`. Set `expectedVersion` to zero
  to create a task or to its last observed version to update. Successful writes
  return the current task, with status 201 for creation or 200 for an update.
- `DELETE /tasks/ID` accepts `{"expectedVersion": VERSION}` and returns its id.
  Updates/deletes require the current version. A stale version returns status 409
  and the current task, or null if it no longer exists. Review conflicts before
  retrying. Invalid input returns status 400.

A database trigger increments versions, preserves them when an id is recreated, and records external INSERT, UPDATE and
DELETE operations too. Statement-level transaction locking serializes sequence
allocation with commit order. Rolled-back allocations can leave sequence gaps;
clients must use delivered sequence numbers rather than assuming adjacency.
Change history is retained. Administrative schema changes and sequence resets
are outside the supported mutation protocol.

Run the API suites from `apps/tasks/api` while the API is listening:

```sh
DATABASE_URL=postgres://worklist:worklist-local@127.0.0.1:5443/worklist \
WORKLIST_API_URL=http://127.0.0.1:4317 \
  ../../../target/release/tsrust test tests --profile
```

The suites share one normal native runner. Mutation and concurrency cases start
isolated loopback HTTP listeners against the same development database and clean
up their fixture rows. The snapshot case also checks the running API process.
The desktop cache stores rows and its applied cursor in a single SQLite
transaction. The sync client fetches an initial snapshot, applies ordered CDC
pages, and refreshes the snapshot when the server requires a cursor reset. Network
errors leave the last committed cache available. The desktop UI remains in
progress.

To include cache, reconnect and crash-recovery tests in the same native runner,
pass `tests ../desktop/tests` to the test command above. The crash case launches
this already compiled runner with a runtime test filter, holds an uncommitted
SQLite transaction, and lets its parent terminate the child before reopening and
replaying the page. It creates no separate native program or source tree.

## Desktop review

Run `../../../target/release/tsrust dev` from `apps/tasks/desktop` while the API
is running. `WORKLIST_API_URL` selects the endpoint; `WORKLIST_DATA_DIR` overrides
the persistent cache and draft location. The title-bar toggle switches light/dark
themes. Cmd/Ctrl with +, - and 0 changes/reset text zoom.

The database starts empty. The retained changes feed includes deleted test rows
and is not the current task list. To populate 300 clearly marked review tasks,
run this explicit command from `apps/tasks/api`:

```sh
DATABASE_URL=postgres://worklist:worklist-local@127.0.0.1:5443/worklist \
  ../../../target/release/tsrust run tools/seed-review.tsr
```

The seed preserves all existing ids and edits on repeated runs. It uses external
database writes so the running desktop receives the rows through CDC. It is never
run automatically at application startup.
