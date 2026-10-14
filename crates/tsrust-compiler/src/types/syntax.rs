//! Structured source type annotations. Parsing does not establish type validity.
use crate::syntax::{NestingPolicy, Token, TokenKind};
#[cfg(test)]
use crate::syntax::lex;
use crate::{Diagnostic, Mode, Span};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeName {
    pub text: String,
    pub span: Span,
}

#[derive(Debug, Eq)]
pub struct ValueAnnotation {
    pub kind: ValueAnnotationKind,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallableOwnership {
    /// Immediate registration callback with invocation-time program access.
    ProgramReusable,
    /// Internal test-registration contract; never parsed from user source.
    Program,
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
    parse_with(source, NestingPolicy::CONSUMERS)
}

/// Parse one complete annotation under an explicit admission policy.
pub(crate) fn parse_with(source: &str, policy: NestingPolicy) -> Result<ValueAnnotation, Diagnostic> {
    let mut tokens = crate::syntax::lex_with(source, policy)?;
    let (value, index) = parse_prefix(&mut tokens, 0, 0, policy)?;
    if tokens[index].kind != TokenKind::End {
        return Err(Diagnostic::new(
            "syntax",
            "Unexpected token after type annotation.",
            tokens[index].span,
        ));
    }
    Ok(value)
}

/// Reject an annotation whose bracket nesting passes the admission bound
/// before any annotation work is queued. Only a bounded policy scans.
fn bounded_nesting(tokens: &[Token], index: usize, depth: usize, policy: NestingPolicy) -> Result<(), Diagnostic> {
    if !policy.bounded() {
        return Ok(());
    }
    // One entry per open bracket level: the result arrows seen at that level.
    // Each bracket and each `=>` result is one level of the recursive parser.
    let mut arrows = vec![0usize];
    let mut total = 0usize;
    let limit = |total: usize, token: &Token| {
        if policy.exceeded(depth + total) {
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
    policy: NestingPolicy,
) -> Result<(ValueAnnotation, usize), Diagnostic> {
    bounded_nesting(tokens, index, depth, policy)?;
    let mut parser = AnnotationParser::new(tokens, index, policy, false, None);
    let value = parser.annotation(depth);
    let end = parser.finish();
    Ok((value?, end))
}

/// Consume one parameter type. An outermost ReadOnly, Mutable or Owned around
/// data is returned as the parameter's access rather than as a type.
pub(crate) fn parse_parameter_prefix(
    tokens: &mut Vec<Token>,
    index: usize,
    depth: usize,
    policy: NestingPolicy,
) -> Result<(Option<ParameterAccess>, ValueAnnotation, usize), Diagnostic> {
    bounded_nesting(tokens, index, depth, policy)?;
    let mut parser = AnnotationParser::new(tokens, index, policy, true, None);
    let value = parser.annotation(depth);
    let end = parser.finish();
    Ok((parser.access.take(), value?, end))
}

/// Type arguments after `<`, through the closing `>`. Reading the list here
/// lets one packed closer end both the last argument and the list. Returns
/// the cursor to continue from: after the list, or where a failed argument
/// started.
pub(crate) fn parse_arguments(
    tokens: &mut Vec<Token>,
    index: usize,
    depth: usize,
    policy: NestingPolicy,
) -> (Result<Vec<ValueAnnotation>, Diagnostic>, usize) {
    let mut parser = AnnotationParser::new(tokens, index, policy, false, None);
    let mut arguments = Vec::new();
    let mut failed = None;
    let result = loop {
        let start = parser.index;
        let argument = bounded_nesting(parser.tokens, start, depth, policy).and_then(|()| parser.annotation(depth));
        match argument {
            Ok(argument) => arguments.push(argument),
            Err(error) => {
                failed = Some(start);
                break Err(error);
            }
        }
        if arguments.len() > 128 {
            break Err(Diagnostic::new("generic-limit", "Explicit type argument count exceeds 128.", parser.peek().span));
        }
        if parser.is(",") {
            parser.take();
            continue;
        }
        if parser.peek().kind == TokenKind::Symbol && matches!(parser.peek().text.as_str(), ">>" | ">>>" | ">=" | ">>=" | ">>>=") {
            parser.close_packed();
        } else if parser.is(">") {
            parser.take();
        } else {
            break Err(Diagnostic::new("syntax", "Expected \">\".", parser.peek().span));
        }
        break Ok(arguments);
    };
    let end = parser.finish();
    (result, failed.unwrap_or(end))
}

/// Whether one result annotation at `index` is followed by `=>`. The probe
/// reads packed closers in place, so the token stream is unchanged.
pub(crate) fn arrow_follows_prefix(tokens: &mut Vec<Token>, index: usize, policy: NestingPolicy) -> bool {
    // A type past the admission bound is still a type; the arrow parser reports it.
    if bounded_nesting(tokens, index, 0, policy).is_err() {
        return true;
    }
    let mut parser = AnnotationParser::new(tokens, index, policy, false, None);
    match parser.annotation(0) {
        Ok(_) => parser.peek().text == "=>",
        Err(error) => error.code == "unsupported",
    }
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

struct CloserEdit {
    index: usize,
    previous: Vec<Token>,
    replaced: usize,
}

/// Probe one formatting annotation using reusable scratch tokens. Roll back
/// every packed-closer edit, including edits made before a parse failure.
/// Independent probes therefore see the exact original lexical stream.
pub(crate) fn parameter_prefix_span(tokens: &mut Vec<Token>, index: usize) -> Result<(Span, bool), Diagnostic> {
    let policy = NestingPolicy::CONSUMERS;
    bounded_nesting(tokens, index, 0, policy)?;
    let mut edits = Vec::new();
    let outcome = {
        let mut parser = AnnotationParser::new(tokens, index, policy, true, Some(&mut edits));
        let annotation = parser.annotation(0);
        let end = parser.finish();
        annotation.map(|annotation| {
            let span = Span::new(parser.tokens[index].span.start,
                parser.tokens[end - 1].span.end.max(annotation.span.end));
            let call = parser.tokens.get(end).is_some_and(|next| next.text == "(");
            (span, call)
        })
    };
    for edit in edits.into_iter().rev() {
        tokens.splice(edit.index..edit.index + edit.replaced, edit.previous);
    }
    outcome
}

struct AnnotationParser<'a> {
    tokens: &'a mut Vec<Token>,
    split_log: Option<&'a mut Vec<CloserEdit>>,
    index: usize,
    policy: NestingPolicy,
    /// The next outermost member may be a data access wrapper.
    access_permitted: bool,
    access: Option<ParameterAccess>,
    /// End of the access wrapper whose inner type was just returned.
    wrapper_end: Option<usize>,
    /// Suspended grammar positions. Every nested annotation is a request on
    /// this stack; no production calls another production.
    work: Vec<Work>,
    values: Vec<ValueAnnotation>,
    /// Access returned beside each completed `Function<...>` or wrapper type.
    signatures: Vec<Option<ParameterAccess>>,
    /// Packed closers such as `>>` that closed one bracket per `>`, in token
    /// order. The token stream is rewritten once, when parsing ends.
    closers: Vec<PackedCloser>,
    /// The unconsumed part of the packed closer at the cursor.
    remainder: Option<Token>,
}

/// One lexer token that starts with several `>` and closed type brackets.
struct PackedCloser {
    index: usize,
    /// Leading `>` characters that became separate closers.
    consumed: usize,
    /// The remaining `=` joined the adjacent `>` as `=>`; that token ends here.
    arrow_end: Option<usize>,
}

/// Parameter syntax read before its annotation is requested.
struct ParameterHead {
    start: usize,
    rest: bool,
    name: TypeName,
    omission: crate::ParameterOmission,
}

struct CallableFrame {
    start: usize,
    parameters: Vec<CallableParameter>,
    depth: usize,
}

/// `Function<R, [A]>` after its result; `depth` belongs to the argument list.
struct FunctionFrame {
    name: Token,
    result: Box<ValueAnnotation>,
    parameters: Vec<CallableParameter>,
    depth: usize,
}

enum Work {
    Annotation { depth: usize },
    UnionHead { depth: usize },
    UnionNext { start: usize, members: Vec<ValueAnnotation>, depth: usize, validate: bool },
    Member { depth: usize },
    MemberSuffix,
    MemberSignature { permitted: bool },
    Tuple { start: usize, positions: Vec<ValueAnnotation>, depth: usize },
    Group { start: usize },
    Applied { name: TypeName, arguments: Vec<ValueAnnotation>, depth: usize },
    CallableParameter { frame: CallableFrame, head: ParameterHead },
    CallableResult { start: usize, parameters: Vec<CallableParameter> },
    Signature { depth: usize },
    FunctionResult { name: Token, depth: usize },
    FunctionArgument { frame: FunctionFrame, head: ParameterHead },
    WrapperSignature { name: Token, depth: usize },
    WrapperFinish { name: Token, access: Option<ParameterAccess> },
}

impl<'a> AnnotationParser<'a> {
    fn new(tokens: &'a mut Vec<Token>, index: usize, policy: NestingPolicy, access_permitted: bool, split_log: Option<&'a mut Vec<CloserEdit>>) -> Self {
        Self { tokens, split_log, index, policy, access_permitted, access: None, wrapper_end: None, work: Vec::new(), values: Vec::new(), signatures: Vec::new(), closers: Vec::new(), remainder: None }
    }
    fn peek(&self) -> &Token {
        self.remainder.as_ref().unwrap_or(&self.tokens[self.index])
    }
    fn is(&self, text: &str) -> bool {
        self.peek().kind == TokenKind::Symbol && self.peek().text == text
    }
    fn take(&mut self) -> Token {
        if let Some(rest) = self.remainder.take() {
            // The rest of a packed closer, with the `>` it may have joined.
            let joined = self.closers.last().is_some_and(|closer| closer.arrow_end.is_some());
            self.index += 1 + usize::from(joined);
            return rest;
        }
        let token = self.peek().clone();
        if token.kind != TokenKind::End {
            self.index += 1;
        }
        token
    }

    /// Close one bracket with the first `>` of a packed closer. The token
    /// stream keeps its length until `finish`; the cursor sees the rest of
    /// the token exactly as a split would leave it.
    fn close_packed(&mut self) -> Token {
        if self.remainder.is_none() {
            self.closers.push(PackedCloser { index: self.index, consumed: 0, arrow_end: None });
        }
        let closer = self.closers.last_mut().expect("packed closer at the cursor");
        closer.consumed += 1;
        let original = &self.tokens[self.index];
        let at = original.span.start + closer.consumed;
        let mut rest = Token { text: original.text[closer.consumed..].into(), span: Span::new(at, original.span.end), kind: TokenKind::Symbol };
        // Maximal lexing reads >=> as >= followed by >. Once this
        // context consumes the generic closer, restore the adjacent arrow.
        if rest.text == "="
            && let Some(next) = self.tokens.get(self.index + 1)
            && next.text == ">"
            && rest.span.end == next.span.start
        {
            rest.text = "=>".into();
            rest.span.end = next.span.end;
            closer.arrow_end = Some(next.span.end);
        }
        self.remainder = Some(rest);
        Token { text: ">".into(), span: Span::new(at - 1, at), kind: TokenKind::Symbol }
    }

    /// End the parse at a token boundary. A packed closer that was consumed
    /// whole stays one token behind the cursor. When the cursor is inside a
    /// packed closer, that token is rewritten as its separate `>` closers and
    /// its rest, so the caller continues at an ordinary token. Returns the
    /// cursor in the resulting stream.
    fn finish(&mut self) -> usize {
        if self.remainder.is_none() {
            self.closers.clear();
            return self.index;
        }
        let keep = self.closers.len() - 1;
        self.closers.drain(..keep);
        let Some(first) = self.closers.first().map(|closer| closer.index) else {
            return self.index;
        };
        let last = self.closers.last().expect("packed closer");
        let affected = last.index + 1 + usize::from(last.arrow_end.is_some());
        let previous = self.split_log.is_some().then(|| self.tokens[first..affected].to_vec());
        let mut closers = std::mem::take(&mut self.closers).into_iter().peekable();
        let mut rewritten = Vec::with_capacity(affected - first + 2);
        let mut cursor = None;
        let mut joined = false;
        for physical in first..affected {
            if std::mem::take(&mut joined) {
                continue;
            }
            let vacant = Token { text: String::new(), span: Span::default(), kind: TokenKind::End };
            let token = std::mem::replace(&mut self.tokens[physical], vacant);
            let Some(closer) = closers.next_if(|closer| closer.index == physical) else {
                rewritten.push(token);
                continue;
            };
            let start = token.span.start;
            rewritten.extend((0..closer.consumed).map(|consumed| Token { text: ">".into(), span: Span::new(start + consumed, start + consumed + 1), kind: TokenKind::Symbol }));
            if physical == self.index && self.remainder.is_some() {
                cursor = Some(first + rewritten.len());
            }
            let rest = Span::new(start + closer.consumed, closer.arrow_end.unwrap_or(token.span.end));
            joined = closer.arrow_end.is_some();
            let text = if joined { "=>".into() } else { token.text[closer.consumed..].into() };
            rewritten.push(Token { text, span: rest, kind: TokenKind::Symbol });
        }
        let replaced = rewritten.len();
        // The cursor is inside the last packed closer or after every rewritten token.
        self.index = cursor.unwrap_or(self.index + replaced - (affected - first));
        self.remainder = None;
        self.tokens.splice(first..affected, rewritten);
        if let (Some(log), Some(previous)) = (&mut self.split_log, previous) {
            log.push(CloserEdit { index: first, previous, replaced });
        }
        self.index
    }
    fn error(&self, message: &str) -> Diagnostic {
        Diagnostic::new("syntax", message, self.peek().span)
    }
    fn expect(&mut self, text: &str) -> Result<Token, Diagnostic> {
        if text == ">" && self.peek().kind == TokenKind::Symbol && matches!(self.peek().text.as_str(), ">>" | ">>>" | ">=" | ">>=" | ">>>=") {
            return Ok(self.close_packed());
        }
        if !self.is(text) {
            let error = self.error(&format!("Expected {text:?} in type annotation."));
            return Err(if matches!(text, ")" | "]") && self.index > 0 {
                // After part of a packed closer, the last accepted token is its `>`.
                let offset = self.remainder.as_ref().map_or_else(|| self.tokens[self.index - 1].span.end, |rest| rest.span.start);
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
    fn nesting_limit(span: Span) -> Diagnostic {
        Diagnostic::new("unsupported", "Type annotation nesting exceeds the prototype limit of 128.", span)
    }
    fn validate_nesting(&self, value: &ValueAnnotation) -> Result<(), Diagnostic> {
        if self.policy.bounded() && self.policy.exceeded(value.nesting()) {
            return Err(Self::nesting_limit(value.span));
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
    /// `Function<...>` or an access wrapper applied with `<`.
    fn signature_start(&self) -> bool {
        self.peek().kind == TokenKind::Name
            && matches!(self.peek().text.as_str(), "Function" | "Mutable" | "ReadOnly" | "Once" | "Owned")
            && self.symbol(1, "<")
    }

    /// Parse one annotation at `depth`. Suspended parents stay on `work`; the
    /// native call depth does not grow with the annotation.
    fn annotation(&mut self, depth: usize) -> Result<ValueAnnotation, Diagnostic> {
        self.work.push(Work::Annotation { depth });
        while let Some(work) = self.work.pop() {
            self.step(work)?;
        }
        Ok(self.values.pop().expect("completed annotation"))
    }

    fn request(&mut self, parent: Work, depth: usize) {
        self.work.push(parent);
        self.work.push(Work::Annotation { depth });
    }

    fn step(&mut self, work: Work) -> Result<(), Diagnostic> {
        match work {
            Work::Annotation { depth } => {
                self.work.push(Work::UnionHead { depth });
                self.work.push(Work::Member { depth });
            }
            Work::UnionHead { depth } => {
                if self.is("|") {
                    let first = self.values.pop().expect("first union member");
                    self.take();
                    self.work.push(Work::UnionNext { start: first.span.start, members: vec![first], depth, validate: true });
                    self.work.push(Work::Member { depth });
                }
            }
            Work::UnionNext { start, mut members, depth, validate } => {
                members.push(self.values.pop().expect("union member"));
                if self.is("|") {
                    self.take();
                    self.work.push(Work::UnionNext { start, members, depth, validate });
                    self.work.push(Work::Member { depth });
                } else {
                    let end = members.last().unwrap().span.end;
                    let value = ValueAnnotation { kind: ValueAnnotationKind::Union(members), span: Span::new(start, end) };
                    if validate {
                        self.validate_nesting(&value)?;
                    }
                    self.values.push(value);
                }
            }
            Work::Member { depth } => self.member(depth)?,
            Work::MemberSuffix => {
                let mut value = self.values.pop().expect("type member");
                self.validate_nesting(&value)?;
                while self.is("[") {
                    if self.policy.bounded() && self.policy.reached(value.nesting()) {
                        return Err(Self::nesting_limit(self.peek().span));
                    }
                    self.take();
                    let end = self.expect("]")?.span.end;
                    let start = value.span.start;
                    value = ValueAnnotation { kind: ValueAnnotationKind::Array(Box::new(value)), span: Span::new(start, end) };
                }
                self.values.push(value);
            }
            Work::MemberSignature { permitted } => {
                if let Some(access) = self.signatures.pop().expect("signature access") {
                    if !permitted {
                        return Err(Diagnostic::new(
                            "callback-access",
                            "ReadOnly, Mutable and Owned describe a Function or a parameter; this position cannot carry data access.",
                            access.span,
                        ));
                    }
                    self.access = Some(access);
                }
            }
            Work::Tuple { start, mut positions, depth } => {
                positions.push(self.values.pop().expect("tuple position"));
                if self.is(",") {
                    self.take();
                    self.tuple_next(start, positions, depth)?;
                } else {
                    self.tuple_finish(start, positions)?;
                }
            }
            Work::Group { start } => {
                let inner = self.values.pop().expect("grouped type");
                let end = self.expect(")")?.span.end;
                self.values.push(ValueAnnotation { kind: ValueAnnotationKind::Group(Box::new(inner)), span: Span::new(start, end) });
            }
            Work::Applied { name, mut arguments, depth } => {
                arguments.push(self.values.pop().expect("type argument"));
                if self.is(",") {
                    self.take();
                    self.request(Work::Applied { name, arguments, depth }, depth + 1);
                } else {
                    let end = self.expect(">")?.span.end;
                    let start = name.span.start;
                    self.values.push(ValueAnnotation { kind: ValueAnnotationKind::Applied { name, arguments }, span: Span::new(start, end) });
                }
            }
            Work::CallableParameter { mut frame, head } => {
                let annotation = self.values.pop().expect("callable parameter type");
                self.access_permitted = false;
                let access = self.access.take();
                let end = self.wrapper_end.take().filter(|_| access.is_some()).map_or(annotation.span.end, |end| end.max(annotation.span.end));
                let span = Span::new(head.name.span.start, end);
                frame.parameters.push(CallableParameter { name: head.name, access, omission: head.omission, rest: head.rest, annotation, span });
                if head.rest && self.is(",") {
                    return Err(Diagnostic::new("rest-parameter", "A rest parameter must be last.", self.peek().span));
                }
                if self.is(",") {
                    self.take();
                    self.callable_parameter(frame)?;
                } else {
                    self.callable_result(frame)?;
                }
            }
            Work::CallableResult { start, parameters } => {
                let result = Box::new(self.values.pop().expect("callable result type"));
                let end = result.span.end;
                self.values.push(ValueAnnotation {
                    kind: ValueAnnotationKind::Callable { ownership: CallableOwnership::Default, parameters, result },
                    span: Span::new(start, end),
                });
            }
            Work::Signature { depth } => self.signature(depth)?,
            Work::FunctionResult { name, depth } => {
                let result = Box::new(self.values.pop().expect("function result type"));
                let frame = FunctionFrame { name, result, parameters: Vec::new(), depth: depth + 1 };
                if self.is(",") {
                    self.take();
                    if !self.is("[") {
                        return Err(self.error("Function arguments are written as a list, such as Function<int, [string, int]>."));
                    }
                    self.take();
                    self.function_argument(frame)?;
                } else {
                    self.function_finish(frame)?;
                }
            }
            Work::FunctionArgument { mut frame, head } => {
                let annotation = self.values.pop().expect("function argument type");
                self.access_permitted = false;
                let access = self.access.take();
                let end = self.wrapper_end.take().filter(|_| access.is_some()).map_or(annotation.span.end, |end| end.max(annotation.span.end));
                if head.rest && head.omission.is_omittable() {
                    return Err(Diagnostic::new("rest-parameter", "A rest parameter cannot be optional.", head.name.span));
                }
                if !head.rest && !head.omission.is_omittable() && frame.parameters.iter().any(|parameter| parameter.omission.is_omittable()) {
                    return Err(Diagnostic::new("parameter-order", "A required parameter cannot follow an optional parameter.", head.name.span));
                }
                let span = Span::new(head.start, end);
                frame.parameters.push(CallableParameter { rest: head.rest, name: head.name, access, omission: head.omission, annotation, span });
                if head.rest && self.is(",") {
                    return Err(Diagnostic::new("rest-parameter", "A rest parameter must be last.", self.peek().span));
                }
                if self.is(",") {
                    self.take();
                    self.function_argument(frame)?;
                } else {
                    self.expect("]")?;
                    self.function_finish(frame)?;
                }
            }
            Work::WrapperSignature { name, depth } => {
                let access = self.signatures.pop().expect("wrapped signature access");
                // A wrapped Function may still join null or undefined.
                if access.is_none() && self.is("|") {
                    let first = self.values.pop().expect("wrapped signature");
                    self.take();
                    self.work.push(Work::WrapperFinish { name, access });
                    self.work.push(Work::UnionNext { start: first.span.start, members: vec![first], depth: depth + 1, validate: false });
                    self.work.push(Work::Member { depth: depth + 1 });
                } else {
                    self.wrapper_finish(name, access)?;
                }
            }
            Work::WrapperFinish { name, access } => self.wrapper_finish(name, access)?,
        }
        Ok(())
    }

    fn tuple_next(&mut self, start: usize, positions: Vec<ValueAnnotation>, depth: usize) -> Result<(), Diagnostic> {
        if self.is("]") {
            return self.tuple_finish(start, positions);
        }
        self.request(Work::Tuple { start, positions, depth }, depth + 1);
        Ok(())
    }

    fn tuple_finish(&mut self, start: usize, positions: Vec<ValueAnnotation>) -> Result<(), Diagnostic> {
        let end = self.expect("]")?.span.end;
        self.values.push(ValueAnnotation { kind: ValueAnnotationKind::Tuple(positions), span: Span::new(start, end) });
        Ok(())
    }

    /// One callable parameter up to its colon; its type is requested next.
    fn callable_parameter(&mut self, frame: CallableFrame) -> Result<(), Diagnostic> {
        let rest = self.is("...");
        if rest {
            self.take();
        }
        if self.peek().kind != TokenKind::Name || matches!(self.peek().text.as_str(), "true" | "false") {
            return Err(self.error("Expected a callable parameter name."));
        }
        let token = self.take();
        let name = TypeName { text: token.text, span: token.span };
        let omission = if self.is("?") {
            self.take();
            crate::ParameterOmission::Optional
        } else {
            crate::ParameterOmission::Required
        };
        self.expect(":")?;
        self.access_permitted = true;
        let depth = frame.depth + 1;
        let head = ParameterHead { start: name.span.start, rest, name, omission };
        self.request(Work::CallableParameter { frame, head }, depth);
        Ok(())
    }

    fn callable_result(&mut self, frame: CallableFrame) -> Result<(), Diagnostic> {
        let mut optional = false;
        for parameter in &frame.parameters {
            if parameter.rest {
                if parameter.omission.is_omittable() {
                    return Err(Diagnostic::new("rest-parameter", "A rest parameter cannot be optional.", parameter.name.span));
                }
                continue;
            }
            if parameter.omission.is_omittable() {
                optional = true;
            } else if optional {
                return Err(Diagnostic::new("parameter-order", "A required parameter cannot follow an optional parameter.", parameter.name.span));
            }
        }
        self.expect(")")?;
        self.expect("=>")?;
        self.request(Work::CallableResult { start: frame.start, parameters: frame.parameters }, frame.depth + 1);
        Ok(())
    }

    /// Parse `Function<R, [A]>` and its wrappers into the existing callable
    /// annotation. A wrapper around data returns the argument access instead;
    /// only a Function argument list can accept that.
    fn signature(&mut self, depth: usize) -> Result<(), Diagnostic> {
        if self.policy.reached(depth) {
            return Err(Self::nesting_limit(self.peek().span));
        }
        let name = self.take();
        self.expect("<")?;
        if name.text == "Function" {
            self.request(Work::FunctionResult { name, depth }, depth + 1);
        } else if self.signature_start() {
            self.work.push(Work::WrapperSignature { name, depth });
            self.work.push(Work::Signature { depth: depth + 1 });
        } else {
            self.request(Work::WrapperFinish { name, access: None }, depth + 1);
        }
        Ok(())
    }

    /// `[string, limit?: int, ...values: int[]]`; labels are optional.
    fn function_argument(&mut self, frame: FunctionFrame) -> Result<(), Diagnostic> {
        if self.is("]") {
            self.take();
            return self.function_finish(frame);
        }
        let start = self.peek().span.start;
        let rest = self.is("...");
        if rest {
            self.take();
        }
        let labelled = self.peek().kind == TokenKind::Name && (self.symbol(1, ":") || (self.symbol(1, "?") && self.symbol(2, ":")));
        let (name, omission) = if labelled {
            let token = self.take();
            let omission = if self.is("?") {
                self.take();
                crate::ParameterOmission::Optional
            } else {
                crate::ParameterOmission::Required
            };
            self.take();
            (TypeName { text: token.text, span: token.span }, omission)
        } else {
            let at = self.peek().span.start;
            (TypeName { text: format!("arg{}", frame.parameters.len()), span: Span::new(at, at) }, crate::ParameterOmission::Required)
        };
        self.access_permitted = true;
        let depth = frame.depth;
        let head = ParameterHead { start, rest, name, omission };
        self.request(Work::FunctionArgument { frame, head }, depth);
        Ok(())
    }

    fn function_finish(&mut self, frame: FunctionFrame) -> Result<(), Diagnostic> {
        if self.is(",") {
            return Err(self.error("Function takes a result type and one optional argument list, such as Function<int, [string]>."));
        }
        let end = self.expect(">")?.span.end;
        let value = ValueAnnotation {
            kind: ValueAnnotationKind::Callable { ownership: CallableOwnership::Default, parameters: frame.parameters, result: frame.result },
            span: Span::new(frame.name.span.start, end),
        };
        self.validate_nesting(&value)?;
        self.values.push(value);
        self.signatures.push(None);
        Ok(())
    }

    fn wrapper_finish(&mut self, name: Token, inner_access: Option<ParameterAccess>) -> Result<(), Diagnostic> {
        let inner = self.values.pop().expect("wrapped type");
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
        if let ValueAnnotationKind::Callable { ownership, .. } = &leaf.kind {
            if *ownership != CallableOwnership::Default {
                return Err(Diagnostic::new("callback-access", "A Function accepts one ownership wrapper: ReadOnly, Mutable, Once or Owned.", name.span));
            }
            let ownership = match name.text.as_str() {
                "ReadOnly" => CallableOwnership::Read,
                "Mutable" => CallableOwnership::Reusable,
                "Once" => CallableOwnership::OwnedOnce,
                _ => CallableOwnership::Owned,
            };
            // The wrapper replaces its groups and the Function inside them.
            let mut kind = inner.into_kind();
            while let ValueAnnotationKind::Group(next) = kind {
                kind = next.into_kind();
            }
            let ValueAnnotationKind::Callable { parameters, result, .. } = kind else {
                unreachable!("wrapped function type")
            };
            self.values.push(ValueAnnotation { kind: ValueAnnotationKind::Callable { ownership, parameters, result }, span });
            self.signatures.push(None);
            return Ok(());
        }
        let mode = match name.text.as_str() {
            "ReadOnly" => Mode::Read,
            "Mutable" => Mode::Mutate,
            "Owned" => Mode::Consume,
            _ => return Err(Diagnostic::new("callback-access", "Once applies to a Function; use Owned<T> to transfer a data argument.", name.span)),
        };
        // The type keeps its own span so diagnostics point at it.
        self.wrapper_end = Some(span.end);
        self.values.push(inner);
        self.signatures.push(Some(ParameterAccess { mode, span: name.span }));
        Ok(())
    }

    fn member(&mut self, depth: usize) -> Result<(), Diagnostic> {
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
        if self.policy.reached(depth) {
            return Err(Self::nesting_limit(self.peek().span));
        }
        let permitted = std::mem::take(&mut self.access_permitted);
        self.work.push(Work::MemberSuffix);
        if self.callable_start() {
            let start = self.peek().span.start;
            self.expect("(")?;
            let frame = CallableFrame { start, parameters: Vec::new(), depth };
            if self.is(")") {
                self.callable_result(frame)?;
            } else {
                self.callable_parameter(frame)?;
            }
        } else if self.is("[") {
            let start = self.take().span.start;
            self.tuple_next(start, Vec::new(), depth)?;
        } else if self.is("(") {
            let start = self.take().span.start;
            self.request(Work::Group { start }, depth + 1);
        } else if self.signature_start() {
            self.work.push(Work::MemberSignature { permitted });
            self.work.push(Work::Signature { depth });
        } else if matches!(self.peek().kind, TokenKind::String | TokenKind::Integer | TokenKind::Number)
            || self.peek().kind == TokenKind::Name && matches!(self.peek().text.as_str(), "true" | "false")
            || self.is("-") && self.tokens.get(self.index + 1).is_some_and(|token| matches!(token.kind, TokenKind::Integer | TokenKind::Number))
        {
            let start = self.peek().span.start;
            let negative = self.is("-");
            if negative {
                self.take();
            }
            let token = self.take();
            let numeric = if negative { format!("-{}", token.text) } else { token.text.clone() };
            let literal = match token.kind {
                TokenKind::String => LiteralAnnotation::String(token.text),
                TokenKind::Integer => LiteralAnnotation::Int(numeric),
                TokenKind::Number => LiteralAnnotation::Number(numeric),
                _ => LiteralAnnotation::Boolean(token.text == "true"),
            };
            self.values.push(ValueAnnotation { kind: ValueAnnotationKind::Literal(literal), span: Span::new(start, token.span.end) });
        } else {
            if self.peek().kind != TokenKind::Name || matches!(self.peek().text.as_str(), "true" | "false") {
                return Err(self.error("Expected a type name."));
            }
            let token = self.take();
            let name = TypeName { text: token.text, span: token.span };
            if self.is("<") && !matches!(name.text.as_str(), "int" | "number" | "string" | "boolean" | "void" | "null" | "undefined") {
                self.take();
                self.request(Work::Applied { name, arguments: Vec::new(), depth }, depth + 1);
            } else {
                self.values.push(ValueAnnotation { span: name.span, kind: ValueAnnotationKind::Named(name) });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod formatting_probe_tests {
    use super::*;

    #[test]
    fn formatting_probe_restores_successful_and_failed_packed_closer_edits() {
        for source in [
            "Box<Box<int>>", "Box<Box<int>>=value", "Box<int>=>value",
            "Function<Box<int>,[value:Box<int>]>",
            "Function<Box<int>,[value:Box<int>]>=>value",
            "Function<Box<int>,[value:Box<int>]>>(",
            "Box<Box<int>> |", "Box<Box<int>>[", "Box<Box<int>>, next",
        ] {
            let original = lex(source).unwrap();
            let identity = |tokens: &[Token]| tokens.iter().map(|token| (token.kind, token.text.clone(), token.span)).collect::<Vec<_>>();
            let mut fresh = original.clone();
            let expected = parse_parameter_prefix(&mut fresh, 0, 0, NestingPolicy::CONSUMERS).map(|(_, annotation, end)| {
                (Span::new(original[0].span.start, fresh[end - 1].span.end.max(annotation.span.end)),
                 fresh.get(end).is_some_and(|next| next.text == "("))
            }).map_err(|error| (error.code, error.message, error.span));
            let mut scratch = original.clone();
            for _ in 0..2 {
                let actual = parameter_prefix_span(&mut scratch, 0).map_err(|error| (error.code, error.message, error.span));
                assert_eq!(actual, expected, "{source}");
                assert_eq!(identity(&scratch), identity(&original), "{source}");
            }
        }
    }
}
