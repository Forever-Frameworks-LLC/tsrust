# Scoped workers

Run `tsrust run examples/scoped-workers/main.tsr`. Expected output: `25205`.

`spawnScoped` starts each worker immediately. `unwrap` returns the successful
value or throws the original typed error. Start both workers before joining to
allow overlap. The caller can do other work before collecting their results.

Each scoped handle registers fallback cleanup with its owner: leaving that scope
waits for any worker not already joined or explicitly detached. This includes
returns and errors. Moving the handle transfers its cleanup responsibility.
Fallback cleanup discards the worker result or error; use `unwrap(join(worker))`
to collect the result and propagate failure. Cleanup never replaces an error
already leaving the scope. It can block indefinitely if the worker never finishes.

Captures still move owned data. Scoped workers cannot borrow caller storage, and
handles cannot be cloned, stored in collections or captured by another worker.
No async scheduler, pool or general-purpose `defer` syntax is introduced.
