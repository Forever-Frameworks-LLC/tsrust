//! Expression grammar steps of the parser scheduler.
use super::scheduler::{BinaryFrame, CallFrame, Callee, ClosureFrame, Machine, RightFrame, Work};
use super::*;

impl Parser<'_> {
    pub(super) fn expression_step(&mut self, task: Work, machine: &mut Machine) -> Result<(), Diagnostic> {
        match task {
            Work::Parse(minimum) => {
                if self.policy.reached(self.depth) {
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
                machine.work.push(Work::LeaveDepth);
                machine.work.push(Work::Binary(frame));
                machine.work.push(Work::Unary(operators));
                machine.work.push(Work::Primary);
            }
            Work::LeaveDepth => self.depth -= 1,
            Work::Unary(operators) => {
                let value = machine.value();
                machine.values.push(self.apply_unary_operators(value, operators)?);
            }
            Work::Primary => self.primary(machine)?,
            Work::Array { start, mut items, spread } => {
                let mut value = machine.value();
                if let Some(operator) = spread {
                    value = Self::spread_expression(operator, value);
                }
                items.push(value);
                if self.accept(",") && !self.is("]") {
                    let spread = self.accept("...").then(|| self.tokens[self.i - 1].span);
                    machine.request(Work::Array { start, items, spread }, Work::Parse(0));
                } else {
                    let end = self.expect("]")?.span.end;
                    machine.values.push(Expr::new(ExprKind::Array(items), Span::new(start, end)));
                    machine.work.push(Work::Postfix { optional: false });
                }
            }
            Work::RecordNext { start, mut fields } => loop {
                if self.is("}") {
                    let end = self.take().span.end;
                    machine.values.push(Expr::new(ExprKind::Record(fields), Span::new(start, end)));
                    machine.work.push(Work::Postfix { optional: false });
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
                        self.record_close(start, fields, machine)?;
                        break;
                    }
                } else {
                    if !spread {
                        self.expect(":")?;
                    }
                    machine.request(Work::RecordField { start, fields, key: Box::new(key) }, Work::Parse(0));
                    break;
                }
            },
            Work::RecordField { start, mut fields, key } => {
                fields.push((*key, machine.value()));
                if self.accept(",") {
                    machine.work.push(Work::RecordNext { start, fields });
                } else {
                    self.record_close(start, fields, machine)?;
                }
            }
            Work::Template { start, mut parts } => {
                parts.push(machine.value());
                let chunk = self.take();
                if !matches!(chunk.kind, TokenKind::TemplateMiddle | TokenKind::TemplateTail) {
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
                    self.postfix_depth(&value)?;
                    machine.values.push(value);
                    machine.work.push(Work::Postfix { optional: false });
                } else {
                    machine.request_comma(Work::Template { start, parts });
                }
            }
            Work::Group => {
                self.expect(")")?;
            }
            Work::Comma(mut operands) => {
                operands.push(machine.value());
                if self.accept(",") {
                    if matches!(self.peek().text.as_str(), "let" | "const" | "var") {
                        return Err(Diagnostic::new(
                            "syntax",
                            "A declaration cannot be a comma operand.",
                            self.peek().span,
                        ));
                    }
                    machine.request(Work::Comma(operands), Work::Parse(0));
                } else if operands.len() == 1 {
                    machine.values.push(operands.pop().unwrap());
                } else {
                    machine.values.push(Self::finish_sequence(operands)?);
                }
            }
            Work::Binary(mut frame) => {
                let mut left = machine.value();
                loop {
                    if frame.minimum <= 8 && (self.is("as") || self.is("satisfies")) {
                        left = self.type_assertion_tail(left)?;
                        frame.unary_base = false;
                        continue;
                    }
                    if frame.minimum == 0 && self.accept("?") {
                        machine.request(Work::Consequent(Box::new(left)), Work::Parse(0));
                        break;
                    }
                    if self.is("in") && frame.minimum <= 8 {
                        self.take();
                        machine.request(Work::In { frame, left: Box::new(left) }, Work::Parse(9));
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
                    let Some((op, precedence)) = operator.filter(|(_, precedence)| *precedence >= frame.minimum) else {
                        machine.values.push(left);
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
                    machine.request(
                        Work::Right(Box::new(RightFrame { frame, left, op, operator, assignment, right_start })),
                        Work::Parse(if assignment || op == BinaryOp::Exponent { precedence } else { precedence + 1 }),
                    );
                    break;
                }
            }
            Work::Right(suspended) => {
                let RightFrame { mut frame, left, op, operator, assignment, right_start } = *suspended;
                let right = machine.value();
                if !assignment {
                    self.nullish_mixing(&left, &right, op, operator, frame.left_start, right_start)?;
                }
                let mut value = Self::binary_expression(left, right, op, operator, assignment);
                if op == BinaryOp::Coalesce {
                    value.span = Span::new(self.tokens[frame.left_start].span.start, self.tokens[self.i - 1].span.end);
                }
                self.postfix_depth(&value)?;
                frame.unary_base = false;
                machine.values.push(value);
                machine.work.push(Work::Binary(frame));
            }
            Work::In { mut frame, left } => {
                let right = machine.value();
                let span = Span::new(self.tokens[frame.left_start].span.start, self.tokens[self.i - 1].span.end);
                let value = Self::object_key_call("hasOwn#in", vec![right, *left], span);
                self.postfix_depth(&value)?;
                frame.unary_base = false;
                machine.values.push(value);
                machine.work.push(Work::Binary(frame));
            }
            Work::Consequent(predicate) => {
                let consequent = machine.value();
                self.expect(":")?;
                machine.request(Work::Alternate(Box::new((*predicate, consequent))), Work::Parse(0));
            }
            Work::Alternate(arms) => {
                let (predicate, consequent) = *arms;
                let alternate = machine.value();
                let value = Self::conditional_expression(predicate, consequent, alternate);
                self.postfix_depth(&value)?;
                machine.values.push(value);
            }
            Work::Postfix { optional } => self.postfix(optional, machine)?,
            Work::Index { base, optional } => {
                let index = machine.value();
                let end = self.expect("]")?;
                let span = Span::new(base.span.start, end.span.end);
                machine.values.push(Expr::new(ExprKind::Index(base, Box::new(index)), span));
                machine.work.push(Work::Postfix { optional });
            }
            Work::Argument { mut call, spread } => {
                let mut argument = machine.value();
                if let Some(operator) = spread {
                    argument = Self::spread_expression(operator, argument);
                }
                call.arguments.push(argument);
                if self.accept(",") {
                    self.schedule_argument(call, machine);
                } else {
                    self.finish_call(call, machine)?;
                }
            }
            _ => unreachable!("expression work"),
        }
        Ok(())
    }

    fn spread_expression(operator: Span, value: Expr) -> Expr {
        let span = Span::new(operator.start, value.span.end);
        Expr::new(ExprKind::Unary { op: UnaryOp::Spread, operator, value: Box::new(value) }, span)
    }

    fn primary(&mut self, machine: &mut Machine) -> Result<(), Diagnostic> {
        self.typed_arrow_without_parentheses()?;
        if self.is("(") && !self.closure_header() {
            self.take();
            machine.work.push(Work::Postfix { optional: false });
            machine.request_comma(Work::Group);
        } else if self.peek().kind == TokenKind::TemplateHead {
            let head = self.take();
            let start = head.span.start;
            let parts = vec![Expr::new(ExprKind::String(head.text), head.span)];
            machine.request_comma(Work::Template { start, parts });
        } else if self.is("[") {
            let start = self.take().span.start;
            if self.is("]") {
                let end = self.take().span.end;
                machine.values.push(Expr::new(ExprKind::Array(Vec::new()), Span::new(start, end)));
                machine.work.push(Work::Postfix { optional: false });
            } else {
                let spread = self.accept("...").then(|| self.tokens[self.i - 1].span);
                machine.request(Work::Array { start, items: Vec::new(), spread }, Work::Parse(0));
            }
        } else if self.is("{") {
            let start = self.take().span.start;
            machine.work.push(Work::RecordNext { start, fields: Vec::new() });
        } else if self.is("function") {
            self.function_primary(machine)?;
        } else if self.is("(") || (self.peek().kind == TokenKind::Name && self.tokens.get(self.i + 1).is_some_and(|token| token.text == "=>")) {
            self.closure_primary(machine)?;
        } else if self.peek().kind == TokenKind::Name && self.peek().text == "new" {
            let start = self.take();
            self.constructor(start, machine)?;
        } else {
            machine.values.push(self.atom()?);
            machine.work.push(Work::Postfix { optional: false });
        }
        Ok(())
    }

    fn postfix(&mut self, mut optional: bool, machine: &mut Machine) -> Result<(), Diagnostic> {
        let mut value = machine.value();
        self.postfix_depth(&value)?;
        if self.is_tagged_template() {
            return self.reject_tagged_template().map(|_| ());
        }
        if self.accept("?.") {
            optional = true;
            let operator = self.tokens[self.i - 1].span;
            let span = value.span;
            value = Expr::new(ExprKind::Unary { op: UnaryOp::OptionalBoundary, operator, value: Box::new(value) }, span);
            if self.accept("[") {
                machine.request_comma(Work::Index { base: Box::new(value), optional });
            } else if self.is("(") || self.has_explicit_call_arguments() {
                self.schedule_call(value, optional, machine)?;
            } else {
                machine.values.push(self.field_expression(value)?);
                machine.work.push(Work::Postfix { optional });
            }
        } else if self.accept("!") {
            machine.values.push(self.non_null_postfix(value, optional));
            machine.work.push(Work::Postfix { optional: false });
        } else if self.accept(".") {
            machine.values.push(self.field_expression(value)?);
            machine.work.push(Work::Postfix { optional });
        } else if self.accept("[") {
            machine.request_comma(Work::Index { base: Box::new(value), optional });
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
                    | ExprKind::Unary { op: UnaryOp::OptionalBoundary | UnaryOp::OptionalSelected | UnaryOp::NonNull, .. }
            ) && self.has_explicit_call_arguments())
        {
            self.schedule_call(value, optional, machine)?;
        } else {
            machine.values.push(self.finish_postfix(value, optional)?);
        }
        Ok(())
    }

    fn record_close(&mut self, start: usize, fields: Vec<(Token, Expr)>, machine: &mut Machine) -> Result<(), Diagnostic> {
        let end = self.expect("}")?.span.end;
        machine.values.push(Expr::new(ExprKind::Record(fields), Span::new(start, end)));
        machine.work.push(Work::Postfix { optional: false });
        Ok(())
    }

    fn schedule_call(&mut self, callee: Expr, optional: bool, machine: &mut Machine) -> Result<(), Diagnostic> {
        let types = if self.accept("(") {
            None
        } else {
            let types = self.explicit_call_arguments()?;
            self.expect("(")?;
            Some(types)
        };
        self.schedule_arguments(CallFrame { target: Callee::Value { callee, types, optional }, arguments: Vec::new() }, machine)
    }

    /// The opening parenthesis is consumed. Arguments are requested one at a time.
    pub(super) fn schedule_arguments(&mut self, call: CallFrame, machine: &mut Machine) -> Result<(), Diagnostic> {
        let call = Box::new(call);
        if self.is(")") {
            self.finish_call(call, machine)
        } else {
            self.schedule_argument(call, machine);
            Ok(())
        }
    }

    fn schedule_argument(&mut self, call: Box<CallFrame>, machine: &mut Machine) {
        let spread = self.accept("...").then(|| self.tokens[self.i - 1].span);
        machine.request(Work::Argument { call, spread }, Work::Parse(0));
    }

    fn finish_call(&mut self, call: Box<CallFrame>, machine: &mut Machine) -> Result<(), Diagnostic> {
        let call = *call;
        let end = self.expect(")")?;
        let (value, optional) = match call.target {
            Callee::Value { callee, types: Some(types), optional } => (self.finish_explicit_call(callee, types, call.arguments, end)?, optional),
            Callee::Value { callee, types: None, optional } => (Self::finish_call_expression(Box::new(callee), call.arguments, end)?, optional),
            Callee::Constructor { start, native, map, annotation } => {
                let kind = if native {
                    ExprKind::NativeConstructor(call.arguments, annotation)
                } else if map {
                    ExprKind::MapConstructor(call.arguments, annotation)
                } else {
                    ExprKind::SetConstructor(call.arguments, annotation)
                };
                (Expr::new(kind, Span::new(start.start, end.span.end)), false)
            }
        };
        machine.values.push(value);
        machine.work.push(Work::Postfix { optional });
        Ok(())
    }

    /// Arrow function parameters, then its result annotation and body.
    fn closure_primary(&mut self, machine: &mut Machine) -> Result<(), Diagnostic> {
        let outer_patterns = std::mem::take(&mut self.parameter_patterns);
        let start = self.peek().span;
        if self.accept("(") {
            machine.request(
                Work::ClosureParameters { start, outer_patterns },
                Work::Parameters(Box::new(scheduler::ParametersFrame { params: Vec::new(), declared: false })),
            );
            return Ok(());
        }
        let name = self.name()?;
        let params = vec![CallableParameter {
            rest: false,
            ty: ParameterAnnotation::Inferred(name.clone()),
            name,
            access: None,
            initializer: ParameterInitializer::Required,
        }];
        self.closure_body(start, outer_patterns, params, machine)
    }

    pub(super) fn closure_body(&mut self, start: Span, outer_patterns: Vec<Stmt>, params: Vec<CallableParameter>, machine: &mut Machine) -> Result<(), Diagnostic> {
        let result = if self.accept(":") { self.result_annotation()? } else { ResultAnnotation::Inferred(start) };
        self.expect("=>")?;
        let id = self.next_closure;
        self.next_closure += 1;
        let patterns = std::mem::replace(&mut self.parameter_patterns, outer_patterns);
        let frame = Box::new(ClosureFrame { id, start, header: Box::new((params, result)), patterns });
        if self.is("{") {
            machine.request(Work::ClosureBlock(frame), Work::Block);
        } else {
            let outer = std::mem::take(&mut self.pending);
            machine.request(Work::ClosureValue { frame, outer }, Work::Parse(0));
        }
        Ok(())
    }

    /// `function (a: int): int { ... }` as a value: an anonymous function
    /// with the parameters, result and block body of an arrow function. A
    /// named function expression is declared in the enclosing block instead.
    fn function_primary(&mut self, machine: &mut Machine) -> Result<(), Diagnostic> {
        if self.tokens.get(self.i + 1).is_some_and(|token| token.kind == TokenKind::Name && token.text != "(") {
            // Statements pending for the enclosing block must not land in this body.
            let outer = std::mem::take(&mut self.pending);
            machine.work.push(Work::NamedFunctionExpression { outer });
            return self.local_function(machine);
        }
        let token = self.take();
        self.expect("(")?;
        machine.request(
            Work::FunctionParameters(token.span),
            Work::Parameters(Box::new(scheduler::ParametersFrame { params: Vec::new(), declared: false })),
        );
        Ok(())
    }

    /// `new Set<int>(...)`, `new Map<K, V>(...)` or a native constructor.
    fn constructor(&mut self, start: Token, machine: &mut Machine) -> Result<(), Diagnostic> {
        let name = self.name()?;
        let native = !matches!(name.text.as_str(), "Set" | "Map");
        let map = name.text == "Map";
        let annotation = if self.is("<") {
            let arguments = self.explicit_call_arguments()?;
            if !native && arguments.len() != if map { 2 } else { 1 } {
                return Err(Diagnostic::new("arity", "Set requires one key type; Map requires key and value types.", name.span));
            }
            Some(Box::new(crate::type_syntax::ValueAnnotation {
                kind: crate::type_syntax::ValueAnnotationKind::Applied {
                    name: crate::type_syntax::TypeName { text: name.text, span: name.span },
                    arguments,
                },
                span: name.span,
            }))
        } else if native {
            Some(Box::new(crate::type_syntax::ValueAnnotation {
                kind: crate::type_syntax::ValueAnnotationKind::Named(crate::type_syntax::TypeName { text: name.text, span: name.span }),
                span: name.span,
            }))
        } else {
            None
        };
        self.expect("(")?;
        self.schedule_arguments(CallFrame { target: Callee::Constructor { start: start.span, native, map, annotation }, arguments: Vec::new() }, machine)
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
