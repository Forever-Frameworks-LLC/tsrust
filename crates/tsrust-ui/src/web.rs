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
pub(crate) const BASE: &str = "body{margin:0;font-family:system-ui,sans-serif;white-space:pre-wrap}:where(.ui){--ui-shadow-ring-width:0px;--ui-shadow-ring-color:var(--ui-ring);position:relative;box-sizing:border-box;margin:0;font:inherit;color:inherit;border-color:var(--ui-border);white-space:inherit}:where(.ui[data-tsrust-view]){display:flex;flex-direction:column}:where(.ui[data-tsrust-text]){display:block}:where([data-tsrust-text] .ui){display:inline}:where(strong.ui){font-weight:700}:where(em.ui){font-style:italic}:where(del.ui){text-decoration:line-through}:where([data-tsrust-text] a.ui){text-decoration:underline;background:transparent}:where(ul.ui,ol.ui){padding:0;list-style:none}:where(a.ui){text-decoration:none;cursor:pointer}:where(a.ui[aria-current=page]){background:var(--ui-muted);font-weight:500}:where(a.ui:focus-visible){outline:2px solid var(--ui-ring);outline-offset:2px}:where(button.ui){display:flex;flex-direction:row;align-items:center;justify-content:center;appearance:none;border:0 solid var(--ui-border);border-radius:0;background:transparent;text-align:center;padding:8px 16px;white-space:inherit}:where(button.ui:disabled,input.ui:disabled,textarea.ui:disabled){opacity:.6}:where(input.ui,textarea.ui){appearance:none;border:0 solid var(--ui-border);border-radius:0;background:transparent;width:240px;height:32px;padding:0;line-height:normal;text-align:start}:where(textarea.ui){height:64px;resize:none;white-space:pre-wrap}:where(input.ui){white-space:pre}:where(input.ui::placeholder,textarea.ui::placeholder){color:inherit;opacity:.6}:where(button.ui:focus-visible,input.ui:focus-visible,textarea.ui:focus-visible){outline:2px solid var(--ui-ring);outline-offset:2px}:where([data-tsrust-theme]){color:var(--ui-foreground);background:var(--ui-background);font-size:var(--ui-text-base)}";
pub(crate) fn scope(node:&Node,theme:&crate::theme::Theme,mode:&str)->String {
    if node.theme.is_some() {theme.variables(mode)}else{String::new()}
}
fn element(node:&Node,out:&mut String,theme:&crate::theme::Theme,mode:&str) {
 let Some(node)=node.visible_root() else{return;};let (theme,mode)=node.theme.as_ref().map(|(t,m)|(t,m.as_str())).unwrap_or((theme,mode));let id=node.identity.get();
 let tag=node.tag();
 out.push('<');out.push_str(tag);
 for (key,value) in [("href",&node.semantics.href),("role",&node.semantics.role),("aria-label",&node.semantics.label),("id",&node.semantics.id),("data-slot",&node.semantics.slot),("data-size",&node.semantics.size),("data-variant",&node.semantics.variant),("for",&node.semantics.html_for),("aria-describedby",&node.semantics.described_by),("aria-invalid",&node.semantics.invalid)] {
  if !value.is_empty(){out.push(' ');out.push_str(key);out.push_str("=\"");escape(value,out);out.push('"');}
 }
 if let Some(current)=super::navigation::current_kind(&node.semantics.href){out.push_str(&format!(" aria-current=\"{current}\""));}
 if let Some(index)=node.semantics.tab_index{out.push_str(&format!(" tabindex=\"{index}\""));}
 if node.copy.is_some(){out.push_str(" aria-live=\"polite\" data-copy-state=\"idle\"");}
 if node.kind==Kind::View{out.push_str(" data-tsrust-view");}
 if node.kind==Kind::Text{out.push_str(" data-tsrust-text");}
 if node.semantics.role=="separator"{out.push_str(if node.semantics.orientation=="vertical"{" aria-orientation=\"vertical\" data-orientation=\"vertical\" data-vertical"}else{" aria-orientation=\"horizontal\" data-orientation=\"horizontal\" data-horizontal"});}
 if node.semantics.hidden{out.push_str(" aria-hidden=\"true\"");}
 if matches!(node.kind,Kind::Button|Kind::DisabledButton) && node.semantics.href.is_empty() {out.push_str(" type=\"button\"");}
 if node.is_disabled() {out.push_str(" disabled");}
 if let Some(field)=&node.editable {
  out.push_str(" placeholder=\"");escape(&field.placeholder,out);out.push('"');
  if field.read_only{out.push_str(" readonly");}
  if field.required{out.push_str(" required");}
  for (key,value) in [("name",&field.name),("autocomplete",&field.auto_complete)] {if !value.is_empty(){out.push(' ');out.push_str(key);out.push_str("=\"");escape(value,out);out.push('"');}}
  if !field.multiline{out.push_str(" type=\"text\" value=\"");escape(&field.value,out);out.push('"');}
 }
 if node.theme.is_some(){out.push_str(" data-tsrust-theme");}
 out.push_str(" class=\"ui ");escape(&node.class_name,out);
 out.push_str(&format!("\" data-tsrust-node=\"{id}\" style=\""));escape(&scope(node,theme,mode),out);out.push_str("\">");
 if node.kind==Kind::Input {return;}
 escape(&node.text,out);for child in &node.children {element(child,out,theme,mode);}
 out.push_str("</");out.push_str(tag);out.push('>');
}
pub(crate) fn stylesheet(node:&Node,scope:&str)->String {
 fn collect(node:&Node,theme:&crate::theme::Theme,mode:&str,scope:&str,out:&mut String) {
  let Some(node)=node.visible_root() else{return;};let (theme,mode)=node.theme.as_ref().map(|(t,m)|(t,m.as_str())).unwrap_or((theme,mode));let id=node.identity.get();out.push_str(&crate::style::css(&node.class_name,theme,mode,id,scope));
  for child in &node.children {collect(child,theme,mode,scope,out);}
 }
 let default=crate::theme::Theme::default();let (theme,mode)=node.theme.as_ref().map(|(t,m)|(t,m.as_str())).unwrap_or((&default,"light"));let mut out=BASE.to_owned();
 out.push_str(&format!("body{{{}color:var(--ui-foreground);background:var(--ui-background);font-size:var(--ui-text-base)}}",theme.variables(mode)));
 node.ensure_identities();collect(node,theme,mode,scope,&mut out);out
}
pub(crate) fn document(root:&Node,title:&str)->String {
 let mut out=String::from("<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>");
 escape(title,&mut out);out.push_str("</title><style>");out.push_str(&stylesheet(root,""));out.push_str("</style></head><body>");
 element(root,&mut out,&crate::theme::Theme::default(),"light");out.push_str("</body></html>");out
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

#[cfg(test)]
mod semantic_tests {
    use super::*;
    #[test]
    fn metadata_is_escaped_and_text_divs_keep_leaf_layout() {
        let text=Node::new(Kind::Text,String::new(),"Body".into(),vec![]).unwrap().with_semantics(r#"{"tag":"div","role":"group","label":"<&\"'","id":"exact.id","slot":"card-title"}"#).unwrap();
        let html=document(&text,"Semantics");
        assert!(html.contains("<div role=\"group\" aria-label=\"&lt;&amp;&quot;&#39;\" id=\"exact.id\" data-slot=\"card-title\" data-tsrust-text"));
        assert!(html.contains(":where(.ui[data-tsrust-text]){display:block}:where([data-tsrust-text] .ui){display:inline}:where(strong.ui){font-weight:700}:where(em.ui){font-style:italic}:where(del.ui){text-decoration:line-through}:where([data-tsrust-text] a.ui){text-decoration:underline;background:transparent}"));
        assert!(html.contains(">Body</div>"));
    }
}
