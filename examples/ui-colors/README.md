# Checked theme colors

Run `tsrust run examples/ui-colors/main.tsr`, or build this same entry for Wasm.
The inline project theme combines an alpha hex surface, OKLCH primary color and a
hyphenated semantic alias. `/50` multiplies the source alpha; it does not replace
it. Rings use independent width/color utilities and do not alter layout.
`inspect.tsr` asserts rendered output and unchanged source data with std:testing.

The root background must be opaque. Unsupported syntax, invalid component ranges
and colors outside the supported sRGB gamut fail instead of being gamut-mapped.
OKLCH conversion uses renderer float precision. Native paint encodes premultiplied
8-bit sRGB; small pixel rounding differences from browser paint remain possible.
This example does not claim wide-gamut, HDR or transparent-window support.
