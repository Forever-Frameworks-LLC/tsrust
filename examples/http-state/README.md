# Application and request state

`Router<State, Variables>` owns application state once. Handlers borrow it through
`c.env`. Use this for configuration and supported concurrent pools, clients,
shared caches and senders. Application state cannot be moved or mutated through
Context; concurrent handles expose their existing checked operations.

The `Variables` record declares literal keys for `c.set` and `c.get`. Each request
starts empty. Setting consumes a correctly typed value, replacing and releasing
any previous value. Getting returns undefined when unset; non-scalar values are
read-only loans. Explicit cloning produces independent data where supported.
Unknown keys, wrong types, caller loans, local resources and task handles reject.

`router.tsr` checks shared configuration, middleware-to-handler state and isolation
between requests. It also exercises cookie helpers and response status after next(). It contains no authentication policy. This section is still
under implementation and has not completed its targeted acceptance batch.

`main.tsr` retains the accepted App closure-capture example, including concurrent
requests and retained state. `sql.tsr` retains the opt-in SQL pool witness.
