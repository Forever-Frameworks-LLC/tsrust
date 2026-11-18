use gpui_kit::component::{ActiveTheme, h_flex};
use gpui_kit::*;

/// One list row: the label and its length in a muted second column.
pub fn row(label: &str, cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    h_flex()
        .justify_between()
        .px_3()
        .py_1()
        .border_b_1()
        .border_color(theme.border)
        .child(div().text_sm().child(label.to_owned()))
        .child(div().text_sm().text_color(theme.muted_foreground).child(format!("{} chars", label.len())))
}
