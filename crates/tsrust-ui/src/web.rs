use crate::{Kind, Node};

fn escape(value: &str, out: &mut String) {
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
}
fn element(node: &Node, out: &mut String) {
    let tag = match node.kind {
        Kind::View => "div",
        Kind::Text => "span",
        Kind::DisabledButton => "button",
    };
    out.push('<');
    out.push_str(tag);
    if node.kind == Kind::DisabledButton {
        out.push_str(" type=\"button\" disabled");
    }
    out.push_str(" class=\"ui ");
    escape(&node.class_name, out);
    out.push_str("\">");
    escape(&node.text, out);
    for child in &node.children {
        element(child, out);
    }
    out.push_str("</");
    out.push_str(tag);
    out.push('>');
}
pub(crate) fn document(root: &Node, title: &str) -> String {
    let mut out = String::from(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>",
    );
    escape(title, &mut out);
    out.push_str("</title><style>body{margin:0;font-family:system-ui,sans-serif;color:#172033;background:#fff}.ui{box-sizing:border-box;font:inherit}div.ui{display:flex;flex-direction:column}span.ui{display:block;white-space:pre-wrap}button.ui{appearance:none;border:0;border-radius:0;background:transparent;color:inherit;text-align:start;padding:8px 16px;white-space:pre-wrap}button.ui:disabled{opacity:.6}");
    let mut tokens = std::collections::BTreeSet::new();
    fn collect<'a>(node: &'a Node, tokens: &mut std::collections::BTreeSet<&'a str>) {
        tokens.extend(node.class_name.split_whitespace());
        for child in &node.children {
            collect(child, tokens);
        }
    }
    collect(root, &mut tokens);
    for token in tokens {
        if let Some(rule) = crate::style::Utility::parse(token) {
            out.push('.');
            out.push_str(token);
            out.push('{');
            out.push_str(&rule.css());
            out.push('}');
        }
    }
    out.push_str("</style></head><body>");
    element(root, &mut out);
    out.push_str("</body></html>");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn semantic_markup_escapes_text_and_rejects_unknown_utilities() {
        let text = Node::new(
            Kind::Text,
            "text-xl".into(),
            "<&\"' 🦀\nsecond".into(),
            vec![],
        )
        .unwrap();
        let root = Node::new(Kind::View, "p-4 gap-2".into(), String::new(), vec![text]).unwrap();
        let html = document(&root, "<title>");
        assert!(html.contains("&lt;&amp;&quot;&#39; 🦀\nsecond"));
        assert!(html.contains("<title>&lt;title&gt;</title>"));
        assert!(!html.contains("<script"));
        assert!(Node::new(Kind::View, "bg-[url(evil)]".into(), String::new(), vec![]).is_err());
    }
}
