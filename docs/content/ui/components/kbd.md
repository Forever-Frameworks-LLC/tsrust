<!-- tsrust-docs
{"id":"component-kbd","group":"components","order":60,"status":"partial","targets":["web","native"],"component":"kbd","examples":["keyboard-shortcuts"]}
-->
# Kbd

Display keyboard keys and combinations. Kbd is visual and semantic content; it does not register a shortcut.

## Install and compose

Run `tsrust ui add kbd --offline`. Import `Kbd` and `KbdGroup` from the same local file. Put individual keys in Kbd and use KbdGroup to arrange combinations in source order. Write the platform-appropriate key names in application code.

## Accessibility and behavior

Keycaps remain readable text with keyboard semantics. They do not receive focus, intercept input or open a command palette. The example intentionally displays shortcuts without pretending to implement those actions. A custom interactive control must supply its own checked handler.

## Customization and ownership

Muted surface and foreground tokens follow the active theme. Caller classes override supported properties; unsupported utilities fail. Children transfer once. Icon glyph assets and automatic platform-key substitution are not implemented.
