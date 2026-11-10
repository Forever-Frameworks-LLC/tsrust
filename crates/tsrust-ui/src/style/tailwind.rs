//! Tailwind families beyond the core set: arbitrary `[...]` values, the rest of the type, spacing, border,
//! shadow, gradient, cursor, visibility and animation scales. The shared resolver supplies compiler checks, editor help and native style rules.
use super::{Length, Utility, layout::{pixels, SPACING}};
use crate::theme::{Theme, COLORS, ResolvedColor};

/// A color name (theme, palette or arbitrary) with an optional `/NN` opacity.
fn color(name: &str, theme: &Theme, mode: &str) -> Option<ResolvedColor> {
    let (name, alpha) = match name.rsplit_once('/') {
        Some((base, percent)) if !percent.is_empty() && percent.bytes().all(|byte| byte.is_ascii_digit()) => (base, percent.parse::<u8>().ok().filter(|value| *value <= 100)? as f32 / 100.),
        _ => (name, 1.),
    };
    let value = if name == "transparent" {ResolvedColor::transparent()} else if let Some(fixed) = crate::theme::fixed_color(name) {fixed} else if COLORS.contains(&name) {theme.color(mode, name).ok()?} else {return None};
    Some(value.opacity(alpha))
}

/// A spacing step, an arbitrary `[Npx]` or `[N]` pixel value.
fn space(size: &str, theme: &Theme) -> Option<f32> {
    super::layout::length_pixels(size, theme)
}

/// A positive number in brackets, such as `[0.35]` or `[1.4]`.
fn bracket_number(size: &str) -> Option<f32> {
    let text = size.strip_prefix('[')?.strip_suffix(']')?;
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit() || byte == b'.') {return None;}
    text.parse::<f32>().ok().filter(|value| value.is_finite())
}

const SIDES: [(&str, &[u8]); 7] = [("", &[0, 1, 2, 3]), ("x", &[1, 3]), ("y", &[0, 2]), ("t", &[0]), ("r", &[1]), ("b", &[2]), ("l", &[3])];

/// Cursor names in GPUI's cursor order (see `cursor_style` in the renderer).
pub(crate) const CURSORS: [&str; 15] = ["auto", "text", "not-allowed", "pointer", "default", "grab", "grabbing", "move", "crosshair", "col-resize", "row-resize", "ew-resize", "ns-resize", "context-menu", "copy"];

/// Animation names: spin, ping, pulse and bounce, as in Tailwind.
pub(crate) const ANIMATIONS: [&str; 5] = ["none", "spin", "ping", "pulse", "bounce"];

/// Gradient directions in degrees for `bg-linear-to-*` / `bg-gradient-to-*`.
const DIRECTIONS: [(&str, u16); 8] = [("t", 0), ("tr", 45), ("r", 90), ("br", 135), ("b", 180), ("bl", 225), ("l", 270), ("tl", 315)];

