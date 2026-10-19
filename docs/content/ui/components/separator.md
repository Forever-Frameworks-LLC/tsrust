<!-- tsrust-docs
{"id":"component-separator","group":"components","order":70,"status":"partial","targets":["web","native"],"component":"separator","examples":["section-dividers"]}
-->
# Separator

Divide adjacent sections with an accessible, nonfocusable line.

## Install and orient

Run `tsrust ui add separator --offline`. Import `Separator` from the installed local file. Omitted `orientation` means `horizontal`; `vertical` selects a vertical line and matching accessibility metadata. Separator accepts no children.

## Layout and accessibility

A horizontal separator fills its available width. A vertical separator stretches within its parent's height; give the row a height when it has no other stretching content. The separator does not add an extra keyboard stop. Use headings to name sections; a separator alone does not describe their contents.

## Customization and ownership

The border theme token controls line color in both modes. Caller classes override checked dimensions and colors. No state or callback is allocated. Decorative separator semantics and arbitrary ARIA props are not implemented in this portable component.
