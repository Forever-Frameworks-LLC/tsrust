use anyhow::Result;
use gpui::{App, Context, MouseMoveEvent, Task, Window};
use instant::Duration;
use ropey::Rope;

use crate::input::{EditorMode, HoverPopoverState, InputBaseState, RopeExt, ShowHover};

/// Hover provider
///
/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_hover
pub trait HoverProvider {
    /// textDocument/hover
    ///
    /// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_hover
    fn hover(
        &self,
        _text: &Rope,
        _offset: usize,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Task<Result<Option<lsp_types::Hover>>>;
}

impl InputBaseState<EditorMode> {
    pub(crate) fn on_action_show_hover(&mut self, _: &ShowHover, window: &mut Window, cx: &mut Context<Self>) {
        self.show_hover(window, cx);
    }

    /// Show documentation at the caret without editing, selecting or moving focus.
    /// Readonly editors are supported; disabled editors and IME preedit are left alone.
    /// Uses the pointer hover provider immediately. Escape, caret movement, blur and
    /// subsequent requests cancel it; replies for an older document are discarded.
    pub fn show_hover(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled || self.ime_marked_range.is_some() { return; }
        self.clear_hover_state(cx);
        self.request_hover(self.cursor(), false, true, window, cx);
    }

    /// Handle hover trigger LSP request.
    pub(super) fn handle_hover_popover(
        &mut self,
        offset: usize,
        window: &mut Window,
        cx: &mut Context<InputBaseState<EditorMode>>,
    ) {
        self.request_hover(offset, true, false, window, cx);
    }

    fn request_hover(&mut self, offset: usize, delay: bool, from_caret: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.selecting || self.disabled || self.ime_marked_range.is_some() {
            return;
        }

        let Some(provider) = self.extras.lsp.hover_provider.clone() else {
            return;
        };

        if let Some(hover_popover) = self.extras.hover_popover.as_ref() {
            if hover_popover.symbol_range.contains(&offset) {
                return;
            }
        }

        let text = self.text.clone();
        let revision = self.document_revision;
        let selection = from_caret.then(|| self.selected_range());
        let mut symbol_range = self.text.word_range(offset).unwrap_or(offset..offset);
        let editor = cx.entity();
        let should_delay = delay && self.extras.hover_popover.is_none();
        // Ask the provider only after the delay: every mouse move replaces this
        // task, so positions the pointer merely passes over never reach it.
        self.extras.lsp._hover_task = cx.spawn_in(window, async move |this, cx| {
            if should_delay {
                cx.background_executor()
                    .timer(Duration::from_millis(150))
                    .await;
            }

            let task = this.update_in(cx, |_, window, cx| {
                provider.hover(&text, offset, window, cx)
            })?;
            let result = task.await?;

            _ = editor.update(cx, |editor, cx| {
                if editor.document_revision != revision || selection.as_ref().is_some_and(|range| editor.selected_range() != *range) {return;}
                match result {
                    Some(hover) => {
                        if let Some(range) = hover.range {
                            let start = editor.text.position_to_offset(&range.start);
                            let end = editor.text.position_to_offset(&range.end);
                            symbol_range = start..end;
                        }
                        editor.extras.hover_popover = Some(HoverPopoverState {
                            symbol_range,
                            hover,
                        });
                    }
                    None => {
                        editor.extras.hover_popover = None;
                    }
                }
                cx.notify();
            });

            Ok(())
        });
    }

    pub(crate) fn handle_mouse_move(
        &mut self,
        offset: usize,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.modifiers.alt {
            self.clear_hover_state(cx);
            return;
        }
        if event.modifiers.secondary(cx) {
            self.handle_hover_definition(offset, window, cx);
        } else {
            // Repaint only when the definition underline goes away; the hover
            // and definition results notify when they arrive.
            if !self.extras.hover_definition.is_empty() {
                self.extras.hover_definition.clear();
                cx.notify();
            }
            self.handle_hover_popover(offset, window, cx);
        }
    }

