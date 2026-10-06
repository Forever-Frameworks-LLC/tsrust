# Milestone 10: callable-valued arrow results

[Callable captures](./09-callable-captures.md) · [Roadmap](../roadmap.md)

**Status:** Complete.

Evidence: six groups in [callable result tests](../../crates/tsrust-compiler/tests/callable_results.rs); at completion, all 139 regression groups and Clippy passed.

Allow recursive callable result annotations on ordinary functions and arrow bodies. Returned environments own their captures, including captures from an enclosing arrow invocation. Classify named and direct returns before capture creation; propagate resulting transfers to the enclosing invocation or fresh parameter. Each callable result must resolve to one concrete environment origin. Reject borrowed escapes and mixed origins rather than inserting runtime unions or implicit allocation.

Callback parameter signatures continue to have non-callable result types. Higher-order parameter types, aliases, and borrowed returns remain outside this gate. Bind a returned callable before invoking it, as with existing factory results.

Exit gates: bounded recursive annotation parsing; exact recursive result/access matching; recursive escape classification; native output and ownership diagnostics; regressions, formatting, Clippy, and documentation.
