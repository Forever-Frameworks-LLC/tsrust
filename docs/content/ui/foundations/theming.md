<!-- tsrust-docs
{"id":"ui-theming","group":"foundations","order":20,"status":"partial","targets":["web","native"]}
-->
# Theming

Project tokens live in `tsrust.config.json` under `ui.theme`. Components use semantic colors, typography, spacing and radii rather than embedding a separate theme file.

## Semantic tokens

Light and dark palettes include background/foreground, card, primary, secondary, muted, accent, destructive, border, input and ring colors. A token can reference another token with `@foreground` or use a supported checked color value.

Utilities such as `bg-primary`, `text-primary-foreground` and `rounded` resolve from the enclosing theme. Caller classes override earlier component defaults per property and state.

Invalid colors, references and theme ranges fail explicitly. Unsupported CSS expressions do not silently fall back to a guessed value.

## Live light and dark modes

`withTheme` accepts a local string signal for its mode. Switching between `light` and `dark` preserves counters, edited fields, DOM identity and pane scroll offsets. An explicitly nested theme scope keeps its own mode.

Token records are currently construction-time values. Live token-record mutation, a complete theme generator and custom named font assets remain pending. Editing project tokens and rebuilding is supported; claiming arbitrary live theme editing would be inaccurate.

## Shared target behavior

Both targets use logical dimensions and the same semantic tokens. Fonts may rasterize differently. Generic font families are supported; no WebView or React runtime is involved.

The native custom header uses genuine platform window controls. Browser pages retain their host's window chrome. Live native titlebar switching is not supported.
