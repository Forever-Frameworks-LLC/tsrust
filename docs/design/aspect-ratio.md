<!-- tsrust-docs
{"id":"component-aspect-ratio","group":"components","order":80,"status":"partial","targets":["web","native"],"component":"aspect-ratio","examples":["aspect-ratio-layout"]}
-->
# Aspect Ratio

Reserve a preferred width and height relationship for content. A widescreen area
uses `16.0 / 9.0`; a square uses `1.0`. Caller classes can override the ratio.

## Install and import

Run `tsrust ui add aspect-ratio --offline`. Import `AspectRatio` from your installed
local component file. Source and theme stay editable in your project.

## Sizing

`ratio` is required and has no default. It means width divided by height. Values
must be finite, positive and within the native layout range. Both renderers use
the same normalized f32 ratio and ordinary logical-pixel rounding. Zero, negative,
nonfinite and native range overflow/underflow ratios fail.

The box fills the available width and does not shrink in a vertical layout. Width utilities override that default. Provide a definite available width. Automatic height follows width and ratio. Explicit
width and height take precedence; min/max dimensions constrain the result. Children
can require more space or overflow. Set explicit bounds and overflow utilities when
clipping is intended.

## Customization and accessibility

`aspect-square`, `aspect-video`, `aspect-[3/4]`, `aspect-[1.5]` and `aspect-auto`
use shared layout rules. Caller classes override the component defaults. Semantic
content and controls keep their source and keyboard order. AspectRatio adds no
focus stop or implicit accessible name; name the content it contains.

## Ownership and limits

Props and children transfer once. The ratio is applied at construction;
reactive ratio updates are unsupported. Image loading/fitting,
absolute/sticky offsets, intrinsic-width cross-axis sizing and container queries are separate unsupported capabilities.
