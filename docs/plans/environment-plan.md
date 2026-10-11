# S09 environment lookup

The first environment API is read-only lookup. It exposes
`get(name: string): Result<string | undefined, EnvError>` from `std:env`.
`EnvError` contains `code` and `message` strings. Missing variables return
`Ok(undefined)`; present empty values return `Ok("")`. Successful strings own their
storage. The name is borrowed, and lookup does not mutate the process environment.

Empty names and names containing `=` or NUL return `InvalidName`. Values that cannot
be represented as Unicode return `NotUnicode`, without lossy replacement. Error
messages must not include environment values. Lookup follows the executable host's
name lookup rules, including platform case sensitivity. There is no compile-time
capture of environment values. Mutation is outside this batch; it needs a separate
contract compatible with native threads and foreign code.

## Backend acceptance

The private backend is implemented in `stdlib/env/runtime.rs`. Isolated subprocess
fixtures cover missing, empty, Unicode and non-Unicode values; invalid names; and
independent returned storage. Fixtures supply their own process environment and do
not call process-global mutation APIs from parallel Rust tests. Four focused tests
pass, including two child fixtures. Original failing and passing records are stored
as `tsrust-env-backend-*` in SQLite. This is backend acceptance only.

## Public implementation

The typed registry and generated bridge now expose `get` and `EnvError`. Public
native checks pass at O0/O3 for optional results, empty and Unicode values, missing
variables, invalid names, non-Unicode errors, aliases and stored function calls.
Wrong arity and argument types are rejected. The source name remains usable after
lookup. Both catalog examples execute at O0/O3; project editor checks verify hover,
parameter help and both import names. Original records use `tsrust-env-public-*`.
This is focused implementation evidence, not complete environment acceptance.

## Remaining acceptance

Matched workload `std_env_get` now cycles through controlled Unicode, empty and
missing variables. Each child gets an isolated environment override; every output
string is retained and compared with the independent exact-shape oracle. Runtime
and allocation drivers apply the same input and source guards. Ten untimed native /
Node-after-tsc cases pass, including 20,000 lookups. The initial missing fallible ABI
registration was repaired; original and repaired evidence use
`tsrust-env-workload-*` in SQLite. Timing remains pending S09.

All eight installed/workspace CLI and LSP consumers were atomically refreshed from
`1db46101` and hash verified. The installed CLI passes all ten matched untimed
cases. Installed LSP protocol checks report no diagnostics for the workload and
verify environment hover documentation. The active `tsrust_test` editor language
server was restarted and shows checking enabled with no Problems. Records use
`tsrust-env-installed-*` in SQLite. Environment lookup has focused and installed
acceptance; timing remains pending the S09 family milestone.
