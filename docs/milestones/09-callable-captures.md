# Milestone 09: concrete callable captures

[Previous gate](./08-nested-local-closures.md) · [Roadmap](../roadmap.md)

**Status:** Complete. All five exit gates pass.

Allow closures to invoke and synchronously forward captured concrete closure bindings. Infer read, mutate, or consume access to the captured environment independently of that environment's own capture storage. Existing block loans apply to callable bindings, including after their last invocation.

Owned returned wrappers may capture fully owned concrete environments. Reject an escaping wrapper that would retain any borrowed environment, including a borrow nested inside an owned field. Do not insert cloning, boxing, or reference counting. Ordinary callback parameters retain their non-escape contract until the retained-callback gate.

Exit gates: capture inference and transfers; transitive borrowed-environment checks; lifetime-correct concrete Rust fields; accepted native-output and rejected diagnostic tests; full regression, formatting, Clippy, and documentation checks.

The six groups in [callable capture tests](../../crates/tsrust-compiler/tests/callable_captures.rs) compile native output and verify transfer/loan diagnostics. At completion, all 133 regression groups and Clippy passed. Separate generated lifetimes prevent a mutable borrow of a callable from inheriting its environment lifetime. Environment lifetime complexity is bounded at 128.
