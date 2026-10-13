//! Expression grammar continuations. Token position and grammar scopes stay in Parser.
use super::*;

struct BinaryFrame {
    minimum: u8,
    left_start: usize,
    unary_base: bool,
}

struct CallFrame {
    callee: Expr,
    arguments: Vec<Expr>,
    types: Option<Vec<crate::type_syntax::ValueAnnotation>>,
    optional: bool,
}

struct ClosureFrame {
    id: usize,
    start: Span,
    header: Box<(Vec<CallableParameter>, ResultAnnotation)>,
    patterns: Vec<Stmt>,
}

struct Recovery {
    start: usize,
    depth: usize,
    pending: usize,
    arguments: usize,
    patterns: usize,
    expressions: usize,
    statements: usize,
    bodies: usize,
}

enum StatementExpression {
    Return,
    Throw,
    Discard,
}

enum Work {
    Block,
    BlockNext(Vec<Stmt>),
    BlockAfter {
        body: Vec<Stmt>,
        recovery: Recovery,
    },
    Statement,
    StatementBlock(Span),
    StatementExpression {
        span: Span,
        kind: StatementExpression,
    },
    ClosureBlock(ClosureFrame),
    ClosureValue {
        frame: ClosureFrame,
        outer: Vec<Stmt>,
    },
    Parse(u8),
    LeaveDepth,
    Primary,
    Array {
        start: usize,
        items: Vec<Expr>,
        spread: Option<Span>,
    },
    RecordNext {
        start: usize,
        fields: Vec<(Token, Expr)>,
    },
    RecordField {
        start: usize,
        fields: Vec<(Token, Expr)>,
        key: Token,
    },
    Template {
        start: usize,
        parts: Vec<Expr>,
    },
    Unary(Vec<Token>),
    Binary(BinaryFrame),
    Right {
        frame: BinaryFrame,
        left: Expr,
        op: BinaryOp,
        operator: Span,
        assignment: bool,
        right_start: usize,
    },
    In {
        frame: BinaryFrame,
        left: Expr,
    },
    Consequent(Expr),
    Alternate {
        predicate: Expr,
        consequent: Expr,
    },
    Comma(Vec<Expr>),
    Group,
    Postfix {
        optional: bool,
    },
    Index {
        base: Expr,
        optional: bool,
    },
    Argument {
        call: CallFrame,
        spread: Option<Span>,
    },
}