/// Completion tokens for the families above (arbitrary [..] values are typed by hand).
pub(super) fn tokens() -> Vec<String> {
    let mut tokens = ["text-lg", "text-8xl", "text-9xl", "font-thin", "font-extralight", "font-light", "font-extrabold", "font-black", "italic", "not-italic",
        "normal-case", "uppercase", "lowercase", "capitalize", "line-through", "no-line-through", "truncate", "text-ellipsis", "text-clip", "line-clamp-none",
        "leading-none", "leading-loose", "tracking-tighter", "tracking-wide", "tracking-wider", "tracking-widest", "flex-auto", "flex-initial", "flex-none",
        "visible", "invisible", "overflow-scroll", "overflow-x-auto", "overflow-x-hidden", "overflow-y-auto", "overflow-y-hidden", "h-dvh", "min-h-dvh",
        "border-none", "border-4", "border-8", "border-x", "border-y", "border-x-2", "border-y-2", "border-x-0", "border-y-0",
        "shadow-none", "shadow-2xs", "shadow-xs", "shadow-sm", "shadow", "shadow-md", "shadow-lg", "shadow-xl", "shadow-2xl",
        "basis-auto", "basis-full", "basis-1/2", "basis-1/3", "basis-2/3", "basis-1/4", "basis-3/4"].map(str::to_owned).to_vec();
    tokens.extend((1..=6).map(|lines| format!("line-clamp-{lines}")));
    tokens.extend((3..=10).map(|step| format!("leading-{step}")));
    tokens.extend((0..=100).step_by(5).map(|step| format!("opacity-{step}")));
    tokens.extend(["start", "center", "end", "between", "around", "evenly", "stretch"].map(|value| format!("content-{value}")));
    tokens.extend(CURSORS.iter().skip(3).map(|cursor| format!("cursor-{cursor}")));
    tokens.extend(ANIMATIONS.iter().map(|animation| format!("animate-{animation}")));
    tokens.extend(DIRECTIONS.iter().flat_map(|(direction, _)| [format!("bg-linear-to-{direction}"), format!("bg-gradient-to-{direction}")]));
    for prefix in ["m", "mx", "my", "mt", "mr", "mb", "ml"] {tokens.push(format!("{prefix}-auto"));}
    for prefix in ["space-x", "space-y"] {tokens.extend(SPACING.iter().map(|step| format!("{prefix}-{step}")));}
    for name in COLORS.iter().copied().chain(["transparent"]).chain(crate::theme::palette_names()) {
        for prefix in ["shadow", "from", "via", "to"] {tokens.push(format!("{prefix}-{name}"));}
    }
    tokens
}

