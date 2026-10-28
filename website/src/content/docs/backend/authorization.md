---
title: Configure API authorization and dispatch
description: Keep access policy on the server and expose only checked operations.
---

In tsrust.config.json api settings, declare the private context and access policy.
The compiler connects checked operations to the native Router in memory.

```json
{
  "server": {
    "context": { "module": "app/context.tsr", "export": "TaskContext" },
    "authorize": { "module": "app/access.tsr", "export": "authorize" },
    "compatible": ["contracts/v1.json"],
    "maxRequestBytes": 65536,
    "maxResponseBytes": 65536
  }
}
```

Add this server member to the project API settings, and add `"context": true`
only to operations whose first parameter is a borrowed native HTTP Context.
That parameter is private; the remaining zero/one plain-data argument is the wire
input. An arbitrary pool/resource or an owned Context cannot be hidden this way.
Context roles do not alter the public contract identity. Application state and
policy stay in backend modules. An authorize hook must return `Task<boolean>`
from the configured Context; it runs before request decoding or operation calls.
Returning true is the application's explicit access decision. Configure Router
body/concurrency/time limits too: the generated decoder's post-read size guard
does not replace HTTP admission limits.

Import dispatch, describe and openapi from `api:server`, then register them on the
application Router. Checking, building and dev create no dispatcher source file
and read no client output directory. Operation facts and executable handlers share
the checked source graph; inconsistent independently loaded inputs fail with a
retry diagnostic. Unreadable nearest project configuration fails instead of
selecting a parent project's access policy. Dispatch requires POST, JSON MIME, a declared operation and an
accepted contract hash. Schemas derive from the checked functions. Generic
materialization uses ordinary checked specialization, never an assertion of
unknown JSON as a DTO. Invalid arguments/results, policy denial and backend
exceptions produce bounded public error messages. Typed domain-result unions
remain ordinary successful data. Neither failure nor cancellation asserts rollback.

### Explicit legacy source export

Existing projects that require a physical dispatcher can opt into server.output,
such as app/generated-api, when explicitly running tsrust api generate. This
retained export mode is independent of normal api:server compilation and is not
used by new backend scaffolds.

The private output must be separate from the public package. Generation checks
the complete virtual handler before replacing outputs, pins checked input/config
versions and preflights ownership for both destinations. Both destinations have
independent generated hash markers/locks; edits are preserved. Replacements are
individually atomic and retryable, not a filesystem-wide transaction. `--check`
writes nothing. An explicit `--out` changes the client destination while a configured
private server keeps its own destination. Omit `server` for client-only adoption.

Retained baselines check that old inputs fit the new operation and new outputs
can be projected into the old strict shape, recursively. Versioned SDK history
also checks the reverse result direction for old replicas. Unknown operation
shapes reject with api-incompatible before invocation, even when their names
exist. Keep immutable contract versions rather than changing an old policy input
to describe a different client. Neither hash matching nor containment establishes
unchanged application semantics.

The HTTP encoder retains platform JSON escaping and marks unsafe integral JS
numbers as decimal floating-point tokens. Thus finite values such as 1e20 do not
overflow native bare-integer JSON storage. Numeric text in strings/keys stays
unchanged. Safe integer DTOs retain their stricter limit.