impl Parser<'_> {
    pub(super) fn expression_work(&mut self, minimum: u8) -> Result<Expr, Diagnostic> {
        let depth = self.depth;
        let result = self.run_expression_work(minimum);
        // Recovery owns token synchronization. No abandoned expression keeps an
        // active grammar-depth contribution after its work stack is dropped.
        self.depth = depth;
        result
    }

    fn run_expression_work(&mut self, minimum: u8) -> Result<Expr, Diagnostic> {
        let mut work = vec![Work::Parse(minimum)];
        let mut values = Vec::new();
        let mut statements = Vec::new();
        let mut bodies = Vec::new();
        while let Some(task) = work.pop() {
            let step = (|| -> Result<(), Diagnostic> {
                match task {
                    Work::ClosureBlock(frame) => {
                        let mut body = bodies.pop().expect("closure body");
                        Self::open_with_patterns(frame.patterns, &mut body);
                        let span = Span::new(frame.start.start, self.tokens[self.i - 1].span.end);
                        values.push(Self::closure_expression(frame.id, frame.header, body, span));
                        work.push(Work::Postfix { optional: false });
                    }
                    Work::ClosureValue { frame, outer } => {
                        let value = values.pop().expect("closure expression body");
                        let mut body = std::mem::replace(&mut self.pending, outer);
                        body.append(&mut Self::returned(Box::new(value)));
                        Self::open_with_patterns(frame.patterns, &mut body);
                        let span = Span::new(frame.start.start, self.tokens[self.i - 1].span.end);
                        values.push(Self::closure_expression(frame.id, frame.header, body, span));
                        work.push(Work::Postfix { optional: false });
                    }
                    Work::Block => {
                        if self.depth >= 128 {
                            return Err(Diagnostic::new(
                                "unsupported",
                                "Block nesting exceeds the prototype limit of 128.",
                                self.peek().span,
                            ));
                        }
                        self.expect("{")?;
                        self.depth += 1;
                        work.push(Work::BlockNext(Vec::new()));
                    }
                    Work::BlockNext(body) => {
                        if self.accept("}") {
                            self.depth -= 1;
                            bodies.push(body);
                        } else {
                            if self.peek().kind == TokenKind::End {
                                self.expect("}")?;
                            }
                            let recovery = Recovery {
                                start: self.i,
                                depth: self.depth,
                                pending: self.pending.len(),
                                arguments: self.call_type_arguments.len(),
                                patterns: self.parameter_patterns.len(),
                                expressions: values.len(),
                                statements: statements.len(),
                                bodies: bodies.len(),
                            };
                            work.push(Work::BlockAfter { body, recovery });
                            work.push(Work::Statement);
                        }
                    }
                    Work::BlockAfter { mut body, .. } => {
                        body.push(statements.pop().expect("block statement"));
                        body.append(&mut self.pending);
                        work.push(Work::BlockNext(body));
                    }
                    Work::Statement => {
                        let span = self.peek().span;
                        if self.is("{") {
                            work.push(Work::StatementBlock(span));
                            work.push(Work::Block);
                        } else if self.accept("return") {
                            if self.accept(";") {
                                statements.push(Stmt {
                                    labels: Vec::new(),
                                    control_target: None,
                                    kind: StmtKind::Return(None),
                                    span,
                                });
                            } else {
                                work.push(Work::StatementExpression {
                                    span,
                                    kind: StatementExpression::Return,
                                });
                                work.push(Work::Comma(Vec::new()));
                                work.push(Work::Parse(0));
                            }
                        } else if self.accept("throw") {
                            if self.source[span.end..self.peek().span.start].contains(['\n', '\r'])
                            {
                                return Err(Diagnostic::new(
                                    "syntax",
                                    "A thrown value must begin on the same line as throw.",
                                    span,
                                ));
                            }
                            work.push(Work::StatementExpression {
                                span,
                                kind: StatementExpression::Throw,
                            });
                            work.push(Work::Comma(Vec::new()));
                            work.push(Work::Parse(0));
                        } else if matches!(
                            self.peek().text.as_str(),
                            "function"
                                | "try"
                                | "for"
                                | "if"
                                | "while"
                                | "do"
                                | "switch"
                                | "const"
                                | "let"
                                | "break"
                                | "continue"
                        ) || self.match_statement_header()
                            || (self.peek().kind == TokenKind::Name
                                && self
                                    .tokens
                                    .get(self.i + 1)
                                    .is_some_and(|token| token.text == ":"))
                        {
                            statements.push(self.block_statement()?);
                        } else {
                            work.push(Work::StatementExpression {
                                span,
                                kind: StatementExpression::Discard,
                            });
                            work.push(Work::Comma(Vec::new()));
                            work.push(Work::Parse(0));
                        }
                    }
                    Work::StatementBlock(span) => {
                        statements.push(Stmt {
                            labels: Vec::new(),
                            control_target: None,
                            kind: StmtKind::Block(bodies.pop().expect("nested block")),
                            span,
                        });
                    }
                    Work::StatementExpression { span, kind } => {
                        let expression = values.pop().expect("statement expression");
                        let kind = match kind {
                            StatementExpression::Return => StmtKind::Return(Some(expression)),
                            StatementExpression::Throw => StmtKind::Throw(expression),
                            StatementExpression::Discard => {
                                if matches!(&expression.kind, ExprKind::Assign { target, .. } if matches!(target.kind, ExprKind::Array(_) | ExprKind::Record(_)))
                                {
                                    self.destructuring_assignment(expression)?
                                } else {
                                    Self::expression_statement(expression)
                                }
                            }
                        };
                        self.expect(";")?;
                        statements.push(Stmt {
                            labels: Vec::new(),
                            control_target: None,
                            kind,
                            span,
                        });
                    }
                    Work::Parse(minimum) => {
                        // Retained during migration: legacy primary/body/type paths
                        // still re-enter this driver. This is not a resource policy.
                        if self.depth >= 128 {
                            return Err(Diagnostic::new(
                                "unsupported",
                                "Expression nesting exceeds the prototype limit of 128.",
                                self.peek().span,
                            ));
                        }
                        self.depth += 1;
                        let frame = BinaryFrame {
                            minimum,
                            left_start: self.i,
                            unary_base: matches!(self.peek().text.as_str(), "-" | "+" | "!" | "~"),
                        };
                        let operators = self.unary_operators()?;
                        work.push(Work::LeaveDepth);
                        work.push(Work::Binary(frame));
                        work.push(Work::Unary(operators));
                        work.push(Work::Primary);
                    }
                    Work::LeaveDepth => self.depth -= 1,
                    Work::Unary(operators) => {
                        let value = values.pop().expect("primary expression");
                        values.push(Self::apply_unary_operators(value, operators)?);
                    }
                    Work::Primary => {
                        self.typed_arrow_without_parentheses()?;
                        if self.is("(") && !self.closure_header() {
                            self.take();
                            work.push(Work::Postfix { optional: false });
                            work.push(Work::Group);
                            work.push(Work::Comma(Vec::new()));
                            work.push(Work::Parse(0));
                        } else if self.peek().kind == TokenKind::TemplateHead {
                            let head = self.take();
                            let start = head.span.start;
                            let parts = vec![Expr::new(ExprKind::String(head.text), head.span)];
                            work.push(Work::Template { start, parts });
                            work.push(Work::Comma(Vec::new()));
                            work.push(Work::Parse(0));
                        } else if self.is("[") {
                            let start = self.take().span.start;
                            if self.is("]") {
                                let end = self.take().span.end;
                                values.push(Expr::new(
                                    ExprKind::Array(Vec::new()),
                                    Span::new(start, end),
                                ));
                                work.push(Work::Postfix { optional: false });
                            } else {
                                let spread =
                                    self.accept("...").then(|| self.tokens[self.i - 1].span);
                                work.push(Work::Array {
                                    start,
                                    items: Vec::new(),
                                    spread,
                                });
                                work.push(Work::Parse(0));
                            }
                        } else if self.is("{") {
                            let start = self.take().span.start;
                            work.push(Work::RecordNext {
                                start,
                                fields: Vec::new(),
                            });
                        } else if self.is("function") {
                            values.push(self.function_primary()?);
                        } else if self.is("(")
                            || (self.peek().kind == TokenKind::Name
                                && self
                                    .tokens
                                    .get(self.i + 1)
                                    .is_some_and(|token| token.text == "=>"))
                        {
                            let outer_patterns = std::mem::take(&mut self.parameter_patterns);
                            let start = self.peek().span;
                            let header = self.closure_signature(start)?;
                            let id = self.next_closure;
                            self.next_closure += 1;
                            let patterns =
                                std::mem::replace(&mut self.parameter_patterns, outer_patterns);
                            let frame = ClosureFrame {
                                id,
                                start,
                                header,
                                patterns,
                            };
                            if self.is("{") {
                                work.push(Work::ClosureBlock(frame));
                                work.push(Work::Block);
                            } else {
                                let outer = std::mem::take(&mut self.pending);
                                work.push(Work::ClosureValue { frame, outer });
                                work.push(Work::Parse(0));
                            }
                        } else {
                            values.push(self.atom()?);
                            work.push(Work::Postfix { optional: false });
                        }
                    }
                    Work::Array {
                        start,
                        mut items,
                        spread,
                    } => {
                        let mut value = values.pop().expect("array element");
                        if let Some(operator) = spread {
                            let span = Span::new(operator.start, value.span.end);
                            value = Expr::new(
                                ExprKind::Unary {
                                    op: UnaryOp::Spread,
                                    operator,
                                    value: Box::new(value),
                                },
                                span,
                            );
                        }
                        items.push(value);
                        if self.accept(",") && !self.is("]") {
                            let spread = self.accept("...").then(|| self.tokens[self.i - 1].span);
                            work.push(Work::Array {
                                start,
                                items,
                                spread,
                            });
                            work.push(Work::Parse(0));
                        } else {
                            let end = self.expect("]")?.span.end;
                            values.push(Expr::new(ExprKind::Array(items), Span::new(start, end)));
                            work.push(Work::Postfix { optional: false });
                        }
                    }
                    Work::RecordNext { start, mut fields } => loop {
                        if self.is("}") {
                            let end = self.take().span.end;
                            values.push(Expr::new(ExprKind::Record(fields), Span::new(start, end)));
                            work.push(Work::Postfix { optional: false });
                            break;
                        }
                        let spread = self.is("...");
                        let key = if spread {
                            let mut key = self.take();
                            key.text = "record#spread".into();
                            key
                        } else {
                            self.name()?
                        };
                        if !spread && (self.is(",") || self.is("}")) {
                            let value = Expr::new(ExprKind::Name(key.text.clone()), key.span);
                            fields.push((key, value));
                            if !self.accept(",") {
                                self.expect_record_close_work(
                                    start,
                                    fields,
                                    &mut work,
                                    &mut values,
                                )?;
                                break;
                            }
                        } else {
                            if !spread {
                                self.expect(":")?;
                            }
                            work.push(Work::RecordField { start, fields, key });
                            work.push(Work::Parse(0));
                            break;
                        }
                    },
                    Work::RecordField {
                        start,
                        mut fields,
                        key,
                    } => {
                        fields.push((key, values.pop().expect("record field")));
                        if self.accept(",") {
                            work.push(Work::RecordNext { start, fields });
                        } else {
                            self.expect_record_close_work(start, fields, &mut work, &mut values)?;
                        }
                    }
                    Work::Template { start, mut parts } => {
                        parts.push(values.pop().expect("template interpolation"));
                        let chunk = self.take();
                        if !matches!(
                            chunk.kind,
                            TokenKind::TemplateMiddle | TokenKind::TemplateTail
                        ) {
                            return Err(Diagnostic::new(
                                "syntax",
                                "Expected a template interpolation boundary.",
                                chunk.span,
                            ));
                        }
                        let tail = chunk.kind == TokenKind::TemplateTail;
                        let end = chunk.span.end;
                        parts.push(Expr::new(ExprKind::String(chunk.text), chunk.span));
                        if tail {
                            let value = Expr::new(ExprKind::Template(parts), Span::new(start, end));
                            Self::postfix_depth(&value)?;
                            values.push(value);
                            work.push(Work::Postfix { optional: false });
                        } else {
                            work.push(Work::Template { start, parts });
                            work.push(Work::Comma(Vec::new()));
                            work.push(Work::Parse(0));
                        }
                    }
                    Work::Group => {
                        self.expect(")")?;
                    }
                    Work::Comma(mut operands) => {
                        operands.push(values.pop().expect("comma operand"));
                        if self.accept(",") {
                            if matches!(self.peek().text.as_str(), "let" | "const" | "var") {
                                return Err(Diagnostic::new(
                                    "syntax",
                                    "A declaration cannot be a comma operand.",
                                    self.peek().span,
                                ));
                            }
                            work.push(Work::Comma(operands));
                            work.push(Work::Parse(0));
                        } else if operands.len() == 1 {
                            values.push(operands.pop().unwrap());
                        } else {
                            values.push(Self::finish_sequence(operands)?);
                        }
                    }
                    Work::Binary(mut frame) => {
                        let mut left = values.pop().expect("left operand");
                        loop {
                            if frame.minimum <= 8 && (self.is("as") || self.is("satisfies")) {
                                left = self.type_assertion_tail(left)?;
                                frame.unary_base = false;
                                continue;
                            }
                            if frame.minimum == 0 && self.accept("?") {
                                work.push(Work::Consequent(left));
                                work.push(Work::Parse(0));
                                break;
                            }
                            if self.is("in") && frame.minimum <= 8 {
                                self.take();
                                work.push(Work::In { frame, left });
                                work.push(Work::Parse(9));
                                break;
                            }
                            let operator = (self.peek().kind == TokenKind::Symbol)
                                .then(|| match self.peek().text.as_str() {
                                    "=" => Some((BinaryOp::Equal, 0)),
                                    "&&=" => Some((BinaryOp::And, 0)),
                                    "||=" => Some((BinaryOp::Or, 0)),
                                    _ => BinaryOp::assignment(&self.peek().text)
                                        .map(|op| (op, 0))
                                        .or_else(|| BinaryOp::parse(&self.peek().text)),
                                })
                                .flatten();
                            let Some((op, precedence)) =
                                operator.filter(|(_, precedence)| *precedence >= frame.minimum)
                            else {
                                values.push(left);
                                break;
                            };
                            let assignment = precedence == 0;
                            let operator = self.take().span;
                            if op == BinaryOp::Exponent && frame.unary_base {
                                return Err(Diagnostic::new(
                                    "syntax",
                                    "Parenthesize a unary expression used as an exponentiation base.",
                                    operator,
                                ));
                            }
                            let right_start = self.i;
                            work.push(Work::Right {
                                frame,
                                left,
                                op,
                                operator,
                                assignment,
                                right_start,
                            });
                            work.push(Work::Parse(if assignment || op == BinaryOp::Exponent {
                                precedence
                            } else {
                                precedence + 1
                            }));
                            break;
                        }
                    }
                    Work::Right {
                        mut frame,
                        left,
                        op,
                        operator,
                        assignment,
                        right_start,
                    } => {
                        let right = values.pop().expect("right operand");
                        if !assignment {
                            self.nullish_mixing(
                                &left,
                                &right,
                                op,
                                operator,
                                frame.left_start,
                                right_start,
                            )?;
                        }
                        let mut value =
                            Self::binary_expression(left, right, op, operator, assignment);
                        if op == BinaryOp::Coalesce {
                            value.span = Span::new(
                                self.tokens[frame.left_start].span.start,
                                self.tokens[self.i - 1].span.end,
                            );
                        }
                        Self::postfix_depth(&value)?;
                        frame.unary_base = false;
                        values.push(value);
                        work.push(Work::Binary(frame));
                    }
                    Work::In { mut frame, left } => {
                        let right = values.pop().expect("in operand");
                        let span = Span::new(
                            self.tokens[frame.left_start].span.start,
                            self.tokens[self.i - 1].span.end,
                        );
                        let value = Self::object_key_call("hasOwn#in", vec![right, left], span);
                        Self::postfix_depth(&value)?;
                        frame.unary_base = false;
                        values.push(value);
                        work.push(Work::Binary(frame));
                    }
                    Work::Consequent(predicate) => {
                        let consequent = values.pop().expect("consequent");
                        self.expect(":")?;
                        work.push(Work::Alternate {
                            predicate,
                            consequent,
                        });
                        work.push(Work::Parse(0));
                    }
                    Work::Alternate {
                        predicate,
                        consequent,
                    } => {
                        let alternate = values.pop().expect("alternate");
                        let value = Self::conditional_expression(predicate, consequent, alternate);
                        Self::postfix_depth(&value)?;
                        values.push(value);
                    }
                    Work::Postfix { mut optional } => {
                        let mut value = values.pop().expect("postfix operand");
                        Self::postfix_depth(&value)?;
                        if self.is_tagged_template() {
                            return self.reject_tagged_template().map(|_| ());
                        }
                        if self.accept("?.") {
                            optional = true;
                            let operator = self.tokens[self.i - 1].span;
                            let span = value.span;
                            value = Expr::new(
                                ExprKind::Unary {
                                    op: UnaryOp::OptionalBoundary,
                                    operator,
                                    value: Box::new(value),
                                },
                                span,
                            );
                            if self.accept("[") {
                                work.push(Work::Index {
                                    base: value,
                                    optional,
                                });
                                work.push(Work::Comma(Vec::new()));
                                work.push(Work::Parse(0));
                            } else if self.is("(") || self.has_explicit_call_arguments() {
                                self.schedule_call_work(value, optional, &mut work, &mut values)?;
                            } else {
                                values.push(self.field_expression(value)?);
                                work.push(Work::Postfix { optional });
                            }
                        } else if self.accept("!") {
                            values.push(self.non_null_postfix(value, optional));
                            work.push(Work::Postfix { optional: false });
                        } else if self.accept(".") {
                            values.push(self.field_expression(value)?);
                            work.push(Work::Postfix { optional });
                        } else if self.accept("[") {
                            work.push(Work::Index {
                                base: value,
                                optional,
                            });
                            work.push(Work::Comma(Vec::new()));
                            work.push(Work::Parse(0));
                        } else if self.is("(")
                            || (matches!(
                                value.kind,
                                ExprKind::Name(_)
                                    | ExprKind::Field(_, _)
                                    | ExprKind::Sequence(_)
                                    | ExprKind::Assign { .. }
                                    | ExprKind::Conditional { .. }
                                    | ExprKind::Call(..)
                                    | ExprKind::Index(..)
                                    | ExprKind::Unary {
                                        op: UnaryOp::OptionalBoundary
                                            | UnaryOp::OptionalSelected
                                            | UnaryOp::NonNull,
                                        ..
                                    }
                            ) && self.has_explicit_call_arguments())
                        {
                            self.schedule_call_work(value, optional, &mut work, &mut values)?;
                        } else {
                            values.push(self.finish_postfix(value, optional)?);
                        }
                    }
                    Work::Index { base, optional } => {
                        let index = values.pop().expect("index expression");
                        let end = self.expect("]")?;
                        let span = Span::new(base.span.start, end.span.end);
                        values.push(Expr::new(
                            ExprKind::Index(Box::new(base), Box::new(index)),
                            span,
                        ));
                        work.push(Work::Postfix { optional });
                    }
                    Work::Argument { mut call, spread } => {
                        let mut argument = values.pop().expect("call argument");
                        if let Some(operator) = spread {
                            let span = Span::new(operator.start, argument.span.end);
                            argument = Expr::new(
                                ExprKind::Unary {
                                    op: UnaryOp::Spread,
                                    operator,
                                    value: Box::new(argument),
                                },
                                span,
                            );
                        }
                        call.arguments.push(argument);
                        if self.accept(",") {
                            self.schedule_argument_work(call, &mut work);
                        } else {
                            self.finish_call_work(call, &mut work, &mut values)?;
                        }
                    }
                }
                Ok(())
            })();
            if let Err(error) = step {
                let mut recovered = false;
                while let Some(abandoned) = work.pop() {
                    match abandoned {
                        Work::ClosureValue { outer, .. } => self.pending = outer,
                        Work::BlockAfter { body, recovery }
                            if error.code == "syntax"
                                && self
                                    .diagnostics
                                    .is_some_and(|errors| errors.borrow().len() < 63) =>
                        {
                            self.diagnostics.unwrap().borrow_mut().push(error.clone());
                            self.depth = recovery.depth;
                            self.pending.truncate(recovery.pending);
                            self.call_type_arguments.truncate(recovery.arguments);
                            self.parameter_patterns.truncate(recovery.patterns);
                            values.truncate(recovery.expressions);
                            statements.truncate(recovery.statements);
                            bodies.truncate(recovery.bodies);
                            self.recover_statement(recovery.start);
                            work.push(Work::BlockNext(body));
                            recovered = true;
                            break;
                        }
                        _ => {}
                    }
                }
                if !recovered {
                    return Err(error);
                }
            }
        }
        assert_eq!(values.len(), 1, "one completed expression");
        Ok(values.pop().unwrap())
    }

    fn expect_record_close_work(
        &mut self,
        start: usize,
        fields: Vec<(Token, Expr)>,
        work: &mut Vec<Work>,
        values: &mut Vec<Expr>,
    ) -> Result<(), Diagnostic> {
        let end = self.expect("}")?.span.end;
        values.push(Expr::new(ExprKind::Record(fields), Span::new(start, end)));
        work.push(Work::Postfix { optional: false });
        Ok(())
    }

    fn schedule_call_work(
        &mut self,
        callee: Expr,
        optional: bool,
        work: &mut Vec<Work>,
        values: &mut Vec<Expr>,
    ) -> Result<(), Diagnostic> {
        let types = if self.accept("(") {
            None
        } else {
            let types = self.explicit_call_arguments()?;
            self.expect("(")?;
            Some(types)
        };
        let call = CallFrame {
            callee,
            arguments: Vec::new(),
            types,
            optional,
        };
        if self.is(")") {
            self.finish_call_work(call, work, values)
        } else {
            self.schedule_argument_work(call, work);
            Ok(())
        }
    }

    fn schedule_argument_work(&mut self, call: CallFrame, work: &mut Vec<Work>) {
        let spread = self.accept("...").then(|| self.tokens[self.i - 1].span);
        work.push(Work::Argument { call, spread });
        work.push(Work::Parse(0));
    }

    fn finish_call_work(
        &mut self,
        call: CallFrame,
        work: &mut Vec<Work>,
        values: &mut Vec<Expr>,
    ) -> Result<(), Diagnostic> {
        let end = self.expect(")")?;
        let value = if let Some(types) = call.types {
            self.finish_explicit_call(call.callee, types, call.arguments, end)?
        } else {
            Self::finish_call_expression(Box::new(call.callee), call.arguments, end)?
        };
        values.push(value);
        work.push(Work::Postfix {
            optional: call.optional,
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expression(source: &str) -> Expr {
        let mut parser = Parser::new(source, Default::default(), Default::default()).unwrap();
        let value = parser.expression().unwrap();
        assert_eq!(parser.peek().kind, TokenKind::End);
        assert_eq!(parser.depth, 0);
        value
    }

    #[test]
    fn queued_closure_blocks_recover_without_losing_the_outer_scope() {
        let source = "()=>{return f(1+); return 7;}";
        let errors = std::cell::RefCell::new(Vec::new());
        let mut parser = Parser::new(source, Default::default(), Default::default()).unwrap();
        parser.diagnostics = Some(&errors);
        let value = parser.expression().unwrap();
        let ExprKind::Closure { body, .. } = &value.kind else {
            panic!("closure");
        };
        assert_eq!(body.len(), 1);
        assert!(
            matches!(&body[0].kind, StmtKind::Return(Some(value)) if matches!(&value.kind, ExprKind::Integer(text) if text == "7"))
        );
        assert_eq!(errors.borrow().len(), 1);
        assert_eq!(errors.borrow()[0].code, "syntax");
        assert_eq!(parser.depth, 0);

        let source = "()=> (()=>{return +;})()";
        let mut parser = Parser::new(source, Default::default(), Default::default()).unwrap();
        parser.pending.push(Stmt {
            labels: Vec::new(),
            control_target: None,
            kind: StmtKind::Return(None),
            span: Span::new(0, 0),
        });
        assert!(parser.expression().is_err());
        assert_eq!(parser.pending.len(), 1);
        assert!(matches!(parser.pending[0].kind, StmtKind::Return(None)));
        assert_eq!(parser.depth, 0);
    }

    #[test]
    fn operator_frames_keep_associativity_and_token_provenance() {
        let value = expression("a = b = 2 ** 3 ** 4");
        let ExprKind::Assign {
            value, operator, ..
        } = &value.kind
        else {
            panic!("assignment");
        };
        assert_eq!(*operator, Span::new(2, 3));
        let ExprKind::Assign { value, .. } = &value.kind else {
            panic!("right associative assignment");
        };
        let ExprKind::Binary {
            op: BinaryOp::Exponent,
            right,
            ..
        } = &value.kind
        else {
            panic!("exponent");
        };
        assert!(matches!(
            right.kind,
            ExprKind::Binary {
                op: BinaryOp::Exponent,
                ..
            }
        ));
        expression("(a ?? b) || c");
        expression("(-2) ** 3");
        for (source, position) in [("-2 ** 3", 3), ("a ?? b || c", 2)] {
            let mut parser = Parser::new(source, Default::default(), Default::default()).unwrap();
            let error = parser.expression().err().expect("operator rejection");
            assert_eq!(error.code, "syntax");
            assert_eq!(error.span.start, position);
            assert_eq!(parser.depth, 0);
        }
    }

    #[test]
    fn suspended_literal_and_call_frames_restore_depth_after_syntax_failure() {
        let source = "f([1, {x: g(2 + )}])";
        let mut parser = Parser::new(source, Default::default(), Default::default()).unwrap();
        let error = parser.expression().err().expect("missing operand");
        assert_eq!(error.code, "syntax");
        assert_eq!(&source[error.span.start..error.span.end], ")");
        assert_eq!(parser.depth, 0);
    }

    #[test]
    fn mixed_group_record_array_call_and_index_frames_keep_source_extent() {
        let mut source = "7".to_owned();
        for _ in 0..16 {
            source = format!("identity(({{value:[{source}]}}).value[0])");
        }
        let value = expression(&source);
        assert_eq!(value.span, Span::new(0, source.len()));
        let source = format!(
            "function identity(value:int):int{{return value;}} function main(){{console.log({source});}}"
        );
        let formatted = crate::format(&source).unwrap();
        assert_eq!(crate::format(&formatted).unwrap(), formatted);
        parse(&formatted).unwrap();
    }
}
