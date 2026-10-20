<!-- tsrust-docs
{"id":"ui-disclosure-semantics","group":"foundations","order":156,"status":"partial","targets":["web","native"],"examples":["disclosure-semantics"]}
-->
# Disclosure state

A disclosure button describes whether its content is open. `aria-expanded` and
`aria-controls` update on the existing button in both browser and native apps.
The application controls visibility and retains its own open state.

## Independent panels

Each example panel reserves an ID with `useId()`. Its button forwards that ID
through the editable local Button component. Enter, Space or a pointer press
opens the panel. Closing removes it from layout, focus and accessibility while
retaining its draft. Opening again restores the original input and value.

`aria-expanded` accepts a boolean or live nullable boolean. False means closed;
undefined removes the explicit expansion state. `aria-controls` accepts a
space-separated ID string or live nullable string. Empty or undefined removes
the relationship. IDs with control characters other than ASCII whitespace reject;
ASCII whitespace is normalized. Native accessibility ignores missing targets.
The example removes controls while its retained panel is hidden, so visible
relationships always resolve to currently presented content.

## Stable IDs

Call `useId()` during component construction, then keep the returned string.
Every call reserves a different ID within the runtime, including native threads.
The ID survives state changes because the component is retained. Separate browser
Wasm modules or native processes have separate counters. IDs are not persistent
keys, secrets or predictable data for application storage. Counter exhaustion
throws instead of wrapping. The returned string is owned; clone explicitly when
separate consumers need it.

## Activation and scope

Native accessibility exposes expanded state, controlled targets and the applicable
Expand or Collapse action. A request for the current state does nothing. Disabled
or hidden controls cannot activate. Ordinary button keyboard behavior is unchanged.
These metadata props do not create visibility state, move focus or install a handler.

This example follows the [WAI disclosure pattern](https://www.w3.org/WAI/ARIA/apg/patterns/disclosure/).
It is an application composition example, not the complete Base UI Collapsible API.
Scoped context, event/ref merging, focus restoration and transitions remain required
for the installable component. Compiler API 9 is required.
