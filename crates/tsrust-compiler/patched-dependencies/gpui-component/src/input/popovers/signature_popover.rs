use std::rc::Rc;

use gpui::{App, AppContext as _, Entity, IntoElement, Render, Window};
use gpui_base::input::SignatureHelpPopoverState;
use lsp_types::{Documentation, MarkupKind, ParameterLabel};

use crate::input::{
    EditorState,
    popovers::{Popover, render_markdown},
};

pub struct SignaturePopover {
    editor: Entity<EditorState>,
    help: Rc<SignatureHelpPopoverState>,
}

impl SignaturePopover {
    pub fn new(
        editor: Entity<EditorState>,
        help: Rc<SignatureHelpPopoverState>,
        cx: &mut App,
    ) -> Entity<Self> {
        cx.new(|_| Self { editor, help })
    }
}

fn inline(text: &str) -> String {
    text.chars()
        .flat_map(|ch| {
            if "\\`*_{}[]<>()#!|".contains(ch) {
                vec!['\\', ch]
            } else {
                vec![ch]
            }
        })
        .collect()
}

fn documentation(doc: &Documentation) -> String {
    match doc {
        Documentation::MarkupContent(markup) if markup.kind == MarkupKind::Markdown => {
            markup.value.clone()
        }
        Documentation::MarkupContent(markup) => inline(&markup.value),
        Documentation::String(text) => inline(text),
    }
}

fn parameter_label<'a>(label: &'a str, parameter: &'a ParameterLabel) -> Option<&'a str> {
    match parameter {
        ParameterLabel::Simple(text) => Some(text),
        ParameterLabel::LabelOffsets([start, end]) => {
            // LSP label offsets count UTF-16 units, independently of document coordinates.
            let boundary = |target| {
                let mut units = 0;
                for (byte, ch) in label.char_indices() {
                    if units == target {
                        return Some(byte);
                    }
                    units += ch.len_utf16() as u32;
                }
                (units == target).then_some(label.len())
            };
            label.get(boundary(*start)?..boundary(*end)?)
        }
    }
}

fn contents(help: &lsp_types::SignatureHelp, language: &str) -> String {
    let Some(signature) = help
        .signatures
        .get(help.active_signature.unwrap_or(0) as usize)
        .or_else(|| help.signatures.first())
    else {
        return String::new();
    };
    let fence = "`".repeat(
        signature
            .label
            .split(|ch| ch != '`')
            .map(str::len)
            .max()
            .unwrap_or(0)
            .max(2)
            + 1,
    );
    let mut text = format!("{fence}{language}\n{}\n{fence}", signature.label);
    let active = signature
        .active_parameter
        .or(help.active_parameter)
        .unwrap_or(0) as usize;
    let mut parameter_documented = false;
    if let Some(parameter) = signature
        .parameters
        .as_ref()
        .and_then(|parameters| parameters.get(active))
    {
        if let Some(label) = parameter_label(&signature.label, &parameter.label) {
            text.push_str(&format!("\n\n**{}**", inline(label)));
        }
        if let Some(doc) = &parameter.documentation {
            parameter_documented = true;
            text.push_str(&format!("\n\n{}", documentation(doc)));
        }
    }
    if !parameter_documented && let Some(doc) = &signature.documentation {
        text.push_str(&format!("\n\n{}", documentation(doc)));
    }
    text
}

impl Render for SignaturePopover {
    fn render(&mut self, _: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let editor = self.editor.read(cx);
        let font_size = editor.display_map().font_size();
        let contents = contents(&self.help.help, &editor.language_name());
        Popover::new(
            "signature-help",
            self.editor.clone(),
            self.help.range.clone(),
            move |window, cx| {
                render_markdown(
                    "signature-help-message",
                    contents.clone(),
                    font_size,
                    window,
                    cx,
                )
            },
        )
        .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_parameters_use_utf16_boundaries_and_signature_override() {
        let label = "call(🦀: int, value: string)";
        assert_eq!(
            parameter_label(label, &ParameterLabel::LabelOffsets([5, 7])),
            Some("🦀")
        );
        assert_eq!(
            parameter_label(label, &ParameterLabel::LabelOffsets([6, 7])),
            None
        );
        assert_eq!(
            parameter_label(label, &ParameterLabel::LabelOffsets([6, 1000])),
            None
        );
        let help: lsp_types::SignatureHelp = serde_json::from_value(serde_json::json!({
            "signatures": [{"label": "call(first: int, second: string)", "activeParameter": 1, "documentation": "Signature fallback",
                "parameters": [{"label":"first: int"}, {"label":"second: string", "documentation":"Literal *text*"}]}],
            "activeSignature": 99, "activeParameter": 0
        })).unwrap();
        let text = contents(&help, "rust");
        assert!(text.starts_with("```rust\ncall("));
        assert!(text.contains("**second: string**"));
        assert!(text.contains("Literal \\*text\\*"));
        assert!(!text.contains("Signature fallback"));
        assert!(
            contents(
                &lsp_types::SignatureHelp {
                    signatures: vec![],
                    active_signature: None,
                    active_parameter: None
                },
                "rust"
            )
            .is_empty()
        );
    }
}
