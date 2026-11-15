use std::{ops::Range, rc::Rc};

use anyhow::Result;
use gpui::{App, Context, Task, Window};
use ropey::Rope;

use crate::input::{EditorMode, InputBaseState, ShowSignatureHelp};

pub trait SignatureHelpProvider {
    /// Read call signatures at a UTF-8 byte offset in the current document.
    fn signature_help(
        &self,
        text: &Rope,
        offset: usize,
        window: &mut Window,
        cx: &mut App,
    ) -> Task<Result<Option<lsp_types::SignatureHelp>>>;
}

pub struct SignatureHelpPopoverState {
    pub range: Range<usize>,
    pub help: lsp_types::SignatureHelp,
}

impl InputBaseState<EditorMode> {
    pub fn signature_help(&self) -> Option<Rc<SignatureHelpPopoverState>> {
        self.extras.lsp.signature_help.clone()
    }

    pub fn signature_help_active(&self) -> bool {
        self.extras.lsp.signature_help_active
    }

    pub fn dismiss_signature_help(&mut self) {
        self.extras.lsp.signature_help_active = false;
        self.extras.lsp.signature_help = None;
        self.extras.lsp.signature_help_position = None;
        self.extras.lsp._signature_help_task = Task::ready(Ok(()));
    }

    pub(crate) fn on_action_show_signature_help(
        &mut self,
        _: &ShowSignatureHelp,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.extras.lsp.signature_help_active = true;
        self.extras.lsp.signature_help_position = None;
        self.refresh_signature_help(window, cx);
    }

    pub(crate) fn handle_signature_help_trigger(
        &mut self,
        text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if matches!(text, "(" | ",") {
            self.extras.lsp.signature_help_active = true;
        }
        self.refresh_signature_help(window, cx);
    }

    /// Called after edits and cursor movement; unchanged frames never query the server.
    pub fn refresh_signature_help(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.extras.lsp.signature_help_active {
            return;
        }
        if !self.focus_handle.is_focused(window)
            || self.disabled
            || self.readonly
            || self.ime_marked_range.is_some()
        {
            self.dismiss_signature_help();
            return;
        }
        let Some(provider) = self.extras.lsp.signature_help_provider.clone() else {
            self.dismiss_signature_help();
            return;
        };
        let position = (self.document_revision, self.cursor());
        if self.extras.lsp.signature_help_position == Some(position) {
            return;
        }
        self.extras.lsp.signature_help_position = Some(position);
        self.extras.lsp.signature_help = None;
        let text = self.text.clone();
        self.extras.lsp._signature_help_task = cx.spawn_in(window, async move |this, cx| {
            // Coalesce typing and cursor movement before entering the language host.
            cx.background_executor()
                .timer(std::time::Duration::from_millis(75))
                .await;
            let task = this.update_in(cx, |_, window, cx| {
                provider.signature_help(&text, position.1, window, cx)
            })?;
            let result = task.await;
            this.update_in(cx, |editor, window, cx| {
                if !editor.extras.lsp.signature_help_active
                    || editor.extras.lsp.signature_help_position != Some(position)
                    || (editor.document_revision, editor.cursor()) != position
                {
                    return;
                }
                let help = result
                    .ok()
                    .flatten()
                    .filter(|help| !help.signatures.is_empty());
                if !editor.focus_handle.is_focused(window) || help.is_none() {
                    editor.dismiss_signature_help();
                } else if let Some(help) = help {
                    editor.extras.lsp.signature_help = Some(Rc::new(SignatureHelpPopoverState {
                        range: position.1..position.1,
                        help,
                    }));
                }
                cx.notify();
            })?;
            Ok(())
        });
        cx.notify();
    }
}