pub(super) fn resolve(token: &str, theme: &Theme, mode: &str) -> Option<Vec<Utility>> {
    let one = |value: Utility| Some(vec![value]);
    let base = theme.typography.base as f32;
    // Negative margins: -m-2, -mt-[4px].
    if let Some(rest) = token.strip_prefix('-') && rest.starts_with('m') {
        // Only lengths negate; -m-auto is not a margin.
        return margins(rest, theme).filter(|values| values.iter().all(|value| matches!(value, Utility::Margin(..)))).map(|values| values.into_iter().map(|value| match value {Utility::Margin(side, amount) => Utility::Margin(side, -amount), other => other}).collect());
    }
    if let Some(values) = margins(token, theme) {return Some(values);}
    // Arbitrary padding and gaps: p-[13px], px-[6px], gap-[6px], gap-x-[10px]; space-x/y as gaps.
    for (prefix, sides) in [("p-", &[0u8, 1, 2, 3][..]), ("px-", &[1, 3]), ("py-", &[0, 2]), ("pt-", &[0]), ("pr-", &[1]), ("pb-", &[2]), ("pl-", &[3])] {
        if let Some(size) = token.strip_prefix(prefix) && let Some(value) = space(size, theme) {return Some(sides.iter().map(|side| Utility::Padding(*side, value)).collect());}
    }
    for (prefix, axes) in [("gap-", &[0u8, 1][..]), ("gap-x-", &[1]), ("gap-y-", &[0])] {
        if let Some(size) = token.strip_prefix(prefix) && let Some(value) = space(size, theme) {return Some(axes.iter().map(|axis| Utility::GapAxis(*axis, value)).collect());}
    }
    for (prefix, axis) in [("space-x-", 1u8), ("space-y-", 0)] {
        if let Some(size) = token.strip_prefix(prefix) && let Some(value) = space(size, theme) {return one(Utility::GapAxis(axis, value));}
    }
    // Arbitrary radii: rounded-[10px], rounded-t-[6px], rounded-tl-[4px].
    if let Some(rest) = token.strip_prefix("rounded-") {
        let (corners, size): (&[u8], &str) = match rest.split_once('-') {
            Some(("t", size)) => (&[0, 1], size), Some(("r", size)) => (&[1, 2], size), Some(("b", size)) => (&[2, 3], size), Some(("l", size)) => (&[3, 0], size),
            Some(("tl", size)) => (&[0], size), Some(("tr", size)) => (&[1], size), Some(("br", size)) => (&[2], size), Some(("bl", size)) => (&[3], size),
            _ => (&[0, 1, 2, 3], rest),
        };
        if let Some(value) = pixels(size) {return Some(corners.iter().map(|corner| Utility::Corner(*corner, value)).collect());}
    }
    // Border widths: border-4, border-8, border-x, border-y-2, border-[3px], border-t-[3px], border-none.
    if token == "border-none" {return Some((0..4).map(|side| Utility::EdgeBorder(side, 0.)).collect());}
    if token == "border-solid" || token == "border-dashed" {return None;}
    if let Some(rest) = token.strip_prefix("border") {
        for (side, sides) in SIDES {
            let tail = if side.is_empty() {rest} else if let Some(tail) = rest.strip_prefix('-').and_then(|tail| tail.strip_prefix(side)) {tail} else {continue};
            let width = match tail {"" if matches!(side, "x" | "y") => Some(1.), "-0" | "-2" if matches!(side, "x" | "y") => Some(tail[1..].parse::<f32>().unwrap()), "-4" => Some(4.), "-8" => Some(8.), tail => tail.strip_prefix('-').and_then(pixels)};
            if let Some(width) = width {return Some(sides.iter().map(|side| Utility::EdgeBorder(*side, width)).collect());}
        }
    }
    // Type: sizes, weights, style, case, decoration, leading, tracking, overflow.
    match token {
        "text-lg" => return Some(vec![Utility::FontSize(base * 1.125), Utility::DefaultLeading(base * 1.75)]),
        "text-8xl" => return one(Utility::FontSize(base * 6.)),
        "text-9xl" => return one(Utility::FontSize(base * 8.)),
        "font-thin" => return one(Utility::Weight(100.)), "font-extralight" => return one(Utility::Weight(200.)), "font-light" => return one(Utility::Weight(300.)),
        "font-extrabold" => return one(Utility::Weight(800.)), "font-black" => return one(Utility::Weight(900.)),
        "italic" => return one(Utility::Italic(true)), "not-italic" => return one(Utility::Italic(false)),
        "normal-case" => return one(Utility::TextTransform(0)), "uppercase" => return one(Utility::TextTransform(1)), "lowercase" => return one(Utility::TextTransform(2)), "capitalize" => return one(Utility::TextTransform(3)),
        "line-through" => return one(Utility::LineThrough(true)), "no-line-through" => return one(Utility::LineThrough(false)),
        "truncate" => return Some(vec![Utility::TextOverflow(1), Utility::WhiteSpace(super::WhiteSpace::NoWrap), Utility::Overflow(1)]),
        "text-ellipsis" => return one(Utility::TextOverflow(1)), "text-clip" => return one(Utility::TextOverflow(0)),
        "line-clamp-none" => return one(Utility::LineClamp(0)),
        "leading-none" => return one(Utility::Leading(1.)), "leading-loose" => return one(Utility::Leading(2.)),
        "tracking-tighter" => return one(Utility::Tracking(-0.05)), "tracking-wide" => return one(Utility::Tracking(0.025)),
        "tracking-wider" => return one(Utility::Tracking(0.05)), "tracking-widest" => return one(Utility::Tracking(0.1)),
        _ => {}
    }
    if let Some(size) = token.strip_prefix("text-[") .map(|rest| format!("[{rest}")) && let Some(value) = pixels(&size) {return one(Utility::FontSize(value));}
    if let Some(weight) = token.strip_prefix("font-[").and_then(|rest| rest.strip_suffix(']')) && let Ok(weight) = weight.parse::<u16>() && (1..=1000).contains(&weight) {return one(Utility::Weight(weight as f32));}
    if let Some(lines) = token.strip_prefix("line-clamp-") && let Ok(lines) = lines.parse::<u8>() && (1..=12).contains(&lines) {return one(Utility::LineClamp(lines));}
    if let Some(size) = token.strip_prefix("leading-") {
        if let Ok(step) = size.parse::<u8>() && (3..=10).contains(&step) {return one(Utility::DefaultLeading(step as f32 * theme.spacing.md as f32 / 4.));}
        if let Some(value) = bracket_number(size) {return one(Utility::Leading(value));}
        if let Some(value) = pixels(size) {return one(Utility::DefaultLeading(value));}
    }
    if let Some(size) = token.strip_prefix("tracking-[").and_then(|rest| rest.strip_suffix("em]")) && let Ok(value) = size.parse::<f32>() && value.is_finite() {return one(Utility::Tracking(value));}
    // Opacity: every step of 5 and opacity-[0.37].
    if let Some(step) = token.strip_prefix("opacity-") {
        if let Ok(step) = step.parse::<u8>() && step <= 100 && step % 5 == 0 {return one(Utility::Opacity(step as f32 / 100.));}
        if let Some(value) = bracket_number(step) && value <= 1. {return one(Utility::Opacity(value));}
    }
    // Flex and grid extras.
    match token {
        "flex-auto" => return Some(vec![Utility::Grow(1.), Utility::Shrink(1.), Utility::Basis(Length::Auto)]),
        "flex-initial" => return Some(vec![Utility::Grow(0.), Utility::Shrink(1.), Utility::Basis(Length::Auto)]),
        "flex-none" => return Some(vec![Utility::Grow(0.), Utility::Shrink(0.), Utility::Basis(Length::Auto)]),
        "visible" => return one(Utility::Visible(true)), "invisible" => return one(Utility::Visible(false)),
        "overflow-scroll" => return one(Utility::Overflow(2)),
        "overflow-x-auto" | "overflow-x-scroll" => return one(Utility::OverflowAxis(0, 2)), "overflow-x-hidden" => return one(Utility::OverflowAxis(0, 1)), "overflow-x-visible" => return one(Utility::OverflowAxis(0, 0)),
        "overflow-y-auto" | "overflow-y-scroll" => return one(Utility::OverflowAxis(1, 2)), "overflow-y-hidden" => return one(Utility::OverflowAxis(1, 1)), "overflow-y-visible" => return one(Utility::OverflowAxis(1, 0)),
        "h-dvh" | "h-svh" | "h-lvh" => return one(Utility::Dimension(1, Length::Viewport(1))),
        "min-h-dvh" | "min-h-svh" => return one(Utility::Dimension(3, Length::Viewport(1))),
        _ => {}
    }
    if let Some(value) = token.strip_prefix("content-") && let Some(index) = ["start", "center", "end", "between", "around", "evenly", "stretch"].iter().position(|name| *name == value) {return one(Utility::AlignContent(index as u8));}
    if let Some(size) = token.strip_prefix("basis-") {
        let value = match size {"auto" => Some(Length::Auto), "full" => Some(Length::Percent(1.)), "1/2" => Some(Length::Percent(0.5)), "1/3" => Some(Length::Percent(1. / 3.)), "2/3" => Some(Length::Percent(2. / 3.)), "1/4" => Some(Length::Percent(0.25)), "3/4" => Some(Length::Percent(0.75)), size => space(size, theme).map(Length::Pixels)};
        if let Some(value) = value {return one(Utility::Basis(value));}
    }
    if let Some(value) = token.strip_prefix("grow-[").and_then(|rest| rest.strip_suffix(']')).and_then(|value| value.parse::<f32>().ok()).filter(|value| value.is_finite() && *value >= 0.) {return one(Utility::Grow(value));}
    // Shadows: shadow-2xs..2xl, shadow-none, shadow-inner and shadow colors (shadow-black/20).
    if let Some(index) = ["shadow-none", "shadow-2xs", "shadow-xs", "shadow-sm", "shadow", "shadow-md", "shadow-lg", "shadow-xl", "shadow-2xl"].iter().position(|name| *name == token) {return one(Utility::Shadow(index as u8));}
    if let Some(name) = token.strip_prefix("shadow-") && let Some(value) = color(name, theme, mode) {return one(Utility::ShadowColor(value));}
    // Gradients: bg-linear-to-r (Tailwind 4) or bg-gradient-to-r, from-*, via-*, to-* stops.
    for prefix in ["bg-linear-to-", "bg-gradient-to-"] {
        if let Some(direction) = token.strip_prefix(prefix) && let Some((_, angle)) = DIRECTIONS.iter().find(|(name, _)| *name == direction) {return one(Utility::GradientAngle(*angle));}
    }
    if let Some(angle) = token.strip_prefix("bg-linear-").and_then(|angle| angle.parse::<u16>().ok()) && angle < 360 {return one(Utility::GradientAngle(angle));}
    for (prefix, stop) in [("from-", 0u8), ("via-", 1), ("to-", 2)] {
        if let Some(name) = token.strip_prefix(prefix) && let Some(value) = color(name, theme, mode) {return one(Utility::GradientStop(stop, value));}
    }
    // Cursors and animations.
    if let Some(name) = token.strip_prefix("cursor-") && let Some(index) = CURSORS.iter().position(|cursor| *cursor == name) && index > 2 {return one(Utility::Cursor(index as u8));}
    if let Some(name) = token.strip_prefix("animate-") && let Some(index) = ANIMATIONS.iter().position(|animation| *animation == name) {return one(Utility::Animation(index as u8));}
    // Arbitrary sizes not covered by w-/h-[px]: size-[..] handled in layout; min/max via layout.
    None
}

