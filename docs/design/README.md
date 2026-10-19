# Landing design reference

[landing-reference.html](landing-reference.html) is a complete standalone HTML
reference with embedded Tailwind 4.1.18 CSS and small browser scripts. It has no
CDN, external image, font or animation-framework dependency. Tailwind's notice is
in [the shared attributions directory](../attributions/tailwind-MIT.txt).

The production landing is [the shared component](../src/components/docs/landing.tsrx)
inside the web/native docs project. The HTML reference does not replace that app or
establish browser-only behavior as native acceptance. Its `/docs` links assume a
host serving the documentation routes. Opening it directly from disk still shows
the design and interactions; clipboard access depends on browser policy.

## Scroll state

An IntersectionObserver selects the most visible narrative chapter in a middle
band of the viewport. The desktop code panel follows that chapter using a short
opacity/position transition. Scrolling backward restores earlier panels. Mobile
shows each example directly after its chapter, in normal document flow. The script
never handles wheel/touch events or writes scroll positions.

Reduced-motion preferences disable animation and transitions. Without JavaScript,
the source panels remain readable as static content. Theme switching and copy/retry
use native browser APIs. No displayed build size, throughput or timing is invented.

## Checks

The bounded `landing-reference` browser witness in
[the UI application driver](../../benchmarks/ui-jsx/verify-application.mjs) covers
forward/backward chapter selection, widths 320/480/768/900/1440, sequential mobile
code, both theme palettes, AA contrast for normal text, reduced-motion styles and
exact copy with a failure/retry state. It owns and reaps Chrome and its profile.

When changing the reference, check:

- Keyboard focus, skip link and links through the sticky header.
- Forward/backward scrolling and viewport changes in the middle of each chapter.
- Narrow layouts, enlarged text and long source lines without page-level overflow.
- Both themes, visible focus and reduced-motion preferences.
- Copy success, denied clipboard access and selectable source fallback.
- JavaScript disabled, browser back/forward cache restoration and touch scrolling.

The last two manual/browser-specific checks are not established by the current
automated witness. Production paired visibility and native layout have separate
acceptance; this HTML reference cannot substitute for those checks.
