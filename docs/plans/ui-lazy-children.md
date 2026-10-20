# Lazy child composition

Continue after accepted shared-vector checkpoint `7b82bcfa`. Compound components
need to establish their scope before constructing children, and create fresh owned
content after an unmount. Eager Child[] forwarding alone cannot provide that.

## Contract

- Add `LazyChildren = Owned<Function<Child[]>>`; it is an ordinary checked callback,
  with no new public handle or lifetime constructor. Component authors call or
  forward it; application JSX remains ordinary nested tags.
- For declared function components with a children field that is an owned,
  zero-argument array factory, lower between-tag children into that factory.
  Preserve ordinary eager Child[] behavior and explicit children={factory}.
- Keep attribute order, deferred child order, checked reusable/once captures,
  callback failure cleanup and existing weak state captures. Do not cache the
  factory result implicitly. Support optional and inherited/generic props.
- Use one paired docs example with deferred stateful content, parent state
  retention and repeated close/reopen behavior. Use std:testing resource observers
  in generated-native fixtures for independent cleanup evidence.
- Keep completion, hover, parameter help, diagnostics and exact examples current.
  Prepare independent Node-after-tsc transition evidence before focused validation.

## Acceptance

- [x] Compiler lowering, public type, ownership/order/failure witnesses.
- [x] Paired deferred-content example and native runtime integration.
- [x] Editor agreement, typing scenarios, hover and documented example.
- [x] Independent reference, batched targeted checks, consumers/preview and SQLite.

This does not by itself complete context providers, event/ref merging, focus or
compound controls. Complete those contracts before marking dependent components
as Base UI/Nova parity. No full regression or timing pass is triggered here.
