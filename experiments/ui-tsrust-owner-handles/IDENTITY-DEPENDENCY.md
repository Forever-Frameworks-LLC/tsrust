# Identity factory compiler dependency

The identity assignment exposed a compiler defect before the identity candidate
could be evaluated. The owner authorized coordinated dependency repairs and merges;
the compiler worker explicitly assigned this repair and released the shared target.
The UI experiment itself remains a separate, unmerged task.

## Reproducer and cause

On source `b255009e366357858ab964e486353dd2e5ff5bb9`, CLI SHA-256
`3551e7be8a852c9222782327e4036ad91f2311784654a94f74890bc31195db96`,
importing this factory fails strict check and emit with `error[name]: Unknown value
"count"` at `const id = count`:

```tsrust
let count: int = 1;
export function make(): () => int {
  const id = count;
  count++;
  return (): int => { return id; };
}
```

A scalar-returning function reading and incrementing the same private module
variable checks, emits and executes successfully. The returned closure above owns
only the scalar copy; it does not borrow module storage.

Callable-result discovery ran before named capture contexts were prepared. It now
waits for the defining context. A factory discovered during this pass can prepare
another factory's captures, so the existing worklist also admits those newly ready
functions. Normal body scheduling retries deferred declarations. Unresolved
recursive origins still reject; no module effect or ownership rule is relaxed.

## Acceptance

Focused compiler suites cover module state, callable results, recursive callable
results, closures and the original B4 handler specialization. Added witnesses cover
independent owned results, transitive factories, diamond initialization, local
capturing factories, live module effects, forbidden borrow escape, early reads and
unused unresolved recursive factories. Successful fixtures exercise formatter
idempotence and native assertions. The project-aware editor witness verifies checked
hover and signature help for the imported factory and returned callable.

Exact commands, outputs, initial failure and passing control are retained in this
experiment's ignored `evidence.sqlite`, under `identity-dependency/`. Whole-project
formatting uses the verified installed CLI and a read-only resolver for the main
checkout's existing TypeScript scanner. No dependency installation or consumer
refresh occurs. No performance claim is made.

This repair supplies existing language behavior needed by the identity candidate;
it is not an identity primitive and establishes no UI resource-release guarantee.
