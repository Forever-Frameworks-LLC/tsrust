# Milestone 19: direct owned-callback returns

[Callback moves](./18-callback-moves.md) · [Callable results](./10-callable-results.md) · [Roadmap](../roadmap.md)

**Status:** Complete. The full 237-group regression suite passes, including 10 callback return groups. Formatting, Clippy, and local documentation links are verified.

## Direct ownership transfer

Functions and arrows can return an `owned` or `owned once` callback directly through a matching callable result annotation. This includes local move chains and callbacks captured by an arrow. Returning transfers ownership; the source cannot be used again on that path.

```typescript
function identity(action: owned () => int): () => int {
  return action;
}
function counter(): () => int {
  let count = 0;
  return identity((): int => { count = count + 1; return count; });
}
function main() {
  const action = counter();
  console.log(action());
  console.log(action());
}
```

This prints `1` and `2`. The returned environment retains its state after both functions exit. Reusable owned returns keep the original concrete environment type. No clone, type erasure, or heap allocation is introduced by returning it.

Result annotations require exact argument types, argument access, arity, and result type. Template bodies are validated even when unused. Non-owned reusable and ordinary `once` parameters still cannot escape; direct returns from those contracts report `callback-escape`. Passing a borrowed environment into an owned parameter also remains rejected.

## Single-use return contracts

Returning `owned once` never restores repeated invocation, even when the original concrete environment is reusable. If the original environment is already single-use, it returns unchanged. Otherwise, the compiler uses a cached concrete wrapper containing the original environment by value. Its invocation consumes the wrapper and delegates through `OnceCallback`.

This wrapper has no allocation, shared pointer, runtime dispatch, or copied capture values. Its type is cached by original concrete environment, so repeated calls and separate forwarding functions retain stable replacement identity. The original closure body is not duplicated. Closure metadata exposes the wrapper's consuming invocation and owned capture.

A returned single-use value remains single-use through local moves, captures, callback forwarding, branch joins, and loop checks. It cannot satisfy a reusable callback parameter. Consuming the callback before returning it reports `callback-consumed`.

## Origins, templates, and nested results

Abstract template analysis records the originating callback parameter as a callable value with the declared invocation and argument contracts. Calls substitute the caller's argument origin through direct forwarding chains. These symbolic descriptors are not emitted as runtime environments. Once specialized, reusable returns use concrete environments and restricted single-use returns use the cached wrapper where needed.

A returned callable value can be passed onward under any existing compatible callback contract. Its former parameter name does not force parameter-to-parameter forwarding rules onto the returned value. Invocation, ownership, capture, and argument-access constraints still apply.

Returning through multiple branches is allowed when all returns retain the same origin. Returning different callback parameters remains unsupported even if signatures match, because separate generic parameters can represent different concrete environments. Loops retain the existing zero-iteration and transfer rules; returning from a loop exits the function instead of reaching a back edge.

An arrow that returns a captured owned callback consumes that capture and is single-use. The returned inner callback independently retains its reusable or single-use behavior. Recursive callable result annotations continue to describe these nested factories.

## Evidence and remaining scope

The [callback return suite](../../crates/tsrust-compiler/tests/callback_returns.rs) checks direct state retention, caller transfers, read/mutate/consume arguments, scalar and void results, exact signature errors, single-use restrictions, declaration chains, branches, loop exits, template forwarding, returned-value callback arguments, nested arrows, captures, replacement, cache determinism, unused-template validation, and backend-independent analysis. Accepted programs compile with the pinned backend and compare native output.

The [identity counter fixture](../../crates/tsrust-compiler/tests/fixtures/callback-returns/identity.tsr) returns owned callbacks directly and prints `1`, `2`, and `1` from independent counters. Earlier callback suites now accept owned direct returns while retaining non-owned escape checks.

Mixed environment types, shared aliases, non-owned callback escapes, callable fields/globals, signature-only callable variable annotations, callable-valued callback arguments/results, borrowed callable returns, collections, async, modules, and the language server remain separate scopes.
