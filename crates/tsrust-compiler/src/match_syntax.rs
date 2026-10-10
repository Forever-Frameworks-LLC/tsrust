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
    pub(super) fn match_statement_header(&self) -> bool {
        if !self.is("match") || !self.tokens.get(self.i + 1).is_some_and(|token| token.text == "(") { return false; }
        let mut depth = 0usize;
        for (offset, token) in self.tokens[self.i + 1..].iter().enumerate() {
            if token.kind != TokenKind::Symbol { continue; }
            match token.text.as_str() {
                "(" => depth += 1,
                ")" => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 { return self.tokens.get(self.i + offset + 2).is_some_and(|token| token.text == "{"); }
                }
                _ => {},
            }
        }
        false
    }

    #[inline(never)]
    pub(super) fn lower_match_statement(&mut self) -> Result<Stmt, Diagnostic> {
        let holder = self.pattern_holder();
        let ParsedMatch { scrutinee, arms, span } = self.match_statement_syntax()?;
        let statement = |kind, span| Stmt { labels: Vec::new(), control_target: None, kind, span };
        let intrinsic = |name: &str, tag: Option<(&str, Span)>, span: Span| {
            let mut arguments = vec![Expr::new(ExprKind::Name(holder.text.clone()), span)];
            if let Some((tag, at)) = tag { arguments.push(Expr::new(ExprKind::String(tag.into()), at)); }
            Expr::new(ExprKind::Call(Box::new(Expr::new(ExprKind::Name(name.into()), span)), arguments), span)
        };
        let mut coverage = intrinsic("@match:coverage", None, span);
        if let ExprKind::Call(_, arguments) = &mut coverage.kind {
            for arm in &arms {
                let (name, at) = match &arm.pattern {
                    ParsedPattern::Wildcard(at) => ("_", *at),
                    ParsedPattern::Variant { name, .. } => (name.text.as_str(), name.span),
                };
                arguments.push(Expr::new(ExprKind::String(name.into()), at));
            }
        }
        let mut tail = Vec::new();
        for arm in arms.into_iter().rev() {
            let mut body = arm.body;
            let tag = match arm.pattern {
                ParsedPattern::Wildcard(_) => None,
                ParsedPattern::Variant { name, binding } => {
                    if let Some(binding) = binding {
                        let value = intrinsic("@match:payload", Some((&name.text, name.span)), binding.span);
                        let mut temporary = self.pattern_holder();
                        temporary.span = Span::new(binding.span.start, binding.span.start);
                        let field = Expr::new(ExprKind::Field(Box::new(Expr::new(ExprKind::Name(temporary.text.clone()), binding.span)), "payload".into()), binding.span);
                        body.insert(0, statement(StmtKind::Let { rebindable: false, name: binding, annotation: None, value: field, binding: None }, arm.span));
                        body.insert(0, statement(StmtKind::Let { rebindable: false, name: temporary, annotation: None, value, binding: None }, arm.span));
                    }
                    Some(name)
                }
            };
            // Coverage checking precedes the chain and proves the final arm is
            // the remainder. This lets existing return/loan joins remain exact.
            if tail.is_empty() { tail.push(statement(StmtKind::Block(body), arm.span)); }
            else {
                let condition = tag.map(|name| intrinsic("@match:variant", Some((&name.text, name.span)), arm.span)).unwrap_or_else(|| Expr::new(ExprKind::Boolean(true), arm.span));
                tail = vec![statement(StmtKind::If { condition, then_body: body, else_body: tail }, arm.span)];
            }
        }
        let mut body = vec![statement(StmtKind::Let { rebindable: false, name: holder, annotation: None, value: scrutinee, binding: None }, span), statement(StmtKind::Expr(coverage), span)];
        body.extend(tail);
        Ok(statement(StmtKind::Block(body), span))
    }

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
            if self.depth + arms.len() >= 128 { return Err(Diagnostic::new("unsupported", "Match arm nesting exceeds the prototype limit of 128.", self.peek().span)); }
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
