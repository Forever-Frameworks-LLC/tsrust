# Computed UI text and exact numeric children

This prerequisite keeps application expressions in ordinary JSX. Arithmetic,
string operations, scalar conditionals and application top-level/imported helpers with declared
scalar return types can read directly declared component state. Their text updates
after event handlers through the existing text run. Text, View and Button share
the same declared child contracts. The compiler retains a private getter with weak
state captures; applications neither attach handles nor select another component.

Literal and array numeric children preserve their checked scalar types. Integers
format directly from all 64 bits rather than passing through a floating-point
number. Numbers use ordinary number formatting, including negative zero as zero.
Null and undefined getter results contribute no characters. Existing direct
boolean state bindings format true/false; bare boolean children remain errors.

Computed getters must be pure. Construction and refresh may evaluate them more
than once. Application errors propagate without being replaced by a disposed-state
message. A browser mount that encounters an error releases its tree and stops
handling events; independent mounts keep working. Callback captures still follow
the ordinary owned reusable callback rules. Binding values cannot be fabricated,
cloned or sent between tasks, and no public binding factory is provided.

`examples/ui-computed-text` contains the native inspector, an imported formatting
helper, a paired interactive page and independent failed/healthy mounts. The
untimed observers compare actual Wasm browser output with a Node-after-tsc oracle,
including exact integers, derived strings, nullable text, unchanged control
identities, accessible names, preserved errors and disposal.

Focused acceptance passes on 2026-10-19: two new and seven affected compiler
witnesses; computed, nullable and scalar editor help; three affected prop receiver
agreements; two new runtime witnesses and two affected scalar runtime witnesses.
Fresh offline locked Wasm builds pass both Chromium observers against the
independent Node-after-tsc oracle. The error observer preserves the original
application error, updates an independent healthy mount and verifies disposal.
Whole-project formatting proves token preservation/idempotence, and formatted
native examples execute with their original assertions.

Original failures and repairs use `tsrust-ui-computed-*` in canonical SQLite.
They include the callback forwarding repair, which now uses ordinary callback
argument checking and supports concrete closure identities, and the test offset
repair after formatting. Temporary diagnostic instrumentation was removed.
No full regression, SQL run, performance timing or new GPU smoke was repeated;
previous unchanged renderer evidence remains applicable.

This is a bounded scalar-expression prerequisite, not general reactive props.
Precomputed local values remain snapshots. Inferred helper returns, generic child
contracts and state reached through arbitrary prop records are not made reactive.
Dynamic node mounting, keyed lists and the remaining Nova component/layout/block
families remain required work. Integrated CLI web build/dev also remains pending;
browser output currently uses `tools/build-ui-web.mjs`.
