# S09 environment lookup

The first environment API is read-only lookup. It will expose
`get(name: string): Result<string | undefined, EnvError>` from `std:env`.
`EnvError` will contain `code` and `message` strings. Missing variables return
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

## Remaining acceptance

- Register `EnvError` and `get` in the typed native registry and generated bridge.
- Add public native O0/O3 fixtures, alias/function-value and diagnostic checks.
- Complete catalog documentation, verified examples, import completion, hover and
  parameter help.
- Register matched Node-after-tsc lookup support with controlled environment inputs,
  an independent result oracle and source guards; run representative untimed cases.
- Refresh installed consumers after public acceptance. Keep timing pending S09.
