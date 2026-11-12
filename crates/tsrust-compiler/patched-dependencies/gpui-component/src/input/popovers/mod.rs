mod caret_anchored;
mod code_action_menu;
mod completion_menu;
mod diagnostic_popover;
mod hover_popover;

pub(crate) use caret_anchored::*;
pub(crate) use code_action_menu::*;
pub(crate) use completion_menu::*;
pub(crate) use diagnostic_popover::*;
pub(crate) use hover_popover::*;

use gpui::{
    App, Div, ElementId, InteractiveElement as _, SharedString, Stateful, StyleRefinement, Pixels,
    Styled as _, Window, div, rems,
};

use crate::{
    ActiveTheme, ThemeStyled as _,
    text::{TextView, TextViewStyle},
};

pub(super) fn render_markdown(
    id: impl Into<ElementId>,
    markdown: impl Into<SharedString>,
    font_size: Pixels,
    _: &mut Window,
    cx: &mut App,
) -> TextView {
    TextView::markdown(id, markdown)
        .style(
            TextViewStyle::default()
                .paragraph_gap(rems(0.5))
                .heading_font_size(move |_, _| font_size)
                .code_block(
                    StyleRefinement::default()
                        .bg(cx.theme().transparent)
                        .p_0()
                        .border_0()
                        .rounded_none()
                        .text_size(font_size),
                ),
        )
        .text_size(font_size)
        .code_block_actions(|_, _, _| gpui::Empty)
        .selectable(true)
}

pub(super) fn editor_popover(id: impl Into<ElementId>, cx: &App) -> Stateful<Div> {
    div()
        .id(id)
        .flex_none()
        .occlude()
        .popover_style(cx)
        .shadow_md()
        .text_xs()
        .p_1()
}
