<!-- tsrust-docs
{"id":"component-input","group":"components","order":30,"status":"partial","targets":["web","native"],"component":"input","examples":["input-states"]}
-->
# Input

The current portable Input is a single-line text field with controlled/uncontrolled values, checked change snapshots and native text editing. It is a partial Nova implementation.

## Install and import

Run `tsrust ui add input --offline`. Import the exports from your configured local
component directory. The installed source belongs to the project; theme tokens and
caller classes customize the same component on web and native.

## Values

`defaultValue` initializes an uncontrolled field. An explicit `value` takes precedence, including an empty string or zero. Direct supported signals and local-state value expressions update controlled fields after handlers.

CR/LF characters are removed for the single-line field. Other Unicode text remains intact. Numeric values format as text; the field does not parse typed numbers automatically.

## Changes and states

`onChange` receives borrowed `target` and `currentTarget` snapshots containing value, name and id. Application code decides validation and updates. A disabled field cannot edit or focus. A read-only field retains selection/focus and cannot change text.

Native editing includes Unicode selection, IME, undo/redo, scrolling and checked OS clipboard operations. Browser composition is retained through ordinary updates.

## Accessibility and limits

Use a visible Label with `htmlFor`, or an appropriate accessible name. Placeholder text is not a substitute for a label.

Only `type="text"` is supported in this slice. Password, file, date-picker and other specialized input types reject. Accessibility text-range navigation and complete Field composition remain pending.
