//! Structured source type annotations. Parsing does not establish type validity.
use crate::syntax::{Token, TokenKind, lex};
use crate::{Diagnostic, Mode, Span};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeName {
    pub text: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValueAnnotation {
    pub kind: ValueAnnotationKind,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallableOwnership {
    Default,
    Read,
    Reusable,
    Once,
    Owned,
    OwnedOnce,
    /// Internal shared environment contract; not a source ownership wrapper.
    SharedRead,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParameterAccess {
    pub mode: Mode,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallableParameter {
    pub rest: bool,
    pub name: TypeName,
    pub access: Option<ParameterAccess>,
    pub omission: crate::ParameterOmission,
    pub annotation: ValueAnnotation,
    pub span: Span,
}

/// Literal syntax retains exact numeric spelling until checked type normalization.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LiteralAnnotation {
    String(String),
    Int(String),
    Number(String),
    Boolean(bool),
}
impl LiteralAnnotation {
    pub fn text(&self)->String {
        match self {
            Self::String(value)=>serde_json::to_string(value).expect("string literal spelling"),
            Self::Int(value)|Self::Number(value)=>value.clone(),
            Self::Boolean(value)=>value.to_string(),
        }
    }
    pub(crate) fn copy(&self)->bool { !matches!(self,Self::String(_)) }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValueAnnotationKind {
    /// Checked polymorphic environment identity; never produced by source parsing.
    Polymorphic { origin: usize, declaration: TypeName },
    Named(TypeName),
    Literal(LiteralAnnotation),
    Callable {
        ownership: CallableOwnership,
        parameters: Vec<CallableParameter>,
        result: Box<ValueAnnotation>,
    },
    Applied {
        name: TypeName,
        arguments: Vec<ValueAnnotation>,
    },
    Array(Box<ValueAnnotation>),
    Group(Box<ValueAnnotation>),
    Union(Vec<ValueAnnotation>),
    Tuple(Vec<ValueAnnotation>),
}

impl ValueAnnotation {
    fn nesting(&self) -> usize {
        match &self.kind {
            ValueAnnotationKind::Literal(_) | ValueAnnotationKind::Named(_) | ValueAnnotationKind::Polymorphic { .. } => 0,
            ValueAnnotationKind::Callable {
                parameters, result, ..
            } => {
                1 + parameters
                    .iter()
                    .map(|parameter| parameter.annotation.nesting())
                    .chain(std::iter::once(result.nesting()))
                    .max()
                    .unwrap_or(0)
            }
            ValueAnnotationKind::Applied { arguments, .. }
            | ValueAnnotationKind::Union(arguments)
            | ValueAnnotationKind::Tuple(arguments) => {
                1 + arguments.iter().map(Self::nesting).max().unwrap_or(0)
            }
            ValueAnnotationKind::Array(inner) | ValueAnnotationKind::Group(inner) => {
                1 + inner.nesting()
            }
        }
    }
}

/// Canonical spelling of a function type. `wrapper` is ReadOnly, Mutable or
/// Once; parameters are already rendered as `name: Type`.
pub fn function_type_text(wrapper: Option<&str>, parameters: &[String], result: &str) -> String {
    let function = if parameters.is_empty() {
        format!("Function<{result}>")
    } else {
        format!("Function<{result}, [{}]>", parameters.join(", "))
    };
    match wrapper {
        Some(wrapper) => format!("{wrapper}<{function}>"),
        None => function,
    }
}

/// Canonical spelling of a data argument with stated access.
pub fn access_type_text(mode: Mode, ty: &str) -> String {
    match mode {
        Mode::Copy => ty.to_owned(),
        Mode::Read => format!("ReadOnly<{ty}>"),
        Mode::Mutate => format!("Mutable<{ty}>"),
        Mode::Consume => format!("Owned<{ty}>"),
    }
}

/// Parse one complete annotation, retaining original UTF-8 byte coordinates.
/// Generic names/arity, ownership and available types are checked separately.
pub fn parse(source: &str) -> Result<ValueAnnotation, Diagnostic> {
    let mut tokens = lex(source)?;
    let (value, index) = parse_prefix(&mut tokens, 0, 0)?;
    if tokens[index].kind != TokenKind::End {
        return Err(Diagnostic::new(
            "syntax",
            "Unexpected token after type annotation.",
            tokens[index].span,
        ));
    }
    Ok(value)
}

/// Reject an annotation whose bracket nesting passes the limit before the
/// recursive parser descends into it. The recursive guard alone is reached
/// only after that many frames exist.
fn bounded_nesting(tokens: &[Token], index: usize, depth: usize) -> Result<(), Diagnostic> {
    // One entry per open bracket level: the result arrows seen at that level.
    // Each bracket and each `=>` result is one level of the recursive parser.
    let mut arrows = vec![0usize];
    let mut total = 0usize;
    let limit = |total: usize, token: &Token| {
        if depth + total > 128 {
            Err(Diagnostic::new(
                "unsupported",
                "Type annotation nesting exceeds the prototype limit of 128.",
                token.span,
            ))
        } else {
            Ok(())
        }
    };
    let mut after_parameters = false;
    for token in &tokens[index.min(tokens.len())..] {
        if token.kind == TokenKind::End {
            break;
        }
        // A result arrow follows a parameter list. Any other `=>` at the
        // base level starts an arrow function body, which is not a type.
        let parameters_closed = std::mem::replace(&mut after_parameters, false);
        if token.kind != TokenKind::Symbol {
            continue;
        }
        match token.text.as_str() {
            "=>" if arrows.len() == 1 && !parameters_closed => break,
            ")" if arrows.len() == 2 => {
                after_parameters = true;
                total -= 1 + arrows.pop().unwrap_or(0);
            }
            "(" | "[" | "<" => {
                arrows.push(0);
                total += 1;
                limit(total, token)?;
            }
            ")" | "]" | ">" | ">>" | ">>>" => {
                let closed = if token.text.starts_with('>') { token.text.len() } else { 1 };
                if arrows.len() <= closed {
                    break;
                }
                for _ in 0..closed {
                    total -= 1 + arrows.pop().unwrap_or(0);
                }
            }
            "=>" => {
                *arrows.last_mut().expect("bracket level") += 1;
                total += 1;
                limit(total, token)?;
            }
            "," | ";" | "{" | "=" if arrows.len() == 1 => break,
            _ => {}
        }
    }
    Ok(())
}

/// Consume an annotation from the program's existing lexer stream.
/// Split generic closers only when a type application requires them.
pub(crate) fn parse_prefix(
    tokens: &mut Vec<Token>,
    index: usize,
    depth: usize,
) -> Result<(ValueAnnotation, usize), Diagnostic> {
    bounded_nesting(tokens, index, depth)?;
    let mut parser = AnnotationParser { tokens, index, access_permitted: false, access: None, wrapper_end: None };
    let value = parser.annotation(depth)?;
    Ok((value, parser.index))
}

/// Consume one parameter type. An outermost ReadOnly, Mutable or Owned around
/// data is returned as the parameter's access rather than as a type.
pub(crate) fn parse_parameter_prefix(
    tokens: &mut Vec<Token>,
    index: usize,
    depth: usize,
) -> Result<(Option<ParameterAccess>, ValueAnnotation, usize), Diagnostic> {
    bounded_nesting(tokens, index, depth)?;
    let mut parser = AnnotationParser { tokens, index, access_permitted: true, access: None, wrapper_end: None };
    let value = parser.annotation(depth)?;
    Ok((parser.access.take(), value, parser.index))
}

pub(crate) fn split_closer(tokens: &mut Vec<Token>, index: usize) {
    let token = &tokens[index];
    if token.kind == TokenKind::Symbol
        && matches!(token.text.as_str(), ">>" | ">>>" | ">=" | ">>=" | ">>>=")
    {
        let original = token.clone();
        tokens[index] = Token {
            text: ">".into(),
            span: Span::new(original.span.start, original.span.start + 1),
            kind: TokenKind::Symbol,
        };
        tokens.insert(
            index + 1,
            Token {
                text: original.text[1..].into(),
                span: Span::new(original.span.start + 1, original.span.end),
                kind: TokenKind::Symbol,
            },
        );
        // Maximal lexing reads >=> as >= followed by >. Once this
        // context consumes the generic closer, restore the adjacent arrow.
        if tokens[index + 1].text == "="
            && tokens.get(index + 2).is_some_and(|next| {
                next.text == ">" && tokens[index + 1].span.end == next.span.start
            })
        {
            let end = tokens[index + 2].span.end;
            tokens[index + 1].text = "=>".into();
            tokens[index + 1].span.end = end;
            tokens.remove(index + 2);
        }
    }
}

struct AnnotationParser<'a> {
    tokens: &'a mut Vec<Token>,
    index: usize,
    /// The next outermost member may be a data access wrapper.
    access_permitted: bool,
    access: Option<ParameterAccess>,
    /// End of the access wrapper whose inner type was just returned.
    wrapper_end: Option<usize>,
}
impl AnnotationParser<'_> {
    fn peek(&self) -> &Token {
        &self.tokens[self.index]
    }
    fn is(&self, text: &str) -> bool {
        self.peek().kind == TokenKind::Symbol && self.peek().text == text
    }
    fn take(&mut self) -> Token {
        let token = self.peek().clone();
        if token.kind != TokenKind::End {
            self.index += 1;
        }
        token
    }
    fn error(&self, message: &str) -> Diagnostic {
        Diagnostic::new("syntax", message, self.peek().span)
    }
    fn expect(&mut self, text: &str) -> Result<Token, Diagnostic> {
        if text == ">" {
            split_closer(self.tokens, self.index);
        }
        if !self.is(text) {
            let error = self.error(&format!("Expected {text:?} in type annotation."));
            return Err(if matches!(text, ")" | "]") && self.index > 0 {
                let offset = self.tokens[self.index - 1].span.end;
                error.fix(
                    format!("Insert missing {text}"),
                    Span::new(offset, offset),
                    text,
                )
            } else {
                error
            });
        }
        Ok(self.take())
    }
    fn annotation(&mut self, depth: usize) -> Result<ValueAnnotation, Diagnostic> {
        let first = self.member(depth)?;
        if !self.is("|") {
            return Ok(first);
        }
        let start = first.span.start;
        let mut members = vec![first];
        while self.is("|") {
            self.take();
            members.push(self.member(depth)?);
        }
        let end = members.last().unwrap().span.end;
        let value = ValueAnnotation {
            kind: ValueAnnotationKind::Union(members),
            span: Span::new(start, end),
        };
        self.validate_nesting(&value)?;
        Ok(value)
    }
    fn validate_nesting(&self, value: &ValueAnnotation) -> Result<(), Diagnostic> {
        if value.nesting() > 128 {
            return Err(Diagnostic::new(
                "unsupported",
                "Type annotation nesting exceeds the prototype limit of 128.",
                value.span,
            ));
        }
        Ok(())
    }
    fn symbol(&self, offset: usize, text: &str) -> bool {
        self.tokens
            .get(self.index + offset)
            .is_some_and(|token| token.kind == TokenKind::Symbol && token.text == text)
    }
    fn callable_start(&self) -> bool {
        self.is("(") && (self.symbol(1, ")") || self.symbol(1, "...") || self.symbol(2, ":") || (self.symbol(2, "?") && self.symbol(3, ":")))
    }
    fn callable(&mut self, depth: usize) -> Result<ValueAnnotation, Diagnostic> {
        let start = self.peek().span.start;
        self.expect("(")?;
        let mut parameters = Vec::new();
        if !self.is(")") {
            loop {
                let rest = self.is("...");
                if rest { self.take(); }
                if self.peek().kind != TokenKind::Name
                    || matches!(self.peek().text.as_str(), "true" | "false")
                {
                    return Err(self.error("Expected a callable parameter name."));
                }
                let token = self.take();
                let name = TypeName {
                    text: token.text,
                    span: token.span,
                };
                let omission = if self.is("?") { self.take(); crate::ParameterOmission::Optional } else { crate::ParameterOmission::Required };
                self.expect(":")?;
                self.access_permitted = true;
                let annotation = self.annotation(depth + 1)?;
                self.access_permitted = false;
                let wrapped = self.access.take();
                let end = self.wrapper_end.take().filter(|_| wrapped.is_some()).map_or(annotation.span.end, |end| end.max(annotation.span.end));
                let access = wrapped;
                let span = Span::new(name.span.start, end);
                parameters.push(CallableParameter {
                    name,
                    access,
                    omission,
                    rest,
                    annotation,
                    span,
                });
                if rest && self.is(",") { return Err(Diagnostic::new("rest-parameter", "A rest parameter must be last.", self.peek().span)); }
                if !self.is(",") {
                    break;
                }
                self.take();
            }
        }
        let mut optional = false;
        for parameter in &parameters {
            if parameter.rest {
                if parameter.omission.is_omittable() { return Err(Diagnostic::new("rest-parameter", "A rest parameter cannot be optional.", parameter.name.span)); }
                continue;
            }
            if parameter.omission.is_omittable() { optional = true; }
            else if optional { return Err(Diagnostic::new("parameter-order", "A required parameter cannot follow an optional parameter.", parameter.name.span)); }
        }
        self.expect(")")?;
        self.expect("=>")?;
        let result = Box::new(self.annotation(depth + 1)?);
        let end = result.span.end;
        Ok(ValueAnnotation {
            kind: ValueAnnotationKind::Callable {
                ownership: CallableOwnership::Default,
                parameters,
                result,
            },
            span: Span::new(start, end),
        })
    }
    /// `Function<...>` or an access wrapper applied with `<`.
    fn signature_start(&self) -> bool {
        self.peek().kind == TokenKind::Name
            && matches!(self.peek().text.as_str(), "Function" | "Mutable" | "ReadOnly" | "Once" | "Owned")
            && self.symbol(1, "<")
    }

    /// Parse `Function<R, [A]>` and its wrappers into the existing callable
    /// annotation. A wrapper around data returns the argument access instead;
    /// only a Function argument list can accept that.
    fn signature_type(&mut self, depth: usize) -> Result<(Option<ParameterAccess>, ValueAnnotation), Diagnostic> {
        if depth >= 128 {
            return Err(Diagnostic::new(
                "unsupported",
                "Type annotation nesting exceeds the prototype limit of 128.",
                self.peek().span,
            ));
        }
        let name = self.take();
        self.expect("<")?;
        if name.text == "Function" {
            let result = Box::new(self.annotation(depth + 1)?);
            let parameters = if self.is(",") {
                self.take();
                self.signature_arguments(depth + 1)?
            } else {
                Vec::new()
            };
            if self.is(",") {
                return Err(self.error("Function takes a result type and one optional argument list, such as Function<int, [string]>."));
            }
            let end = self.expect(">")?.span.end;
            let value = ValueAnnotation {
                kind: ValueAnnotationKind::Callable { ownership: CallableOwnership::Default, parameters, result },
                span: Span::new(name.span.start, end),
            };
            self.validate_nesting(&value)?;
            return Ok((None, value));
        }
        let (inner_access, inner) = if self.signature_start() {
            let (access, mut value) = self.signature_type(depth + 1)?;
            // A wrapped Function may still join null or undefined.
            if access.is_none() && self.is("|") {
                let start = value.span.start;
                let mut members = vec![value];
                while self.is("|") {
                    self.take();
                    members.push(self.member(depth + 1)?);
                }
                let end = members.last().unwrap().span.end;
                value = ValueAnnotation { kind: ValueAnnotationKind::Union(members), span: Span::new(start, end) };
            }
            (access, value)
        } else {
            (None, self.annotation(depth + 1)?)
        };
        if self.is(",") {
            return Err(self.error(&format!("{} takes exactly one type.", name.text)));
        }
        let end = self.expect(">")?.span.end;
        let span = Span::new(name.span.start, end);
        if inner_access.is_some() {
            return Err(Diagnostic::new("callback-access", "Access wrappers cannot be nested; use one of ReadOnly, Mutable or Owned.", name.span));
        }
        let mut leaf = &inner;
        while let ValueAnnotationKind::Group(next) = &leaf.kind {
            leaf = next;
        }
        if let ValueAnnotationKind::Callable { ownership, parameters, result } = &leaf.kind {
            if *ownership != CallableOwnership::Default {
                return Err(Diagnostic::new("callback-access", "A Function accepts one ownership wrapper: ReadOnly, Mutable, Once or Owned.", name.span));
            }
            let ownership = match name.text.as_str() {
                "ReadOnly" => CallableOwnership::Read,
                "Mutable" => CallableOwnership::Reusable,
                "Once" => CallableOwnership::OwnedOnce,
                _ => CallableOwnership::Owned,
            };
            let value = ValueAnnotation {
                kind: ValueAnnotationKind::Callable { ownership, parameters: parameters.clone(), result: result.clone() },
                span,
            };
            return Ok((None, value));
        }
        let mode = match name.text.as_str() {
            "ReadOnly" => Mode::Read,
            "Mutable" => Mode::Mutate,
            "Owned" => Mode::Consume,
            _ => return Err(Diagnostic::new("callback-access", "Once applies to a Function; use Owned<T> to transfer a data argument.", name.span)),
        };
        // The type keeps its own span so diagnostics point at it.
        self.wrapper_end = Some(span.end);
        Ok((Some(ParameterAccess { mode, span: name.span }), inner))
    }

    /// `[string, limit?: int, ...values: int[]]`; labels are optional.
    fn signature_arguments(&mut self, depth: usize) -> Result<Vec<CallableParameter>, Diagnostic> {
        if !self.is("[") {
            return Err(self.error("Function arguments are written as a list, such as Function<int, [string, int]>."));
        }
        self.take();
        let mut parameters: Vec<CallableParameter> = Vec::new();
        while !self.is("]") {
            let start = self.peek().span.start;
            let rest = self.is("...");
            if rest {
                self.take();
            }
            let labelled = self.peek().kind == TokenKind::Name
                && (self.symbol(1, ":") || (self.symbol(1, "?") && self.symbol(2, ":")));
            let (name, omission) = if labelled {
                let token = self.take();
                let omission = if self.is("?") { self.take(); crate::ParameterOmission::Optional } else { crate::ParameterOmission::Required };
                self.take();
                (TypeName { text: token.text, span: token.span }, omission)
            } else {
                let at = self.peek().span.start;
                (TypeName { text: format!("arg{}", parameters.len()), span: Span::new(at, at) }, crate::ParameterOmission::Required)
            };
            self.access_permitted = true;
            let annotation = self.annotation(depth)?;
            self.access_permitted = false;
            let access = self.access.take();
            let end = self.wrapper_end.take().filter(|_| access.is_some()).map_or(annotation.span.end, |end| end.max(annotation.span.end));
            if rest && omission.is_omittable() {
                return Err(Diagnostic::new("rest-parameter", "A rest parameter cannot be optional.", name.span));
            }
            if !rest && !omission.is_omittable() && parameters.iter().any(|parameter| parameter.omission.is_omittable()) {
                return Err(Diagnostic::new("parameter-order", "A required parameter cannot follow an optional parameter.", name.span));
            }
            let span = Span::new(start, end);
            parameters.push(CallableParameter { rest, name, access, omission, annotation, span });
            if rest && self.is(",") {
                return Err(Diagnostic::new("rest-parameter", "A rest parameter must be last.", self.peek().span));
            }
            if !self.is(",") {
                break;
            }
            self.take();
        }
        self.expect("]")?;
        Ok(parameters)
    }

    fn member(&mut self, depth: usize) -> Result<ValueAnnotation, Diagnostic> {
        if let Some(replacement) = match self.peek().text.as_str() {
            "read" => Some("ReadOnly<T>"),
            "mutate" => Some("Mutable<T>"),
            "consume" | "owned" => Some("Owned<T>"),
            "once" => Some("Once<Function<R>>"),
            _ => None,
        } {
            if self.tokens.get(self.index + 1).is_some_and(|t| t.kind == TokenKind::Name || t.text == "(") {
            return Err(Diagnostic::new("retired-access-keyword", format!("The `{}` type keyword is retired; use {replacement} instead.", self.peek().text), self.peek().span));
            }
        }

        if depth >= 128 {
            return Err(Diagnostic::new(
                "unsupported",
                "Type annotation nesting exceeds the prototype limit of 128.",
                self.peek().span,
            ));
        }
        let permitted = std::mem::take(&mut self.access_permitted);
        let mut value = if self.callable_start() {
            self.callable(depth)?
        } else if self.is("[") {
            let start = self.take().span.start;
            let mut positions = Vec::new();
            while !self.is("]") {
                positions.push(self.annotation(depth + 1)?);
                if !self.is(",") { break; }
                self.take();
            }
            let end = self.expect("]")?.span.end;
            ValueAnnotation { kind: ValueAnnotationKind::Tuple(positions), span: Span::new(start, end) }
        } else if self.is("(") {
            let start = self.take().span.start;
            let inner = self.annotation(depth + 1)?;
            let end = self.expect(")")?.span.end;
            ValueAnnotation {
                kind: ValueAnnotationKind::Group(Box::new(inner)),
                span: Span::new(start, end),
            }
        } else if self.signature_start() {
            let (access, value) = self.signature_type(depth)?;
            if let Some(access) = access {
                if permitted {
                    self.access = Some(access);
                } else {
                return Err(Diagnostic::new(
                    "callback-access",
                    "ReadOnly, Mutable and Owned describe a Function or a parameter; this position cannot carry data access.",
                    access.span,
                ));
                }
            }
            value
        } else if matches!(self.peek().kind,TokenKind::String|TokenKind::Integer|TokenKind::Number)
            || self.peek().kind==TokenKind::Name&&matches!(self.peek().text.as_str(),"true"|"false")
            || self.is("-")&&self.tokens.get(self.index+1).is_some_and(|token|matches!(token.kind,TokenKind::Integer|TokenKind::Number)) {
            let start=self.peek().span.start;
            let negative=self.is("-");if negative{self.take();}
            let token=self.take();
            let numeric=if negative{format!("-{}",token.text)}else{token.text.clone()};
            let literal=match token.kind {
                TokenKind::String=>LiteralAnnotation::String(token.text),
                TokenKind::Integer=>LiteralAnnotation::Int(numeric),
                TokenKind::Number=>LiteralAnnotation::Number(numeric),
                _=>LiteralAnnotation::Boolean(token.text=="true"),
            };
            ValueAnnotation{kind:ValueAnnotationKind::Literal(literal),span:Span::new(start,token.span.end)}
        } else {
            if self.peek().kind != TokenKind::Name
                || matches!(self.peek().text.as_str(), "true" | "false")
            {
                return Err(self.error("Expected a type name."));
            }
            let token = self.take();
            let name = TypeName {
                text: token.text,
                span: token.span,
            };
            let start = name.span.start;
            if self.is("<") && !matches!(name.text.as_str(), "int" | "number" | "string" | "boolean" | "void" | "null" | "undefined") {
                self.take();
                let mut arguments = vec![self.annotation(depth + 1)?];
                while self.is(",") {
                    self.take();
                    arguments.push(self.annotation(depth + 1)?);
                }
                let end = self.expect(">")?.span.end;
                ValueAnnotation {
                    kind: ValueAnnotationKind::Applied { name, arguments },
                    span: Span::new(start, end),
                }
            } else {
                ValueAnnotation {
                    span: name.span,
                    kind: ValueAnnotationKind::Named(name),
                }
            }
        };
        self.validate_nesting(&value)?;
        while self.is("[") {
            if value.nesting() >= 128 {
                return Err(Diagnostic::new(
                    "unsupported",
                    "Type annotation nesting exceeds the prototype limit of 128.",
                    self.peek().span,
                ));
            }
            self.take();
            let end = self.expect("]")?.span.end;
            let start = value.span.start;
            value = ValueAnnotation {
                kind: ValueAnnotationKind::Array(Box::new(value)),
                span: Span::new(start, end),
            };
        }
        Ok(value)
    }
}
