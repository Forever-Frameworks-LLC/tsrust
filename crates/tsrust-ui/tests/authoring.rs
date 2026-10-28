//! Legacy authoring must remain usable without node, event-loop or vector runtimes.
use tsrust_ui::{style, theme::Theme};

#[test]
fn theme_and_class_validation_work_without_runtime() {
    let theme = Theme::default();
    assert!(style::validate("flex gap-4 text-primary").is_ok());
    assert!(style::validate("unrecognized-utility").is_err());
    assert!(theme.color("light", "primary").is_ok());
    assert!(!style::documentation("gap-4").is_empty());
    assert!(!style::completion_tokens("text-pri").is_empty());
}
