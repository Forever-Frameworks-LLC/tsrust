<!-- tsrust-docs
{"id":"component-kbd","group":"components","order":60,"status":"partial","targets":["web","native"],"component":"kbd","examples":["keyboard-shortcuts","keyboard-customization","keyboard-icons-theme"]}
-->
# Kbd

Display a key or shortcut with `Kbd` and `KbdGroup`. They render keyboard text;
application handlers implement the shortcut itself.

## Installation

```sh
tsrust ui add kbd --offline
```

Import both components from the installed local file. An upgrade preserves the
existing `kbd.tsrx` and application imports, then adds `kbd-v2.tsrx`. New projects
receive `kbd.tsrx` directly.

## Composition

Wrap each key in `Kbd`. Use `KbdGroup` for combinations, with text separators where
needed. Both retain the upstream `kbd` tag and their `kbd` / `kbd-group` slots.
Choose platform-appropriate key names in application code.

Keycaps work inside a container or Button. A Button remains the keyboard target;
keycaps do not take focus or intercept pointer input. Do not place boxed keycaps
inside Text paragraphs: shared inline text currently accepts typography, not
inline layout boxes.

## Props

| Prop | Behavior |
| --- | --- |
| `children` | Owned text, icons and supported child nodes, in source order. |
| `className` | Supported utility classes or a reactive string. Caller classes override defaults. |
| `id`, `aria-label`, `aria-describedby`, `aria-hidden` | Forward the supported shared semantic metadata. |
| `data-slot` | Overrides the default slot. Changing it also changes which ancestor/descendant slot rules match. |

`KbdProps` and `KbdGroupProps` inherit the portable View contract. They do not
provide arbitrary browser events, DOM refs or the full Base UI event/ref API.
There are no variant or size props; customize those with classes.

## Customization

`Kbd` uses muted surface/foreground tokens, small radius, sans text and compact
spacing. Class bindings update the retained keycap; they do not recreate it.
`KbdGroup` forwards live classes independently. Custom themes change tokens,
radius and numeric spacing within a scoped example. Font utilities select a
generic family; a theme field for the keycap body font is not supported.

An SVG without a `size-*` class receives `size-3`; an explicitly sized SVG keeps
its own size. Under a `tooltip-content` ancestor, the keycap switches to the
upstream translucent background and contrast text. The ancestor example exercises
that styling relationship; it does not install a Tooltip widget.

## Accessibility and ownership

Use readable key text. Label an icon-only SVG when its meaning is unclear.
A keycap needs a naming role, such as `role="group"`, before it can receive
`aria-label`; a bare keyboard text container cannot be named.
`aria-hidden` removes decorative duplicates from accessibility. Keycaps do not
install keyboard listeners or open a command palette. Component props, children
and evaluator captures transfer once and release with the subtree.
