# B7 caller-borrowed record stores: acceptance map

Date: 2026-10-10. Ledger row: B7 (gate 183) in `docs/compiler-completion-plan.md`.
Plan: `docs/caller-borrowed-record-stores-plan.md`.

A callback installed through a mutable record parameter may borrow an input that the
caller owns. This map lists every form in the plan's acceptance sequence, its status
and its evidence. Three statuses are used:

- **Supported**: runs natively and matches the Node reference.
- **Conservative**: accepted and correct, but a capture stays borrowed longer than
  strictly necessary. Precision can improve later without breaking programs.
- **Diagnosed**: rejected with a source diagnostic. Not supported.

No listed form reaches "Rust rejected generated code" or a compiler panic.

## Supported

Tests are in `crates/tsrust-compiler/tests/` unless a path is given.

| Form | Evidence |
| --- | --- |
| Readonly and mutable data inputs, readonly aliases, record inputs | `caller_borrowed_record_stores.rs` |
| Nested record destinations, two destinations, several captures in one callback | `caller_borrowed_record_stores.rs`, `caller_store_boundaries.rs` |
| Named relays, reordered parameters, recursive and mutually recursive relays | `caller_borrowed_record_stores.rs`, `caller_store_boundaries.rs` |
| Borrowed callback inputs captured or stored directly, readonly and mutable | `caller_borrowed_callback_replacement.rs` |
| Consuming callback inputs, single use | `caller_borrowed_callback_replacement.rs` |
| Nullable callback fields: install, replace, clear, reuse of the cleared input | `nullable_caller_callback_stores.rs` |
| Replacement with a different input releases the old input (plain and nullable fields, equal callback types, independent factory instances) | `caller_input_instance_replacement.rs` |
| Conditional callback stores, saved local callback families | `caller_input_instance_replacement.rs` |
| Stores in branches, loops and after early returns | `caller_store_boundaries.rs` |
| Stores inside `try`, `catch` and `finally`; failures before and after a store; typed failure of the installed callback | `caller_callback_store_failures.rs`, `caller_store_boundaries.rs` |
| Named installer saved with `const` and called through that name | `caller_input_instance_replacement.rs` |
| Installer closures: called directly, capturing effect state, capturing a readonly source, nested construction | `caller_input_instance_replacement.rs` |
| Installer closure called from a closure that captured it, through one to three layers; two sources through one receiver | `caller_input_instance_replacement.rs` |
| Reusable installer that captures a mutable source, with clearing between calls | `caller_input_instance_replacement.rs` |
| Owned transfer through a `Once` installer into an owning callback field | `caller_input_instance_replacement.rs` |
| Disposal: the input is free after the holder's scope ends | `caller_store_boundaries.rs` |

## Conservative

| Form | Behavior |
| --- | --- |
| A store that may not happen (early return, one branch only) | The earlier input and the new input both stay borrowed |
| A relay that installs twice (`install(h, a); install(h, b);`) | Both inputs stay borrowed although only the last is held |
| A recursive relay followed by mutation of an earlier input | The earlier input stays borrowed |
| An installer with default parameters or exception syntax | Replacement does not release the earlier input |
| A nullable callback *record* field (`saved: Holder \| undefined`) stored or cleared through a parameter | The caller's captures stay borrowed (B5 rule) |

## Diagnosed

| Form | Code |
| --- | --- |
| Callback that borrows a local of the installer | `callback-escape` |
| Input with a shorter life than the holder | `callback-escape` |
| Input that is an indexed element, a call result or another temporary | `callback-escape` |
| Mutation or move of an input while the holder is live | `capture-conflict` |
| Installer closure passed as a callback, aliased or converted to another contract | `unsupported` |
| Named installer passed as a callback argument | `unsupported` |
| Closure that stores a borrowing callback into a record it captured, by call or by assignment | `unsupported` |
| Closure that calls a captured installer whose stored callback mutates a variable the installer captured | `unsupported` |
| Recursive call that wraps its own borrowed callback in a new closure | `callback-escape` |
| Direct replacement of a local nullable borrowing callback field | `unsupported` |

## Possible later work

None of this blocks B7. Each item would turn a diagnosed or conservative row into a
supported one.

- Abstract installer invocation: a callback contract that carries a record view and
  replacement obligations.
- Stores into captured records.
- A held installer with a mutable captured source (needs the receiver reborrow
  lifetime in `projected_capture_lifetime`).
- Precise release for conditional stores and recursive relays.
