//! Source comments are descriptions, never an alternative type checker.
pub(crate) fn clean(comment:&str)->String {
    let body=if let Some(body)=comment.strip_prefix("/*") {
        let body=body.strip_suffix("*/").unwrap_or(body);
        body.strip_prefix('*').unwrap_or(body)
    } else {comment.strip_prefix("//").unwrap_or(comment).trim_start_matches('/')};
    body.lines().map(|line| {
        let trimmed=line.trim_start();
        if let Some(rest)=trimmed.strip_prefix('*') {rest.strip_prefix(' ').unwrap_or(rest)} else {line.strip_prefix(' ').unwrap_or(line)}
    }).collect::<Vec<_>>().join("\n").trim().to_owned()
}
fn blocks(docs:&str)->Vec<(String,String)> {
    let mut blocks=vec![(String::new(),String::new())];let mut fence=None;
    for line in docs.lines() {
        let trimmed=line.trim_start();
        let delimiter=if trimmed.starts_with("```"){Some('`')}else if trimmed.starts_with("~~~"){Some('~')}else{None};
        if let Some(delimiter)=delimiter {if fence==Some(delimiter){fence=None}else if fence.is_none(){fence=Some(delimiter)}}
        if fence.is_none() && delimiter.is_none() && trimmed.starts_with('@') {
            let (tag,body)=trimmed.split_once(char::is_whitespace).unwrap_or((trimmed,""));
            blocks.push((tag.into(),body.trim_start().into()));
        } else {
            let body=&mut blocks.last_mut().unwrap().1;
            if !body.is_empty(){body.push('\n');}body.push_str(line);
        }
    }
    blocks
}
fn without_type(text:&str)->&str {
    let text=text.trim_start();
    if !text.starts_with('{'){return text;}
    let mut depth=0;
    for (at,ch) in text.char_indices(){match ch {'{'=>depth+=1,'}'=>{depth-=1;if depth==0{return text[at+1..].trim_start();}},_=>{}}}
    text
}
fn parameter(text:&str)->(&str,&str) {
    let text=without_type(text);let (name,body)=text.split_once(char::is_whitespace).unwrap_or((text,""));
    (name.trim_start_matches('[').trim_end_matches(']').split('=').next().unwrap_or(name).trim_start_matches("..."),body.trim_start().strip_prefix('-').unwrap_or(body.trim_start()).trim_start())
}
/// Markdown soft breaks collapse in editor hovers. Preserve authored prose lines
/// as hard breaks, but never alter fenced/indented code or paragraph boundaries.
fn lines(text:&str)->String {
    let mut fence=None;let input=text.trim().lines().collect::<Vec<_>>();let mut out=String::new();
    for (index,line) in input.iter().enumerate() {
        let trimmed=line.trim_start();
        let delimiter=if trimmed.starts_with("```"){Some('`')}else if trimmed.starts_with("~~~"){Some('~')}else{None};
        if let Some(delimiter)=delimiter {if fence==Some(delimiter){fence=None}else if fence.is_none(){fence=Some(delimiter)}}
        out.push_str(line);
        if index+1<input.len() {
            if fence.is_none() && delimiter.is_none() && !line.trim().is_empty() && !input[index+1].trim().is_empty() && !line.starts_with("    ") && !line.ends_with("  ") {out.push_str("  ");}
            out.push('\n');
        }
    }
    out
}
pub(crate) fn render(docs:&str)->String {
    blocks(docs).into_iter().filter_map(|(tag,body)| {
        if tag.is_empty(){return (!body.trim().is_empty()).then(||lines(&body));}
        let (title,body)=match tag.as_str() {
            "@param"|"@arg"|"@argument"=>{let (name,body)=parameter(&body);(format!("Parameter `{name}`"),body)},
            "@returns"|"@return"=>("Returns".into(),without_type(&body)),
            "@throws"|"@exception"=>("Throws".into(),body.as_str()),
            "@example"=>("Example".into(),body.as_str()),
            "@deprecated"=>("Deprecated".into(),body.as_str()),
            _=>(tag.clone(),body.as_str()),
        };
        Some(format!("**{title}**\n\n{}",lines(body)))
    }).collect::<Vec<_>>().join("\n\n")
}
pub(crate) fn parameter_description(docs:&str,name:&str)->String {
    blocks(docs).into_iter().find_map(|(tag,body)| {
        if !matches!(tag.as_str(),"@param"|"@arg"|"@argument"){return None;}
        let (found,description)=parameter(&body);(found==name).then(||lines(description))
    }).unwrap_or_default()
}

#[cfg(test)]
mod tests {
 use super::*;
 #[test]
 fn fenced_tags_indentation_and_markdown_survive() {
  let docs=clean("/**\r\n * **Bold** text.\r\n * Next line.\r\n *\r\n * @example\r\n * ```tsrust\r\n * @notATag\r\n *   nested();\r\n * ```\r\n */");
  let rendered=render(&docs);
  assert!(rendered.contains("**Bold** text.  \nNext line."));
  assert!(rendered.contains("```tsrust\n@notATag\n  nested();\n```"));
  assert!(!rendered.contains("**@notATag**"));
 }
 #[test]
 fn parameter_tags_match_exact_names_and_accept_optional_and_legacy_types() {
  let docs="@param nameExtra - Wrong parameter.\n@param {{value: string}} [name=guest] - First line.\n  Second line.\n@return {string} Result description.\n@custom Keep unknown information.";
  assert_eq!(parameter_description(docs,"name"),"First line.  \n  Second line.");
  assert!(render(docs).contains("**Returns**\n\nResult description."));
  assert!(render(docs).contains("**@custom**\n\nKeep unknown information."));
  assert!(parameter_description(docs,"missing").is_empty());
 }
 #[test]
 fn ordinary_line_comments_preserve_paragraphs() {
  assert_eq!(clean("// First line."),"First line.");
  assert_eq!(render("First.\nSecond.\n\nThird."),"First.  \nSecond.\n\nThird.");
 }
}
