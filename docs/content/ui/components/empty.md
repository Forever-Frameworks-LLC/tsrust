<!-- tsrust-docs
{"id":"component-empty","group":"components","order":190,"status":"partial","targets":["web","native"],"component":"empty","examples":["empty-projects","empty-media-states","empty-layouts","empty-theme"]}
-->
# Empty

Give an empty screen a clear next action. Compose media, a title, a description and
optional actions using six editable exports. Application code owns loading,
filtering, file picking and creation behavior.

## Install

Run `tsrust ui add empty --offline`, then import from `src/components/ui/empty`.
Compiler API 10 is required. The installer uses bundled source and shares its
license notice in `attributions/`. Existing files and application imports remain
yours to edit.

## Structure

`Empty` centers its contents with a full-width, flexible column. It uses dashed
border styling; add `border` to make the edge visible. `EmptyHeader` groups media
and copy. `EmptyContent` stacks actions. Both cap their width at `max-w-sm`:
24 times the enclosing theme's base type size, 384 logical pixels by default.

`EmptyTitle` and `EmptyDescription` use flowing Text content, defaulting to a div.
Choose a heading tag on the title to fit the surrounding document. Description
links are real Link components; direct links are underlined and use the primary
color on hover. Text selection and link hit regions share the native shaped layout.
Text slots accept text/inline content rather than arbitrary controls. Put controls
in EmptyContent. The container slots accept View replacement composition; text
slots use the existing Text contract without a replacement callback.

## Media variants

`EmptyMedia` uses the `empty-icon` slot. Omission selects `default`, which supplies
spacing and a transparent background. `icon` supplies a muted, rounded square and
foreground color. Explicit null removes variant styling and `data-variant`.
Constants and live variant/class values are accepted.

Descendant SVGs ignore pointer input and do not shrink. The icon variant supplies
small sizing when an SVG has no `size-` class; explicit icon sizes remain in the
caller’s control. The media example switches variant and icon size independently.
No network images, icon package or browser-only dependency is required.

## Theme and layout

All colors, spacing, type and radii come from the enclosing theme. Set
`typography.headingFamily` to `sans` (default), `serif` or `mono` for font-heading.
Named font files are unsupported. The custom example changes type, spacing and
heading family within an independent light/dark scope.

Tight tracking uses -0.025em. Descriptions use small type with a 1.625 line-height
ratio. Balanced wrapping improves short centered copy without changing its text
or ordinary line count. Native balancing covers 2–6 lines; longer native text uses
ordinary wrapping. Browser algorithms and font metrics can select different breaks.
The [CSS text draft](https://drafts.csswg.org/css-text-4/#valdef-text-wrap-style-balance)
permits different balancing strategies. Precise dash distribution around corners
also varies by renderer; border thickness, rounding and layout remain shared.

Caller classes override individual defaults. Examples include a project action,
live media variants, a responsive two-column layout and custom theme. Empty adds
no implicit status role or focus stop. Label meaningful media; decorative icons
remain excluded from accessibility. Announcements belong to the application.

## Scope

These exports port the pinned Base UI + Nova empty-state structure and styles to
portable primitives. React DOM event/ref props and text-slot replacement callbacks
are not implemented. The component adds no hidden upload, navigation or creation
policy. The checked renderer still rejects unsupported classes and ancestry.