    pub fn clear_hover_state(&mut self, cx: &mut Context<Self>) {
        let changed =
            !self.extras.hover_definition.is_empty() || self.extras.hover_popover.is_some();
        self.extras.hover_definition.clear();
        self.extras.hover_popover = None;
        self.extras.lsp._hover_task = Task::ready(Ok(()));
        if changed {
            cx.notify();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{AppContext as _, TestAppContext};
    use std::{cell::RefCell, rc::Rc};
    use crate::input::{EditorState, Escape};

    struct Provider {
        calls: Rc<RefCell<Vec<usize>>>,
        delay: Duration,
    }

    impl HoverProvider for Provider {
        fn hover(&self, _: &Rope, offset: usize, _: &mut Window, cx: &mut App) -> Task<Result<Option<lsp_types::Hover>>> {
            self.calls.borrow_mut().push(offset);
            let timer = cx.background_executor().timer(self.delay);
            cx.spawn(async move |_| {
                timer.await;
                Ok(Some(lsp_types::Hover {
                    contents: lsp_types::HoverContents::Markup(lsp_types::MarkupContent {
                        kind: lsp_types::MarkupKind::Markdown,
                        value: "```tsrust\nconst café: Int\n```".into(),
                    }),
                    range: None,
                }))
            })
        }
    }

    #[gpui::test]
    fn hover_requests_keyboard_are_immediate_readonly_and_dismissible(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let cx = cx.add_empty_window();
        let calls = Rc::new(RefCell::new(Vec::new()));
        let editor = cx.update(|window, cx| {
            let editor = cx.new(|cx| EditorState::new(window, cx).default_value("const café = 8;"));
            editor.update(cx, |state, cx| {
                state.focus(window, cx);
                state.set_selected_range(8..8, cx);
                state.set_readonly(true, cx);
                state.extras.lsp.hover_provider = Some(Rc::new(Provider {calls: calls.clone(), delay: Duration::ZERO}));
                assert!(state.context_menu_capabilities().has_hover());
                state.on_action_show_hover(&ShowHover, window, cx);
            });
            editor
        });
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::ZERO);
        cx.run_until_parked();
        assert_eq!(*calls.borrow(), vec![8]);
        cx.update(|window, cx| editor.update(cx, |state, cx| {
            assert!(state.hover_popover().is_some());
            assert_eq!(state.value(), "const café = 8;");
            assert_eq!(state.selected_range(), 8..8);
            state.escape(&Escape, window, cx);
            assert!(state.hover_popover().is_none());
            state.set_disabled(true, cx);
            state.show_hover(window, cx);
        }));
        cx.run_until_parked();
        assert_eq!(*calls.borrow(), vec![8], "disabled editor must not query");
    }

    #[gpui::test]
    fn hover_requests_cancel_on_cursor_escape_and_document_changes(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let cx = cx.add_empty_window();
        let calls = Rc::new(RefCell::new(Vec::new()));
        let editor = cx.update(|window, cx| {
            let editor = cx.new(|cx| EditorState::new(window, cx).default_value("const café = 8;"));
            editor.update(cx, |state, cx| {
                state.focus(window, cx);
                state.set_selected_range(8..8, cx);
                state.extras.lsp.hover_provider = Some(Rc::new(Provider {calls: calls.clone(), delay: Duration::from_millis(50)}));
            });
            editor
        });
        for change in 0..3 {
            cx.update(|window, cx| editor.update(cx, |state, cx| state.show_hover(window, cx)));
            cx.run_until_parked();
            assert_eq!(calls.borrow().len(), change + 1);
            cx.update(|window, cx| editor.update(cx, |state, cx| match change {
                0 => {state.set_selected_range(0..0, cx); state.set_selected_range(8..8, cx);},
                1 => state.escape(&Escape, window, cx),
                _ => state.set_value("new document", window, cx),
            }));
            cx.executor().advance_clock(Duration::from_millis(100));
            cx.run_until_parked();
            assert!(editor.read_with(cx, |state, _| state.hover_popover().is_none()));
        }
    }

    #[gpui::test]
    fn hover_requests_pointer_keep_their_coalescing_delay(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let cx = cx.add_empty_window();
        let calls = Rc::new(RefCell::new(Vec::new()));
        let editor = cx.update(|window, cx| {
            let editor = cx.new(|cx| EditorState::new(window, cx).default_value("const café = 8;"));
            editor.update(cx, |state, cx| {
                state.extras.lsp.hover_provider = Some(Rc::new(Provider {calls: calls.clone(), delay: Duration::ZERO}));
                state.handle_hover_popover(8, window, cx);
            });
            editor
        });
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_millis(100));
        cx.run_until_parked();
        assert!(calls.borrow().is_empty());
        cx.executor().advance_clock(Duration::from_millis(100));
        cx.run_until_parked();
        assert_eq!(*calls.borrow(), vec![8]);
        assert!(editor.read_with(cx, |state, _| state.hover_popover().is_some()));
    }
}
