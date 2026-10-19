<!-- tsrust-docs
{"id":"component-label","group":"components","order":50,"status":"partial","targets":["web","native"],"component":"label","examples":["field-labels"]}
-->
# Label

Associate visible text with an editable control. Clicking a label focuses its enabled control on both targets.

## Install and associate

Run `tsrust ui add label --offline` and import `Label` from your local component directory. Set `htmlFor` to an existing control ID, or wrap exactly one control. IDs must be unique and cannot contain ASCII whitespace or control characters. An invalid association fails instead of silently labeling another control.

## Accessibility and states

Placeholder text does not replace a label. Disabled and read-only states belong to the control. A disabled control remains named but cannot receive focus. Do not place unrelated interactive descendants inside a label.

## Customization and ownership

Caller classes override local typography and layout. The component consumes its children once and forwards shared metadata. It adds no validation, focus owner, event subscription or application state. Compound Field validation and arbitrary HTML label behavior are not implemented.
