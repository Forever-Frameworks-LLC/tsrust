# Application-owned SQLite jobs

Run against a fresh disposable file:

```sh
DATABASE_URL='sqlite:/tmp/jobs-demo.sqlite?mode=rwc' tsrust run examples/durable-jobs/main.tsr
```

The example uses existing `std:sql` pools and transactions. `jobs.tsr` is application
code, not a standard-library queue. The executable uses explicit millisecond times
to demonstrate retry and lease expiry without sleeping. A real worker supplies a
consistent wall clock, polls with a bounded delay, and stops claiming before shutdown.
The demonstration expects an empty database and leaves its rows available to inspect.

`enqueue` deduplicates by application job ID and returns false for a conflicting payload.
One atomic SQLite `UPDATE ... RETURNING` claims the earliest eligible job. Claims
last 30 seconds and increment a generation number. Only the current generation can
retry or acknowledge before expiry. Failed attempts wait `attempt * 1000`
milliseconds; three attempts exhaust the job. Expired final attempts become failed
on the next claim. Jobs retain terminal rows so repeated enqueue calls remain
idempotent. Retention, manual replay, payload validation and monitoring are application
policy. Time and generation arithmetic uses checked integers; overflow fails.

Delivery can repeat after a crash or lease expiry, up to the configured attempt
limit. This is bounded retry delivery, not exactly-once processing or a guarantee
that every job eventually succeeds. Here the effect is a row in the same SQLite
database: the effect and acknowledgement commit together. A stale worker cannot
commit that effect after losing its lease. External effects such as email or HTTP
need their own durable idempotency key; a SQL transaction cannot undo them.

Supported scope: file-backed SQLite, short transactions, one effect table and a
single bounded worker demonstration. Concurrent workers use the same atomic claim;
SQLite busy failures must be retried by the application. No lease renewal, remote
broker, distributed clock coordination, transaction spanning external work, PostgreSQL
or MySQL dialect adapter is provided. Workers whose work exceeds the lease must not
assume they still own the job. Do not hold a database transaction while doing remote
work. Authentication and access control belong to the application.
