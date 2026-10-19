<!-- tsrust-docs
{"id":"component-textarea","group":"components","order":40,"status":"partial","targets":["web","native"],"component":"textarea","examples":["multiline-draft"]}
-->
# Textarea

Compose multiline text with a local editable component. Its value, change snapshots, disabled state and read-only state use the same checked contract as Input.

## Install and use

Run `tsrust ui add textarea --offline`. Import `Textarea` from your configured local component directory. Pair it with Label using an ID and `htmlFor`.

## Controlled and uncontrolled values

`value` controls the field; update application state in `onChange`. `defaultValue` initializes an uncontrolled field once. An explicit empty controlled value takes precedence over the default. Numeric inputs format as text without parsing user edits.

CRLF and CR normalize to LF. Other Unicode text remains intact. String `.length` counts UTF-8 bytes, not displayed characters. The example labels that count explicitly.

## Editing and accessibility

Enter inserts a newline. Selection, keyboard movement, undo/redo and composition use the target's text editor. Read-only fields retain focus and selection; disabled fields cannot edit or focus. `aria-invalid` exposes application validation without running it. Provide `aria-describedby` when explaining an error.

## Layout and customization

The local source uses content-based sizing with a minimum height. Add a maximum height to bound long drafts; the field scrolls within that bound. Theme tokens style its surface, border, focus and error states on both targets. Specialized rich-text editing and accessibility text-range navigation remain unsupported.
