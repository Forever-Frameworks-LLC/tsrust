<!-- tsrust-docs
{"id":"ui-vector-graphics","group":"foundations","order":90,"status":"partial","targets":["web","native"],"examples":["vector-icons","vector-chart","vector-viewports"]}
-->
# Vector graphics

Draw icons and data with shared `Svg` and `Path` primitives. The browser uses SVG elements; native uses Vello geometry. Paths belong to the application tree and use the same checked state and cleanup rules as other components.

## Icons are local components

Write an icon function beside the component that uses it. Set `fill="none"`, `stroke="currentColor"` and the cap/join style you need. Size and color come from ordinary classes. No package manager, script bridge or remote image is involved.

An unlabelled Svg is decorative by default and does not change a button's name. Keep the action in the button's application handler. Give an icon-only button its own accessible label; a meaningful standalone image needs `aria-label` on Svg.

## Live data

`Path.d` accepts ordinary reactive expressions. Changing it updates the retained path and its parsed geometry. An empty string paints nothing while keeping identity. Invalid updates report an error and keep the last valid path data.

The chart example plots six daily build counts. Its accessible image name stays stable, while `aria-describedby` points to the current text values. A chart still needs useful words and data; color and shape alone are insufficient. This is a drawing foundation, not a completed chart component or tooltip system.

## Viewports

`viewBox` describes the coordinate system as x, y, width and height. Width/height must be positive; negative origins are allowed. Preferred viewport dimensions default to 24px and can be overridden with width/height utilities.

The default `xMidYMid meet` scales uniformly and centers the complete drawing. `slice` fills the viewport and may crop. All nine xMin/Mid/Max and YMin/Mid/Max alignments work. `none` stretches the axes independently. Strokes scale with geometry. The viewport clips by default; `overflow-visible` permits paint outside it.

## Paint and themes

Fill defaults to `currentColor`; stroke defaults to `none`. Paths inherit paint from Svg and can override it. Acceptable paint values are `none`, `currentColor`, semantic token names, `#RRGGBB`, `#RRGGBBAA` and checked in-gamut `oklch()` colors. Theme scopes belong on Svg or an enclosing View. Path classes support base text colors and opacity; put layout and interaction styles on Svg.

Stroke width is a finite nonnegative number in viewBox units. Caps are butt, round or square; joins are miter, round or bevel. Miter limit defaults to 4 and must be at least 1. Fill rule is nonzero or evenodd. NaN/infinite values throw before serialization can turn them into absent options.

## Boundaries

This section requires a compiler with API 6. Source path strings remain unchanged. Parsed geometry, viewport transforms and native paint must fit native floating precision; values that overflow that boundary reject. Rendering is approximate: native GPU and browser rasterization need not produce identical pixels. Extremely small geometry can fall below pixel resolution. Intrinsic auto/fit sizing is not implemented and rejects; set explicit, fractional or viewport dimensions.

Path data supports SVG M/L/H/V/C/S/Q/T/A/Z commands in absolute and relative forms. The limit is 1048576 UTF-8 bytes and 100000 source segments or resulting geometry elements per path. Nonzero arc radii must exceed 0.00001 viewBox units; zero radii draw a line. Smaller nonzero radii reject instead of silently becoming lines. Unknown commands, malformed numbers, nonfinite geometry and external paint URLs reject. Svg accepts direct Path children only; use live empty path data for conditional geometry. Inline Svg inside Text, arbitrary SVG elements, nested SVG, the defer aspect flag, masks, gradients, filters, non-scaling strokes and path-specific interaction policy are not implemented. Context, focus/overlays and the remaining component library remain separate work.
