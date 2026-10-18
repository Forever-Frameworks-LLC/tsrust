use tsrust_ui::{Kind, Node};
fn main() -> Result<(), String> {
    let screen = Node::new(
        Kind::View,
        "p-4 gap-4 bg-background".into(),
        String::new(),
        vec![
            Node::new(
                Kind::Text,
                "text-xl font-bold".into(),
                "Tsrust UI".into(),
                vec![],
            )?,
            Node::new(
                Kind::Text,
                "text-base".into(),
                "One semantic tree.\nUnicode: café · العربية · 🦀".into(),
                vec![],
            )?,
            Node::new(
                Kind::DisabledButton,
                "p-2 rounded bg-primary text-primary-foreground".into(),
                "Static preview".into(),
                vec![],
            )?,
        ],
    )?;
    if std::env::args().any(|arg| arg == "--html") {
        println!("{}", tsrust_ui::document(&screen, "Tsrust UI"));
        return Ok(());
    }
    #[cfg(feature = "acceptance")]
    if std::env::args().any(|arg| arg == "--verify-window") {
        tsrust_ui::verify_window(screen, "Tsrust UI acceptance".into())?;
        println!("GPU frame presented; event loop closed");
        return Ok(());
    }
    #[cfg(feature = "native")]
    {
        tsrust_ui::run(screen, "Tsrust UI".into())
    }
    #[cfg(not(feature = "native"))]
    {
        Err("Enable the native feature to open a window, or pass --html".into())
    }
}
