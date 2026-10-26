# Examples

Use `tsrust run <path>` from the repository root unless an example's README gives
project-specific setup. Native execution requires the pinned Rust backend.

| Build | Start here |
| --- | --- |
| A console project | `tsrust new hello`, then `cd hello && tsrust run` |
| Tests and assertions | [testing/](testing/) |
| HTTP routing and request state | [http-context/](http-context/) and [http-state/](http-state/) |
| An integrated service | [service/](service/) |
| SQL values and migrations | [sql-domain-values/](sql-domain-values/) and [sql-migrations/](sql-migrations/) |
| A cache or queue | [cache/](cache/) and [work-queue/](work-queue/) |
| Rust bindings | [rust-data-boundaries/](rust-data-boundaries/) and [rust-regex/](rust-regex/) |
| A reactive UI | [ui-application/](ui-application/) and [ui-composition/](ui-composition/) |
| The shared docs application | [../docs/](../docs/) |

Examples demonstrate scoped supported behavior. They are not blanket compatibility
claims for TypeScript, Hono or shadcn. Some require databases, a graphical desktop,
Wasm tooling or explicit environment variables; read their setup before running.
