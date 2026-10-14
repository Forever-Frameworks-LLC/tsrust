//! Span-preserving match syntax inspection. Parsing does not establish coverage,
//! payload types, ownership, or executable Result support.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pattern {
    Literal(Span),
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
pub(super) struct ParsedMatch {
    pub(super) scrutinee: Expr,
    pub(super) arms: Vec<ParsedArm>,
    pub(super) span: Span,
}

pub(super) struct ParsedArm {
    pub(super) pattern: ParsedPattern,
    pub(super) body: Vec<Stmt>,
    pub(super) value: Option<Expr>,
    pub(super) body_span: Span,
    pub(super) span: Span,
}

pub(super) enum ParsedPattern {
    Literal(Expr),
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
        let ParsedArm { pattern, body, body_span, span, .. } = arm;
        // Inspection validates the body with the ordinary parser but does not
        // expose an unchecked compiler AST through the public interface.
        drop(body);
        let pattern = match pattern {
            ParsedPattern::Literal(value) => Pattern::Literal(value.span),
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

    /// Lower one parsed match statement. `holder` was allocated before its
    /// scrutinee and arms were parsed.
    pub(super) fn lower_match_statement(&mut self, holder: Token, parsed: ParsedMatch) -> Stmt {
        let ParsedMatch { scrutinee, arms, span } = parsed;
        let statement = |kind, span| Stmt { labels: Vec::new(), control_target: None, kind, span };
        let intrinsic = |name: &str, tag: Option<(&str, Span)>, span: Span| {
            let mut arguments = vec![Expr::new(ExprKind::Name(holder.text.clone()), span)];
            if let Some((tag, at)) = tag { arguments.push(Expr::new(ExprKind::String(tag.into()), at)); }
            Expr::new(ExprKind::Call(Box::new(Expr::new(ExprKind::Name(name.into()), span)), arguments), span)
        };
        let mut coverage = intrinsic("@match:coverage", None, span);
        if let ExprKind::Call(_, arguments) = &mut coverage.kind {
            for arm in &arms {
                if let ParsedPattern::Literal(value) = &arm.pattern {
                    arguments.push(Expr::new(ExprKind::String(literal_label(value)), value.span));
                    continue;
                }
                let (name, at) = match &arm.pattern {
                    ParsedPattern::Literal(_) => unreachable!(),
                    ParsedPattern::Wildcard(at) => ("_", *at),
                    ParsedPattern::Variant { name, .. } => (name.text.as_str(), name.span),
                };
                arguments.push(Expr::new(ExprKind::String(name.into()), at));
            }
        }
        let mut tail = Vec::new();
        for arm in arms.into_iter().rev() {
            let mut body = arm.body;
            if let Some(value) = arm.value { body.push(statement(StmtKind::Expr(value), arm.span)); }
            let tag = match arm.pattern {
                ParsedPattern::Wildcard(_) => None,
                ParsedPattern::Literal(value) => Some(Expr::new(ExprKind::Binary { op: BinaryOp::Equal, operator: value.span, left: Box::new(Expr::new(ExprKind::Name(holder.text.clone()), span)), right: Box::new(value) }, arm.span)),
                ParsedPattern::Variant { name, binding } => {
                    // Retain the selected variant so payload loans keep their original ownership mode.
                    let at = binding.as_ref().map_or(name.span, |binding| binding.span);
                    let value = intrinsic("@match:payload", Some((&name.text, name.span)), at);
                    let mut temporary = self.pattern_holder();
                    temporary.span = Span::new(at.start, at.start);
                    if let Some(binding) = binding {
                        let selected = Expr::new(ExprKind::Name(temporary.text.clone()), binding.span);
                        let callee = Expr::new(ExprKind::Name("@match:binding".into()), binding.span);
                        let value = Expr::new(ExprKind::Call(Box::new(callee), vec![selected]), binding.span);
                        body.insert(0, statement(StmtKind::Let { rebindable: false, name: binding, annotation: None, value, binding: None }, arm.span));
                    }
                    body.insert(0, statement(StmtKind::Let { rebindable: false, name: temporary, annotation: None, value, binding: None }, arm.span));
                    Some(intrinsic("@match:variant", Some((&name.text, name.span)), arm.span))
                }
            };
            // Coverage checking precedes the chain and proves the final arm is
            // the remainder. This lets existing return/loan joins remain exact.
            if tail.is_empty() { tail.push(statement(StmtKind::Block(body), arm.span)); }
            else {
                let condition = tag.unwrap_or_else(|| Expr::new(ExprKind::Boolean(true), arm.span));
                tail = vec![statement(StmtKind::If { condition, then_body: body, else_body: tail }, arm.span)];
            }
        }
        let mut body = vec![statement(StmtKind::Let { rebindable: false, name: holder, annotation: None, value: scrutinee, binding: None }, span), statement(StmtKind::Expr(coverage), span)];
        body.extend(tail);
        statement(StmtKind::Block(body), span)
    }

    pub(super) fn lower_match_value(&mut self, holder: Token, mut parsed: ParsedMatch) -> Result<Expr, Diagnostic> {
        let id = parsed.span.start;
        for arm in &mut parsed.arms {
            if arm.value.is_none() && !crate::completion::definitely_abrupt(&arm.body) {
                return Err(Diagnostic::new("match-value", "A value arm uses => expression; a block arm must exit with return, throw, break, or continue.", arm.body_span));
            }
            if let Some(value) = arm.value.take() {
                let span = value.span;
                let callee = Expr::new(ExprKind::Name("@match:yield".into()), span);
                arm.value = Some(Expr::new(ExprKind::Call(Box::new(callee), vec![value]), span));
            }
        }
        let span = parsed.span;
        let lowered = self.lower_match_statement(holder, parsed);
        let StmtKind::Block(body) = lowered.into_kind() else { unreachable!() };
        Ok(Expr::new(ExprKind::Match { id, body }, span))
    }

    pub(super) fn match_pattern(&mut self) -> Result<ParsedPattern, Diagnostic> {
        if matches!(self.peek().kind, TokenKind::String | TokenKind::Integer | TokenKind::Number)
            || matches!(self.peek().text.as_str(), "true" | "false" | "null" | "undefined" | "-") {
            let negative_number = self.is("-") && self.tokens.get(self.i + 1).is_some_and(|token| token.kind == TokenKind::Number);
            let value = if negative_number {
                let start = self.take().span.start;
                let mut value = self.atom()?;
                if let ExprKind::Number(number) = &mut value.kind { *number = -*number; }
                value.span.start = start;
                value
            } else if self.is("undefined") {
                let token = self.take(); Expr::new(ExprKind::Undefined, token.span)
            } else { self.atom()? };
            return Ok(ParsedPattern::Literal(value));
        }
        let name = self.name()?;
        if name.text == "_" { return Ok(ParsedPattern::Wildcard(name.span)); }
        self.expect("(")?;
        let binding = self.name()?;
        self.expect(")")?;
        Ok(ParsedPattern::Variant { name, binding: (binding.text != "_").then_some(binding) })
    }
}

fn literal_label(value: &Expr) -> String {
    match &value.kind {
        ExprKind::String(value) => format!("@string:{value}"),
        ExprKind::Integer(value) => format!("@int:{value}"),
        ExprKind::Number(value) => format!("@number:{value}"),
        ExprKind::Boolean(value) => format!("@boolean:{value}"),
        ExprKind::Null => "@null:".into(),
        ExprKind::Undefined => "@undefined:".into(),
        _ => unreachable!("literal pattern"),
    }
}
