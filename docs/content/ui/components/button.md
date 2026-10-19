<!-- tsrust-docs
{"id":"component-button","group":"components","order":10,"status":"partial","targets":["web","native"],"component":"button","examples":["counter"]}
-->
# Button

The existing editable Button provides a press callback, checked disabled conditions and a small set of project-themed variants. It is not yet the complete pinned Base UI + Nova Button.

## Install and import

```text
tsrust ui add button --offline
import { Button } from '../components/ui/button';
```

## Behavior

`onPress` runs application logic. `disabled` prevents activation and focus; a supported local-state JSX condition updates after handlers. Text is centered and the native focus outline follows the rounded shape.

The current variants are `default`, `secondary`, `outline` and `destructive`. Sizes are `sm`, `default` and `lg`. Omission selects `default`. Edit the local source or provide checked `className` utilities to customize it.

## Composition and accessibility

Children can contain noninteractive phrasing text and owned inline content. Nested buttons, editable fields and focusable containers reject. Buttons keep their implicit accessibility role and ordinary Enter/Space activation.

The current local Button wrapper does not forward every inherited metadata field to its primitive. Native primitive naming and identity support do not establish wrapper parity. That forwarding gap remains part of completing this component.

## Ownership

The component consumes its props and owned children once. Press callbacks follow the same checked capture rules as the primitive. Application arithmetic belongs in the callback or an imported application helper.