/// Arbitrary and auto margins: m-[6px], mx-auto, mt-[10px].
fn margins(token: &str, theme: &Theme) -> Option<Vec<Utility>> {
    for (prefix, sides) in [("m-", &[0u8, 1, 2, 3][..]), ("mx-", &[1, 3]), ("my-", &[0, 2]), ("mt-", &[0]), ("mr-", &[1]), ("mb-", &[2]), ("ml-", &[3])] {
        let Some(size) = token.strip_prefix(prefix) else {continue};
        if size == "auto" {return Some(sides.iter().map(|side| Utility::MarginAuto(*side)).collect());}
        if let Some(value) = space(size, theme) {return Some(sides.iter().map(|side| Utility::Margin(*side, value)).collect());}
    }
    None
}

/// Hover documentation for the tokens this module adds.
pub(super) fn documentation(token: &str) -> Option<&'static str> {
    let token = token.strip_prefix('-').unwrap_or(token);
    Some(if token.starts_with("shadow") {
        "Adds a drop shadow: shadow-2xs through shadow-2xl grow the blur and offset, shadow-none removes it, and shadow-<color>/<opacity> tints it. Paints outside the box without changing layout."
    } else if token.starts_with("bg-linear-") || token.starts_with("bg-gradient-") || token.starts_with("from-") || token.starts_with("via-") || token.starts_with("to-") {
        "Paints a linear gradient background. bg-linear-to-r (or bg-gradient-to-r, or bg-linear-45 in degrees) sets the direction; from-, via- and to- colors set the stops, with /<opacity> allowed. A from- color alone fades to transparent."
    } else if token.starts_with("animate-") {
        "Runs a looping animation: animate-spin rotates icons, animate-pulse fades the opacity in and out, animate-ping and animate-bounce pulse the size and position. animate-none stops it."
    } else if token.starts_with("cursor-") {
        "Sets the mouse cursor over the box: pointer, default, grab, grabbing, move, crosshair, col-resize, row-resize, ew-resize, ns-resize, context-menu or copy."
    } else if matches!(token, "italic" | "not-italic") {
        "Sets italic or upright text for the element and its text descendants."
    } else if matches!(token, "uppercase" | "lowercase" | "capitalize" | "normal-case") {
        "Changes the displayed letter case of text without changing the text value: uppercase, lowercase, capitalize (first letter of each word) or normal-case."
    } else if matches!(token, "line-through" | "no-line-through") {
        "Draws or removes a line through text."
    } else if matches!(token, "truncate" | "text-ellipsis" | "text-clip") || token.starts_with("line-clamp-") {
        "Shortens overflowing text: truncate keeps one line and ends it with an ellipsis; line-clamp-N limits text to N lines; text-ellipsis and text-clip choose the overflow marker."
    } else if token.starts_with("opacity-") {
        "Sets the opacity of the box and its contents: opacity-0 through opacity-100 in steps of 5, or opacity-[0.37]."
    } else if token.starts_with("leading-") || token.starts_with("tracking-") {
        "Sets line height (leading-none, leading-loose, leading-3 to leading-10, leading-[1.4], leading-[20px]) or letter spacing (tracking-tighter to tracking-widest, tracking-[0.02em])."
    } else if token.starts_with("font-") || token.starts_with("text-") {
        "Sets font weight (font-thin to font-black, font-[550]) or size (text-lg, text-8xl, text-9xl, text-[13px])."
    } else if token.starts_with("space-") {
        "Spaces the children of a flex container along one axis, the same as gap-x or gap-y with that step."
    } else if token.starts_with("basis-") || matches!(token, "flex-auto" | "flex-initial" | "flex-none") || token.starts_with("grow-") {
        "Sets how a flex item sizes: basis-* is its starting size (spacing step, fraction, full, auto or [Npx]); flex-auto grows and shrinks from its content size; flex-none keeps it fixed."
    } else if token.starts_with("content-") {
        "Aligns wrapped flex lines or grid rows along the cross axis: start, center, end, between, around, evenly or stretch."
    } else if matches!(token, "visible" | "invisible") {
        "invisible hides the box but keeps its space in the layout; visible shows it again."
    } else if token.starts_with("overflow-") || token.starts_with("h-") || token.starts_with("min-h-") {
        "Sets per-axis overflow (overflow-x-auto scrolls horizontally) or a full-viewport height (h-dvh)."
    } else if token.starts_with("rounded-") || token.starts_with("border") {
        "Sets corner radius or border width with exact pixels or wider steps: rounded-[10px], rounded-t-[6px], border-4, border-x, border-y-2, border-[3px], border-none."
    } else if token.starts_with('m') || token.starts_with('p') || token.starts_with("gap-") {
        "Sets spacing with exact pixels or auto: p-[13px], gap-[6px], m-[10px], mx-auto centers a box with a width, and a leading minus (-mt-2) pulls it outward."
    } else {
        return None;
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn added_families_resolve_with_docs_and_reject_invalid_arbitrary_values() {
        let theme=Theme::default();
        for token in tokens() {
            super::super::validate(&token).unwrap_or_else(|error|panic!("{token}: {error}"));
            assert!(!super::super::resolved(&token,&theme,"light").is_empty(),"{token}");
            assert!(!super::super::documentation(&token).is_empty(),"{token}");
        }
        let rules=super::super::resolved("p-[13px] px-[6px] gap-[7px] -mt-[4px] mx-auto rounded-tr-[9px] border-x-[3px] font-[550] text-[13px] opacity-[0.37] basis-[20px]",&theme,"light");
        for expected in ["padding-top:13px","padding-left:6px","row-gap:7px","margin-top:-4px","margin-left:auto","border-top-right-radius:9px","border-left-width:3px","font-weight:550","font-size:13px","opacity:0.37","flex-basis:20px"] {
            assert!(rules.iter().any(|(_,rule)|rule.css()==expected),"{expected}");
        }
        for (token,expected) in [("w-4/50","width:8%"),("w-[37%]","width:37%"),("w-[1.5rem]","width:24px"),("p-13","padding-top:52px"),("p-[0.5rem]","padding-top:8px"),("text-2xl","font-size:24px")] {
            super::super::validate(token).unwrap();
            assert!(super::super::resolved(token,&theme,"light").iter().any(|(_,rule)|rule.css()==expected),"{token}");
        }
        for token in ["p-[NaNpx]","gap-[infpx]","-m-auto","opacity-[1.1]","font-[0]","grow-[NaN]","grow-[-1]","rounded-[bad]"] {
            assert!(super::super::validate(token).is_err(),"{token}");
        }
    }
}
