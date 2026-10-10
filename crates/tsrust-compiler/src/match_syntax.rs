//! Span-preserving match syntax inspection. Parsing does not establish coverage,
//! payload types, ownership, or executable Result support.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pattern {
    Wildcard(Span),
    Variant { name: Span, binding: Option<Span> },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Arm {
    pub pattern: Pattern,
    pub body: Span,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchSyntax {
    pub scrutinee: Span,
    pub arms: Vec<Arm>,
    pub span: Span,
}

// Retain ordinary expression and statement trees for the checking integration;
// the public inspection view exposes only original-source coordinates.
struct ParsedMatch {
    scrutinee: Expr,
    arms: Vec<ParsedArm>,
    span: Span,
}

struct ParsedArm {
    pattern: ParsedPattern,
    body: Vec<Stmt>,
    body_span: Span,
    span: Span,
}

enum ParsedPattern {
    Wildcard(Span),
    Variant { name: Token, binding: Option<Token> },
}

/// Inspect one statement using the compiler's ordinary expression and block
/// parsers. Source offsets include any leading comments and whitespace.
pub fn parse(source: &str) -> Result<MatchSyntax, Diagnostic> {
    let mut parser = Parser::new(source, Default::default(), Default::default())?;
    let value = parser.match_statement_syntax()?;
    if parser.peek().kind != TokenKind::End {
        return Err(Diagnostic::new("syntax", "Unexpected token after match statement.", parser.peek().span));
    }
    let ParsedMatch { scrutinee, arms, span } = value;
    let arms = arms.into_iter().map(|arm| {
        let ParsedArm { pattern, body, body_span, span } = arm;
        // Inspection validates the body with the ordinary parser but does not
        // expose an unchecked compiler AST through the public interface.
        drop(body);
        let pattern = match pattern {
            ParsedPattern::Wildcard(span) => Pattern::Wildcard(span),
            ParsedPattern::Variant { name, binding } => Pattern::Variant { name: name.span, binding: binding.map(|value| value.span) },
        };
        Arm { pattern, body: body_span, span }
    }).collect();
    Ok(MatchSyntax { scrutinee: scrutinee.span, arms, span })
}

impl Parser<'_> {
    #[inline(never)]
    fn match_statement_syntax(&mut self) -> Result<ParsedMatch, Diagnostic> {
        let start = self.expect("match")?.span.start;
        self.expect("(")?;
        let scrutinee = self.comma_expression()?;
        self.expect(")")?;
        self.expect("{")?;
        let mut arms = Vec::new();
        while !self.accept("}") {
            if self.peek().kind == TokenKind::End { self.expect("}")?; }
            let arm_start = self.peek().span.start;
            let pattern = self.match_pattern()?;
            self.expect("=>")?;
            let body_start = self.peek().span.start;
            let body = self.block()?;
            let end = self.tokens[self.i - 1].span.end;
            arms.push(ParsedArm { pattern, body, body_span: Span::new(body_start, end), span: Span::new(arm_start, end) });
            // Rust-style block arms may omit their separator; a trailing comma
            // is accepted. A doubled comma remains a malformed next pattern.
            self.accept(",");
        }
        Ok(ParsedMatch { scrutinee, arms, span: Span::new(start, self.tokens[self.i - 1].span.end) })
    }

    #[inline(never)]
    fn match_pattern(&mut self) -> Result<ParsedPattern, Diagnostic> {
        let name = self.name()?;
        if name.text == "_" { return Ok(ParsedPattern::Wildcard(name.span)); }
        self.expect("(")?;
        let binding = self.name()?;
        self.expect(")")?;
        Ok(ParsedPattern::Variant { name, binding: (binding.text != "_").then_some(binding) })
    }
}
