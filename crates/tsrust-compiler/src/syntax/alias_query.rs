//! Editor-facing alias queries use the compiler's parser and substitution rules.
use super::*;
use crate::type_syntax::{CallableOwnership, ValueAnnotation, ValueAnnotationKind as Kind};
#[derive(Clone, Debug, Default)]
pub struct TypeAliasCatalog {
    aliases: crate::type_aliases::Aliases,
}
#[derive(Clone, Debug)]
pub struct TypeAliasInfo {
    pub name: String,
    pub span: Span,
    pub extent: Span,
    pub parameters: Vec<String>,
    pub signature: String,
}
impl TypeAliasCatalog {
    pub fn parse(source: &str) -> Result<Self, Diagnostic> {
        let mut parser = Parser::new(source, Default::default(), Default::default())?;
        parser.collect_type_aliases()?;
        Ok(Self {
            aliases: parser.aliases,
        })
    }
    pub fn has_generics(&self) -> bool {
        self.aliases
            .values()
            .any(|alias| !alias.parameters.is_empty())
    }
    pub fn expand(&self, text: &str) -> Result<String, Diagnostic> {
        let mut value = crate::type_syntax::parse(text)?;
        crate::type_aliases::expand(&mut value, &self.aliases)?;
        Ok(render(&value))
    }
    pub fn declarations(&self) -> Vec<TypeAliasInfo> {
        let mut values = self
            .aliases
            .iter()
            .map(|(name, alias)| {
                let parameters = alias
                    .parameters
                    .iter()
                    .map(|p| match &p.default {
                        Some(value) => format!("{} = {}", p.name.text, render(value)),
                        None => p.name.text.clone(),
                    })
                    .collect::<Vec<_>>();
                let suffix = if parameters.is_empty() {
                    String::new()
                } else {
                    format!("<{}>", parameters.join(", "))
                };
                TypeAliasInfo {
                    name: name.clone(),
                    span: alias.span,
                    extent: Span::new(alias.span.start, alias.annotation.span.end),
                    parameters: alias
                        .parameters
                        .iter()
                        .map(|p| p.name.text.clone())
                        .collect(),
                    signature: format!("type {name}{suffix} = {}", render(&alias.annotation)),
                }
            })
            .collect::<Vec<_>>();
        values.sort_by(|a, b| a.name.cmp(&b.name));
        values
    }
    /// The importer supplies its existing type-name mapping; parameters remain lexical.
    pub fn import_as(
        &mut self,
        other: &Self,
        exported: &str,
        local: &str,
        rename: impl Fn(&str) -> String,
    ) -> bool {
        let Some(alias) = other
            .aliases
            .get(exported)
            .filter(|a| !a.parameters.is_empty())
        else {
            return false;
        };
        let mut alias = alias.clone();
        crate::type_aliases::link_alias(&mut alias, &rename);
        self.aliases.insert(local.into(), alias);
        true
    }
}
fn render(root: &ValueAnnotation) -> String {
    enum Work<'a> {
        Visit(&'a ValueAnnotation),
        Finish(&'a ValueAnnotation, usize),
    }
    let mut work = vec![Work::Visit(root)];
    let mut values = Vec::<String>::new();
    while let Some(next) = work.pop() {
        match next {
            Work::Visit(value) => {
                let children = match &value.kind {
                    Kind::Named(name) => {
                        values.push(name.text.clone());
                        continue;
                    }
                    Kind::Polymorphic { declaration, .. } => {
                        values.push(declaration.text.clone());
                        continue;
                    }
                    Kind::Literal(value) => {
                        values.push(value.text());
                        continue;
                    }
                    Kind::Array(v) | Kind::Group(v) => vec![v.as_ref()],
                    Kind::Applied { arguments, .. }
                    | Kind::Union(arguments)
                    | Kind::Tuple(arguments) => arguments.iter().collect(),
                    Kind::Callable {
                        parameters, result, ..
                    } => parameters
                        .iter()
                        .map(|p| &p.annotation)
                        .chain(std::iter::once(result.as_ref()))
                        .collect(),
                };
                work.push(Work::Finish(value, values.len()));
                work.extend(children.into_iter().rev().map(Work::Visit));
            }
            Work::Finish(value, start) => {
                let mut children = values.drain(start..).collect::<Vec<_>>();
                let text = match &value.kind {
                    Kind::Group(_) => children.remove(0),
                    Kind::Array(inner) => {
                        let mut inner = inner.as_ref();
                        while let Kind::Group(value) = &inner.kind {
                            inner = value;
                        }
                        let text = &children[0];
                        if matches!(inner.kind, Kind::Union(_) | Kind::Callable { .. }) {
                            format!("({text})[]")
                        } else {
                            format!("{text}[]")
                        }
                    }
                    Kind::Applied { name, .. } => format!("{}<{}>", name.text, children.join(", ")),
                    Kind::Union(_) => children.join(" | "),
                    Kind::Tuple(_) => format!("[{}]", children.join(", ")),
                    Kind::Callable {
                        ownership,
                        parameters,
                        ..
                    } => {
                        let result = children.pop().unwrap();
                        let params = parameters
                            .iter()
                            .zip(children)
                            .map(|(p, ty)| {
                                let ty = p.access.as_ref().map_or(ty.clone(), |access| {
                                    crate::type_syntax::access_type_text(access.mode, &ty)
                                });
                                format!(
                                    "{}{}{}: {ty}",
                                    if p.rest { "..." } else { "" },
                                    p.name.text,
                                    if p.omission == crate::ParameterOmission::Required {
                                        ""
                                    } else {
                                        "?"
                                    }
                                )
                            })
                            .collect::<Vec<_>>();
                        let wrapper = match ownership {
                            CallableOwnership::Task => Some("Task"),
                            CallableOwnership::Read => Some("ReadOnly"),
                            CallableOwnership::Reusable => Some("Mutable"),
                            CallableOwnership::Once => Some("Once"),
                            CallableOwnership::Owned => Some("Owned"),
                            CallableOwnership::OwnedOnce => Some("Once"),
                            _ => None,
                        };
                        crate::type_syntax::function_type_text(wrapper, &params, &result)
                    }
                    _ => unreachable!(),
                };
                values.push(text);
            }
        }
    }
    values.pop().expect("annotation text")
}

/// Formatting uses the same header parser, including packed generic closers.
pub(crate) fn alias_parameter_spans(tokens: &[Token]) -> Vec<Span> {
    let mut parser = Parser::new("", Default::default(), Default::default()).expect("empty source");
    parser.tokens = tokens.to_vec();
    let mut spans = Vec::new();
    let mut index = 0;
    let mut braces = 0usize;
    while index + 2 < parser.tokens.len() {
        let token = &parser.tokens[index];
        if token.kind == TokenKind::Symbol {
            match token.text.as_str() {
                "{" => braces += 1,
                "}" => braces = braces.saturating_sub(1),
                _ => {}
            }
        }
        if braces == 0 && token.text == "type" && parser.tokens[index + 2].text == "<" {
            let start = parser.tokens[index + 2].span.start;
            if let Ok((_, after)) = parser.alias_parameters_at(index + 2) {
                spans.push(Span::new(start, parser.tokens[after - 1].span.end));
                index = after;
                continue;
            }
        }
        index += 1;
    }
    spans
}
