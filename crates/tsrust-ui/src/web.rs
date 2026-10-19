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
pub(crate) const BASE: &str = "body{margin:0;font-family:system-ui,sans-serif}:where(.ui){box-sizing:border-box;font:inherit;color:inherit;border-color:var(--ui-border)}:where(div.ui){display:flex;flex-direction:column}:where(span.ui){display:block;white-space:pre-wrap}:where(button.ui){appearance:none;border:0 solid var(--ui-border);border-radius:0;background:transparent;text-align:center;padding:8px 16px;white-space:pre-wrap}:where(button.ui:disabled){opacity:.6}:where(button.ui:focus-visible){outline:2px solid var(--ui-ring);outline-offset:2px}:where([data-tsrust-theme]){color:var(--ui-foreground);background:var(--ui-background);font-size:var(--ui-text-base)}";
pub(crate) fn scope(node:&Node,theme:&crate::theme::Theme,mode:&str)->String {
    if node.theme.is_some() {theme.variables(mode)}else{String::new()}
}
fn element(node:&Node,out:&mut String,next:&mut usize,theme:&crate::theme::Theme,mode:&str) {
 let (theme,mode)=node.theme.as_ref().map(|(t,m)|(t,m.as_str())).unwrap_or((theme,mode));
 let id=*next;*next+=1;
 let tag=match node.kind {Kind::View=>"div",Kind::Text=>"span",Kind::Button|Kind::DisabledButton=>"button"};
 out.push('<');out.push_str(tag);
 if matches!(node.kind,Kind::Button|Kind::DisabledButton) {out.push_str(" type=\"button\"");}
 if node.kind==Kind::DisabledButton {out.push_str(" disabled");}
 if node.theme.is_some(){out.push_str(" data-tsrust-theme");}
 out.push_str(" class=\"ui ");escape(&node.class_name,out);
 out.push_str(&format!("\" data-tsrust-node=\"{id}\" style=\""));escape(&scope(node,theme,mode),out);out.push_str("\">");
 escape(&node.text,out);for child in &node.children {element(child,out,next,theme,mode);}
 out.push_str("</");out.push_str(tag);out.push('>');
}
pub(crate) fn stylesheet(node:&Node)->String {
 fn collect(node:&Node,next:&mut usize,theme:&crate::theme::Theme,mode:&str,out:&mut String) {
  let (theme,mode)=node.theme.as_ref().map(|(t,m)|(t,m.as_str())).unwrap_or((theme,mode));
  let id=*next;*next+=1;out.push_str(&crate::style::css(&node.class_name,theme,mode,id));
  for child in &node.children {collect(child,next,theme,mode,out);}
 }
 let default=crate::theme::Theme::default();let (theme,mode)=node.theme.as_ref().map(|(t,m)|(t,m.as_str())).unwrap_or((&default,"light"));let mut out=BASE.to_owned();
 out.push_str(&format!("body{{{}color:var(--ui-foreground);background:var(--ui-background);font-size:var(--ui-text-base)}}",theme.variables(mode)));
 collect(node,&mut 1,theme,mode,&mut out);out
}
pub(crate) fn document(root:&Node,title:&str)->String {
 let mut out=String::from("<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>");
 escape(title,&mut out);out.push_str("</title><style>");out.push_str(&stylesheet(root));out.push_str("</style></head><body>");
 element(root,&mut out,&mut 1,&crate::theme::Theme::default(),"light");out.push_str("</body></html>");out
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
